// commands/node.rs — `fsn node` subcommand: server lifecycle + invite management.
//
// Subcommands:
//   fsn node serve                     — start full NodeServer (Auth+S3+Proxy+API)
//   fsn node invite create             — generate a new invite token + encrypted bundle
//   fsn node invite list               — show active invites
//   fsn node invite accept <token>     — accept an invite from another node

use anyhow::Result;
use chrono::Utc;
use fs_auth::capabilities::ProtocolSupport;
use fs_auth::error::AuthError;
use fs_auth::{AuthCapabilities, OAuthProvider, SsoProvider};
use fs_auth::{AuthorizationUrl, SsoSession, TokenPair};
use fs_node_server::{
    invite::InviteSystem,
    layer::{
        auth::AuthGateway,
        federation::{FederationConfig, FederationGate},
        proxy::ServiceProxy,
        storage::S3Provider,
    },
    server::{NodeApiConfig, NodeServer},
    InviteBundle,
};
use fs_s3::StorageConfig;

use crate::cli::{InviteCommand, NodeCommand};

// ── command dispatch ──────────────────────────────────────────────────────────

pub async fn run(root: &std::path::Path, cmd: NodeCommand) -> Result<()> {
    match cmd {
        NodeCommand::Serve { bind, port, domain } => {
            serve(root, &bind, port, domain.as_deref()).await
        }
        NodeCommand::Invite { cmd } => match cmd {
            InviteCommand::Create {
                ttl_hours,
                label,
                port_min,
                port_max,
            } => invite_create(ttl_hours, label, port_min, port_max),
            InviteCommand::Accept { token, bundle } => invite_accept(&token, &bundle).await,
        },
    }
}

// ── fsn node serve ────────────────────────────────────────────────────────────

async fn serve(root: &std::path::Path, bind: &str, port: u16, domain: Option<&str>) -> Result<()> {
    let storage_config = StorageConfig {
        enabled: true,
        port: 9000,
        bind: "127.0.0.1".to_owned(),
        data_root: root.join("storage"),
        access_key: "fs_local".to_owned(),
        secret_key: "changeme_secret_key".to_owned(),
        sync: None,
    };

    let node_domain = domain.unwrap_or("localhost");
    let node_url = format!("http://{bind}:{port}");

    let auth = AuthGateway::new(
        "stub",
        AuthCapabilities {
            oauth: ProtocolSupport::Unsupported,
            scim: ProtocolSupport::Unsupported,
            sso: ProtocolSupport::Unsupported,
            pam: ProtocolSupport::Unsupported,
        },
        Box::new(StubOAuth),
        Box::new(StubSso),
    );
    let storage = S3Provider::new(storage_config);
    let proxy = ServiceProxy::open(root.join("registry.db").to_str().unwrap_or(":memory:")).await?;
    let federation = FederationGate::new(FederationConfig::new(node_domain));
    let invites = InviteSystem::new(
        b"change-me-32-byte-node-secret!!".to_vec(),
        9100,
        9200,
        &node_url,
    );

    let server = NodeServer::new(
        auth,
        storage,
        proxy,
        federation,
        invites,
        NodeApiConfig::new(bind, port),
    );

    server.run().await
}

// ── fsn node invite create ────────────────────────────────────────────────────

fn invite_create(
    ttl_hours: u64,
    label: Option<String>,
    port_min: u16,
    port_max: u16,
) -> Result<()> {
    let system = InviteSystem::new(
        b"change-me-32-byte-node-secret!!".to_vec(),
        port_min,
        port_max,
        "https://node.example.com", // TODO(G1.5): read from node config
    );

    let ttl_secs = i64::try_from(ttl_hours * 3600).unwrap_or(i64::MAX);
    let (token, bundle, port) = system.create(ttl_secs, label.clone())?;

    let expires = Utc::now() + chrono::Duration::seconds(ttl_secs);

    println!("Invite created:");
    println!("  Token   : {token}");
    println!("  Port    : {port}");
    println!("  Expires : {}", expires.format("%Y-%m-%d %H:%M UTC"));
    if let Some(l) = label {
        println!("  Label   : {l}");
    }
    println!();
    println!("Encrypted bundle (share with invited node):");
    println!("{bundle}");

    Ok(())
}

// ── fsn node invite accept ────────────────────────────────────────────────────

async fn invite_accept(token: &str, bundle: &str) -> Result<()> {
    let decoded = InviteBundle::decrypt(bundle, token)
        .map_err(|e| anyhow::anyhow!("failed to decrypt invite bundle: {e}"))?;

    println!("Invite bundle decrypted:");
    println!("  Node URL   : {}", decoded.node_url);
    println!("  Invite Port: {}", decoded.invite_port);
    println!(
        "  Expires    : {}",
        decoded.expires_at.format("%Y-%m-%d %H:%M UTC")
    );
    if let Some(label) = &decoded.label {
        println!("  Label      : {label}");
    }
    println!();
    println!(
        "TODO: establish connection to {} on port {}",
        decoded.node_url, decoded.invite_port
    );

    Ok(())
}

// ── Stub protocol implementations (until G1.2 KanidmBackend is ready) ────────

struct StubOAuth;

#[async_trait::async_trait]
impl OAuthProvider for StubOAuth {
    async fn authorize(&self, _: &str, _: &str, _: &str) -> Result<AuthorizationUrl, AuthError> {
        Err(AuthError::NotImplemented("stub backend"))
    }
    async fn exchange_code(&self, _: &str, _: &str) -> Result<TokenPair, AuthError> {
        Err(AuthError::NotImplemented("stub backend"))
    }
    async fn refresh_token(&self, _: &str) -> Result<TokenPair, AuthError> {
        Err(AuthError::NotImplemented("stub backend"))
    }
    async fn revoke_token(&self, _: &str) -> Result<(), AuthError> {
        Err(AuthError::NotImplemented("stub backend"))
    }
}

struct StubSso;

#[async_trait::async_trait]
impl SsoProvider for StubSso {
    async fn validate_session(&self, _: &str) -> Result<SsoSession, AuthError> {
        Err(AuthError::NotImplemented("stub backend"))
    }
    async fn create_session(&self, _: &str) -> Result<SsoSession, AuthError> {
        Err(AuthError::NotImplemented("stub backend"))
    }
    async fn invalidate_session(&self, _: &str) -> Result<(), AuthError> {
        Err(AuthError::NotImplemented("stub backend"))
    }
}
