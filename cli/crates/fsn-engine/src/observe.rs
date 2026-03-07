// Observe actual state – query systemd and podman for what is running.
// Replaces sync-stack.yml

use anyhow::Result;
use fsn_core::state::{ActualState, HealthStatus, RunState, ServiceStatus};
use fsn_podman::systemd;

/// Query the current state of all FSN-managed services on this host.
pub async fn observe() -> Result<ActualState> {
    let unit_names = systemd::list_fsn_units().await?;

    let mut services = Vec::with_capacity(unit_names.len());
    for unit in &unit_names {
        // Strip ".service" suffix to get the instance name
        let name = unit.trim_end_matches(".service").to_string();

        let unit_status = systemd::status(unit).await.unwrap_or(systemd::UnitStatus::NotFound);

        let run_state = match unit_status {
            systemd::UnitStatus::Active   => RunState::Running,
            systemd::UnitStatus::Inactive => RunState::Stopped,
            systemd::UnitStatus::Failed   => RunState::Failed,
            systemd::UnitStatus::NotFound => RunState::Missing,
        };

        services.push(ServiceStatus {
            name,
            state: run_state,
            health: HealthStatus::Unknown,   // HTTP health check is async / separate step
            deployed_version: read_deployed_version(unit).unwrap_or_default(),
            container_id: None,
        });
    }

    Ok(ActualState { services })
}

/// Read the deployed version from the state marker file.
fn read_deployed_version(unit_name: &str) -> Option<String> {
    let name  = unit_name.trim_end_matches(".service");
    let home  = std::env::var("HOME").ok()?;
    let path  = std::path::PathBuf::from(home)
        .join(".local/share/fsn/deployed")
        .join(format!("{}.version", name));
    std::fs::read_to_string(path).ok()?.lines().next().map(str::to_owned)
}
