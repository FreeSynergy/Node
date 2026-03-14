// `fsn conductor` — container management via the Podman socket.
//
// Wraps PodmanClient (fsn-container) for direct container control from the CLI.
// For a graphical view, use `fsn tui` (opens fsd-conductor).

use anyhow::{bail, Result};
use fsn_container::PodmanClient;
use tokio::time::{sleep, Duration};

// ── start ──────────────────────────────────────────────────────────────────────

/// Start a stopped container by name.
pub async fn start(service: &str) -> Result<()> {
    let client = PodmanClient::new()?;
    client.start(service).await?;
    println!("Started: {service}");
    Ok(())
}

// ── stop ───────────────────────────────────────────────────────────────────────

/// Stop a running container by name.
pub async fn stop(service: &str) -> Result<()> {
    let client = PodmanClient::new()?;
    client.stop(service, None).await?;
    println!("Stopped: {service}");
    Ok(())
}

// ── restart ────────────────────────────────────────────────────────────────────

/// Restart a container by name.
pub async fn restart(service: &str) -> Result<()> {
    let client = PodmanClient::new()?;
    client.restart(service).await?;
    println!("Restarted: {service}");
    Ok(())
}

// ── logs ───────────────────────────────────────────────────────────────────────

/// Print recent log lines for a container.
///
/// When `follow` is `true`, polls for new lines every second until interrupted.
pub async fn logs(service: &str, follow: bool, tail: u64) -> Result<()> {
    let client = PodmanClient::new()?;

    // Verify container exists before entering any loop.
    if client.inspect(service).await?.is_none() {
        bail!("container not found: {service}");
    }

    if !follow {
        let lines = client.logs(service, Some(tail)).await?;
        for line in lines {
            println!("{line}");
        }
        return Ok(());
    }

    // Follow mode: print initial batch then poll for new lines.
    let mut printed = 0usize;
    loop {
        let lines = client.logs(service, Some(tail.max(printed as u64 + 1))).await?;
        for line in lines.iter().skip(printed) {
            println!("{line}");
        }
        printed = lines.len();
        sleep(Duration::from_secs(1)).await;
    }
}

// ── list ───────────────────────────────────────────────────────────────────────

/// List all containers with their current state.
pub async fn list(all: bool) -> Result<()> {
    let client = PodmanClient::new()?;
    let containers = client.list(all).await?;

    if containers.is_empty() {
        println!("No containers found.");
        return Ok(());
    }

    println!("{:<30} {:<12} {}", "NAME", "STATE", "IMAGE");
    println!("{}", "─".repeat(72));
    for c in &containers {
        println!("{:<30} {:<12} {}", c.name, format!("{:?}", c.state).to_lowercase(), c.image);
    }
    Ok(())
}
