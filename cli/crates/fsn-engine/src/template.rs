// Jinja2-compatible template engine – wraps minijinja.
//
// The existing .j2 templates in playbooks/templates/ work unchanged.
// Variable names match the Ansible template context (instance_name,
// project_root, service_domain, vault_*, ...).

use anyhow::Result;
use minijinja::Environment;
use std::collections::HashMap;

use fsn_core::config::VaultConfig;

/// Template rendering context – mirrors the Ansible variable namespace.
pub struct TemplateContext<'a> {
    pub project_name: &'a str,
    pub project_domain: &'a str,
    pub instance_name: &'a str,
    pub service_domain: &'a str,
    pub parent_instance_name: &'a str,
    pub vault: &'a VaultConfig,
}

/// Render a single Jinja2 template string with the given context.
pub fn render(template: &str, ctx: &TemplateContext) -> Result<String> {
    let mut env = Environment::new();

    // Build variable map – includes core vars plus vault secrets
    let mut vars: HashMap<String, minijinja::Value> = [
        ("project_name",          ctx.project_name),
        ("project_domain",        ctx.project_domain),
        ("instance_name",         ctx.instance_name),
        ("service_domain",        ctx.service_domain),
        ("parent_instance_name",  ctx.parent_instance_name),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), minijinja::Value::from(v)))
    .collect();

    // Inject vault secrets (vault_* keys) into the template context.
    // Vault values are exposed only at render time, never stored as plain strings.
    for key in ctx.vault.keys() {
        if let Some(exposed) = ctx.vault.expose(key) {
            vars.insert(key.to_string(), minijinja::Value::from(exposed));
        }
    }

    let tmpl = env.template_from_str(template)?;
    Ok(tmpl.render(vars)?)
}

/// Render a multi-line template file (e.g. container.quadlet.j2).
pub fn render_file(template_content: &str, ctx: &TemplateContext) -> Result<String> {
    render(template_content, ctx)
}
