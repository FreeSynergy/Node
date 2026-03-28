// fs-node-server/src/error.rs — NodeServerError hierarchy.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum NodeServerError {
    #[error("auth error: {0}")]
    Auth(String),

    #[error("storage error: {0}")]
    Storage(#[from] anyhow::Error),

    #[error("registry error: {0}")]
    Registry(#[from] fs_registry::RegistryError),

    #[error("invite error: {0}")]
    Invite(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
