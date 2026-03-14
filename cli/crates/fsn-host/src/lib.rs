//! Host management — SSH connections, remote install, and server provisioning.
//!
//! # Usage
//! ```no_run
//! # async fn example() -> anyhow::Result<()> {
//! use fsn_host::{RemoteHost, SshSession};
//!
//! let host = RemoteHost {
//!     name: "prod".into(),
//!     address: "192.168.1.10".into(),
//!     ssh_port: 22,
//!     ssh_user: "deploy".into(),
//!     ssh_key_path: Some("/home/user/.ssh/id_ed25519".into()),
//! };
//! let session = SshSession::connect(&host).await?;
//! session.exec("echo hello").await?;
//! session.close().await?;
//! # Ok(())
//! # }
//! ```

mod session;
mod systemd;

pub use session::{ExecOutput, SshSession};
pub use systemd::RemoteSystemd;

/// A remote host that FSN manages.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RemoteHost {
    pub name: String,
    pub address: String,
    pub ssh_port: u16,
    pub ssh_user: String,
    /// Path to the private key file. Falls back to SSH agent if None.
    pub ssh_key_path: Option<String>,
}

impl Default for RemoteHost {
    fn default() -> Self {
        Self {
            name: String::new(),
            address: String::new(),
            ssh_port: 22,
            ssh_user: "root".into(),
            ssh_key_path: None,
        }
    }
}
