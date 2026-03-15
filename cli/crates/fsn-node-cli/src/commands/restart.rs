use std::path::Path;
use anyhow::Result;
use fsn_container::SystemctlManager;

/// Restart one or all FSN-managed services.
pub async fn run(_root: &Path, _project: Option<&Path>, service: Option<&str>) -> Result<()> {
    let systemd = SystemctlManager::user();
    if let Some(name) = service {
        let unit = format!("{}.service", name);
        systemd.stop(&unit).await.map_err(anyhow::Error::from)?;
        systemd.start(&unit).await.map_err(anyhow::Error::from)?;
        println!("Restarted {}", name);
    } else {
        let units = fsn_deploy::observe::list_fsn_units(&systemd).await?;
        for unit in &units {
            let _ = systemd.stop(unit).await;
            let _ = systemd.start(unit).await;
            println!("Restarted {}", unit.trim_end_matches(".service"));
        }
    }
    Ok(())
}
