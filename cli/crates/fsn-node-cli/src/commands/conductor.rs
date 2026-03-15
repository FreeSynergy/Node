// `fsn conductor` — container management via systemctl + podman CLI.
//
// Uses SystemctlManager (fsn-container) for systemd unit operations and
// invokes `podman` as a subprocess for container-level queries.
//
// For a graphical view, use `fsn tui` (opens fsd-conductor).

use anyhow::{bail, Result};
use fsn_container::SystemctlManager;

// ── Conductor (Facade) ────────────────────────────────────────────────────────

/// Facade over SystemctlManager for direct container management from the CLI.
pub struct Conductor {
    systemd: SystemctlManager,
}

impl Conductor {
    /// Create a new `Conductor` using the user systemd session.
    pub fn new() -> Result<Self> {
        Ok(Self { systemd: SystemctlManager::user() })
    }

    // ── start ─────────────────────────────────────────────────────────────────

    /// Start a stopped service by name.
    pub async fn start(&self, service: &str) -> Result<()> {
        let unit = unit_name(service);
        self.systemd.start(&unit).await.map_err(anyhow::Error::from)?;
        println!("Started: {service}");
        Ok(())
    }

    // ── stop ──────────────────────────────────────────────────────────────────

    /// Stop a running service by name.
    pub async fn stop(&self, service: &str) -> Result<()> {
        let unit = unit_name(service);
        self.systemd.stop(&unit).await.map_err(anyhow::Error::from)?;
        println!("Stopped: {service}");
        Ok(())
    }

    // ── restart ───────────────────────────────────────────────────────────────

    /// Restart a service by name.
    pub async fn restart(&self, service: &str) -> Result<()> {
        let unit = unit_name(service);
        self.systemd.restart(&unit).await.map_err(anyhow::Error::from)?;
        println!("Restarted: {service}");
        Ok(())
    }

    // ── logs ──────────────────────────────────────────────────────────────────

    /// Print recent log lines for a container via `podman logs`.
    ///
    /// When `follow` is `true`, passes `--follow` to podman.
    pub async fn logs(&self, service: &str, follow: bool, tail: u64) -> Result<()> {
        let container_name = service;
        let exists = podman_container_exists(container_name).await?;
        if !exists {
            bail!("container not found: {service}");
        }

        if !follow {
            let output = tokio::process::Command::new("podman")
                .args(["logs", "--tail", &tail.to_string(), container_name])
                .output()
                .await?;
            let text = String::from_utf8_lossy(&output.stdout);
            let err  = String::from_utf8_lossy(&output.stderr);
            print!("{}{}", text, err);
            return Ok(());
        }

        // Follow mode: run podman logs --follow (blocks until Ctrl-C)
        let mut child = tokio::process::Command::new("podman")
            .args(["logs", "--follow", "--tail", &tail.to_string(), container_name])
            .spawn()?;
        child.wait().await?;
        Ok(())
    }

    // ── list ──────────────────────────────────────────────────────────────────

    /// List all FSN-managed services with their current state.
    pub async fn list(&self, _all: bool) -> Result<()> {
        let output = tokio::process::Command::new("systemctl")
            .args([
                "--user", "--type=service",
                "--plain", "--no-legend", "--no-pager",
                "--state=loaded",
            ])
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let units: Vec<&str> = stdout
            .lines()
            .filter(|l| l.contains("fsn-"))
            .collect();

        if units.is_empty() {
            println!("No FSN-managed services found.");
            return Ok(());
        }

        println!("{:<32} {:<12} {}", "SERVICE", "ACTIVE", "SUB");
        println!("{}", "─".repeat(60));
        for line in units {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let name   = parts[0].trim_end_matches(".service");
                let active = parts.get(2).copied().unwrap_or("-");
                let sub    = parts.get(3).copied().unwrap_or("-");
                println!("{:<32} {:<12} {}", name, active, sub);
            }
        }
        Ok(())
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Convert a service name to a systemd unit name.
fn unit_name(service: &str) -> String {
    if service.ends_with(".service") {
        service.to_string()
    } else {
        format!("{}.service", service)
    }
}

/// Returns `true` if a podman container with the given name exists.
async fn podman_container_exists(name: &str) -> Result<bool> {
    let output = tokio::process::Command::new("podman")
        .args(["container", "exists", name])
        .output()
        .await?;
    Ok(output.status.success())
}
