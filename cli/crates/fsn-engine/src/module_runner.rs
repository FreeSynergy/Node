// Module plugin runner — process-based plugin protocol (stdin/stdout JSON).
//
// Design Pattern: Command (each plugin invocation is a self-contained command).
//
// Flow:
//   1. Core builds a PluginContext (what the plugin needs to know)
//   2. Runner serialises it as JSON → writes to plugin's stdin
//   3. Plugin process produces a PluginResponse on stdout
//   4. Runner deserialises, then Core acts on the response:
//      - writes declared files
//      - runs declared shell commands (in order)
//
// The plugin executable lives at: {store_module_dir}/plugin
// (or the path declared in ModuleManifest::executable, if added later)
//
// Error handling: any I/O or parse failure returns Err(FsnError).
// If PluginResponse::error is non-empty, Runner returns Err with that message.

use std::fs;
use std::io::Write as _;
use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use fsn_core::config::manifest::{PluginContext, PluginResponse};
use fsn_core::error::FsnError;

// ── ModuleRunner ──────────────────────────────────────────────────────────────

/// Runs a module plugin and applies the resulting file writes and shell commands.
pub struct ModuleRunner {
    /// Root of the Store directory for this module.
    /// E.g. `/home/kal/Server/Store/Node/proxy/zentinel`
    pub store_module_dir: PathBuf,
}

impl ModuleRunner {
    pub fn new(store_module_dir: impl Into<PathBuf>) -> Self {
        Self { store_module_dir: store_module_dir.into() }
    }

    /// Invoke the plugin with the given context and apply the response.
    ///
    /// Returns `Ok(PluginResponse)` on success — caller can inspect logs.
    /// Returns `Err` if the plugin process fails, produces invalid JSON,
    /// or reports a non-empty `error` field.
    pub fn run(&self, context: &PluginContext) -> Result<PluginResponse, FsnError> {
        let executable = self.store_module_dir.join("plugin");
        if !executable.exists() {
            return Err(FsnError::ConstraintViolation {
                message: format!(
                    "plugin executable not found: {}",
                    executable.display()
                ),
            });
        }

        let ctx_json = serde_json::to_string(context)
            .map_err(|e| FsnError::ConstraintViolation {
                message: format!("failed to serialise PluginContext: {e}"),
            })?;

        let mut child = Command::new(&executable)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())   // plugin stderr goes to our stderr (visible in logs)
            .spawn()
            .map_err(|e| FsnError::ConstraintViolation {
                message: format!("failed to spawn plugin {}: {e}", executable.display()),
            })?;

        // Write context JSON to stdin
        if let Some(stdin) = child.stdin.take() {
            let mut stdin = stdin;
            stdin.write_all(ctx_json.as_bytes())
                .map_err(|e| FsnError::ConstraintViolation {
                    message: format!("failed to write to plugin stdin: {e}"),
                })?;
            // stdin is dropped here, closing the pipe → plugin sees EOF
        }

        let output = child.wait_with_output()
            .map_err(|e| FsnError::ConstraintViolation {
                message: format!("plugin process failed: {e}"),
            })?;

        if !output.status.success() {
            return Err(FsnError::ConstraintViolation {
                message: format!(
                    "plugin exited with status {} (command: {})",
                    output.status, context.command
                ),
            });
        }

        let response: PluginResponse = serde_json::from_slice(&output.stdout)
            .map_err(|e| FsnError::ConstraintViolation {
                message: format!("invalid JSON from plugin stdout: {e}"),
            })?;

        if !response.error.is_empty() {
            return Err(FsnError::ConstraintViolation {
                message: format!("plugin error: {}", response.error),
            });
        }

        if response.protocol != 1 {
            return Err(FsnError::ConstraintViolation {
                message: format!(
                    "unsupported plugin protocol version {} (expected 1)",
                    response.protocol
                ),
            });
        }

        Ok(response)
    }

    /// Apply a PluginResponse: write files then run shell commands.
    ///
    /// Call this after `run()` once you've decided to commit the changes.
    pub fn apply(&self, response: &PluginResponse) -> Result<(), FsnError> {
        self.write_files(response)?;
        self.run_commands(response)?;
        Ok(())
    }

    // ── Private helpers ───────────────────────────────────────────────────────

    fn write_files(&self, response: &PluginResponse) -> Result<(), FsnError> {
        for file in &response.files {
            let dest = Path::new(&file.dest);

            // Ensure parent directory exists
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| FsnError::ConstraintViolation {
                        message: format!("failed to create dir {}: {e}", parent.display()),
                    })?;
            }

            fs::write(dest, &file.content)
                .map_err(|e| FsnError::ConstraintViolation {
                    message: format!("failed to write {}: {e}", dest.display()),
                })?;

            // Apply permissions
            fs::set_permissions(dest, fs::Permissions::from_mode(file.mode))
                .map_err(|e| FsnError::ConstraintViolation {
                    message: format!("failed to chmod {}: {e}", dest.display()),
                })?;
        }
        Ok(())
    }

    fn run_commands(&self, response: &PluginResponse) -> Result<(), FsnError> {
        for cmd in &response.commands {
            let mut builder = Command::new("sh");
            builder.arg("-c").arg(&cmd.cmd);

            if let Some(cwd) = &cmd.cwd {
                builder.current_dir(cwd);
            }

            for (k, v) in &cmd.env {
                builder.env(k, v);
            }

            let status = builder
                .status()
                .map_err(|e| FsnError::ConstraintViolation {
                    message: format!("failed to run command `{}`: {e}", cmd.cmd),
                })?;

            if !status.success() {
                return Err(FsnError::ConstraintViolation {
                    message: format!(
                        "command failed (exit {}): {}",
                        status.code().unwrap_or(-1),
                        cmd.cmd
                    ),
                });
            }
        }
        Ok(())
    }
}

// ── ContextBuilder ────────────────────────────────────────────────────────────

/// Convenience builder for constructing a PluginContext from engine types.
pub struct ContextBuilder;

impl ContextBuilder {
    /// Build a PluginContext from a resolved ServiceInstance and its peers.
    pub fn build(
        command: &str,
        instance: &fsn_core::state::desired::ServiceInstance,
        project_domain: &str,
        data_root: &str,
        peers: &[&fsn_core::state::desired::ServiceInstance],
    ) -> PluginContext {
        use fsn_core::config::manifest::{InstanceInfo, PeerRoute, PeerService};
        use fsn_core::resource::VarProvider as _;

        let peer_services = peers.iter().map(|p| {
            let routes = p.class.contract.routes.iter().map(|r| PeerRoute {
                id: r.id.clone(),
                path: r.path.clone(),
                strip: r.strip,
            }).collect();

            PeerService {
                name: p.name.clone(),
                class_key: p.class_key.clone(),
                types: p.service_types.iter().map(|t| t.to_string()).collect(),
                domain: p.service_domain.clone(),
                port: p.class.meta.port,
                upstream_tls: p.class.contract.upstream_tls,
                routes,
                exported_vars: p.exported_vars(),
            }
        }).collect();

        // Collect all peer exported vars into a single env map
        let env: std::collections::HashMap<String, String> = peers.iter()
            .flat_map(|p| p.exported_vars())
            .collect();

        PluginContext {
            protocol: 1,
            command: command.to_string(),
            instance: InstanceInfo {
                name: instance.name.clone(),
                class_key: instance.class_key.clone(),
                domain: instance.service_domain.clone(),
                project: project_domain.split('.').next().unwrap_or("").to_string(),
                project_domain: project_domain.to_string(),
                data_root: data_root.to_string(),
                env: instance.resolved_env.clone(),
            },
            peers: peer_services,
            env,
        }
    }
}
