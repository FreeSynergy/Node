// fs-node-server/src/server.rs — NodeServer: composes all orchestration layers.
//
// NodeServer owns all five layers and starts them in order:
//   1. AuthGateway  — auth backend connection
//   2. S3Provider   — embedded object storage
//   3. ServiceProxy — capability registry
//   4. FederationGate — well-known discovery (Phase-P stub)
//   5. NodeApi      — HTTP API (axum), routes from all layers

use std::sync::Arc;

use anyhow::Result;
use axum::{extract::State, response::Json, routing::get, Router};
use serde_json::json;
use tokio::task::JoinHandle;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

use crate::{
    invite::InviteSystem,
    layer::{AuthGateway, FederationGate, NodeLayer, S3Provider, ServiceProxy},
};

// ── NodeConfig ────────────────────────────────────────────────────────────────

/// Bind address + port for the Node HTTP API.
#[derive(Debug, Clone)]
pub struct NodeApiConfig {
    pub bind: String,
    pub port: u16,
}

impl NodeApiConfig {
    #[must_use]
    pub fn new(bind: impl Into<String>, port: u16) -> Self {
        Self {
            bind: bind.into(),
            port,
        }
    }

    #[must_use]
    pub fn addr(&self) -> String {
        format!("{}:{}", self.bind, self.port)
    }
}

// ── Shared API state ──────────────────────────────────────────────────────────

#[derive(Clone)]
struct ApiState {
    node_url: String,
    federation_domain: String,
    available_invite_ports: usize,
}

// ── NodeServer ────────────────────────────────────────────────────────────────

/// The FreeSynergy.Node orchestration server.
///
/// Owns all five layers and manages their lifecycle.
pub struct NodeServer {
    auth: Arc<AuthGateway>,
    storage: Arc<S3Provider>,
    proxy: Arc<ServiceProxy>,
    federation: Arc<FederationGate>,
    invites: Arc<InviteSystem>,
    api_config: NodeApiConfig,
}

impl NodeServer {
    /// Create a new `NodeServer` from the five layers plus API config.
    #[must_use]
    pub fn new(
        auth: AuthGateway,
        storage: S3Provider,
        proxy: ServiceProxy,
        federation: FederationGate,
        invites: InviteSystem,
        api_config: NodeApiConfig,
    ) -> Self {
        Self {
            auth: Arc::new(auth),
            storage: Arc::new(storage),
            proxy: Arc::new(proxy),
            federation: Arc::new(federation),
            invites: Arc::new(invites),
            api_config,
        }
    }

    /// Start all layers and block serving the Node HTTP API.
    ///
    /// Call from `fsn node serve` or `fsn serve`.
    ///
    /// # Errors
    ///
    /// Returns the first layer start error or the axum bind error.
    pub async fn run(self) -> Result<()> {
        // Start all layers in order
        for layer in self.layers() {
            layer.start().await?;
        }

        // S3 server in background task
        let _s3_handle: JoinHandle<()> = self.storage.start_server().await?;

        // Build axum router
        let state = ApiState {
            node_url: format!("http://{}", self.api_config.addr()),
            federation_domain: self.federation.domain().to_owned(),
            available_invite_ports: self.invites.available_ports(),
        };

        let app = Router::new()
            .route("/api/node/health", get(handle_health))
            .route("/api/node/capabilities", get(handle_capabilities))
            .route("/api/node/well-known", get(handle_well_known))
            .route("/api/node/invites/status", get(handle_invite_status))
            .layer(
                CorsLayer::new()
                    .allow_origin(Any)
                    .allow_methods(Any)
                    .allow_headers(Any),
            )
            .with_state(state);

        let addr = self.api_config.addr();
        let listener = tokio::net::TcpListener::bind(&addr).await?;

        info!("NodeServer API listening on http://{addr}");
        println!("Node API : http://{addr}/api/node/");
        println!("S3 API   : {}", self.storage.endpoint());
        println!("Press Ctrl+C to stop.");

        axum::serve(listener, app).await?;
        Ok(())
    }

    fn layers(&self) -> [&dyn NodeLayer; 4] {
        [
            self.auth.as_ref(),
            self.storage.as_ref(),
            self.proxy.as_ref(),
            self.federation.as_ref(),
        ]
    }
}

// ── HTTP handlers ─────────────────────────────────────────────────────────────

async fn handle_health(State(_): State<ApiState>) -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "layers": ["auth-gateway", "s3-provider", "service-proxy", "federation-gate"]
    }))
}

async fn handle_capabilities(State(_): State<ApiState>) -> Json<serde_json::Value> {
    // TODO(G3): query ServiceProxy for live capabilities once bus wiring is done
    Json(json!({
        "capabilities": []
    }))
}

async fn handle_well_known(State(state): State<ApiState>) -> Json<serde_json::Value> {
    Json(json!({
        "domain": state.federation_domain,
        "node_url": state.node_url,
    }))
}

async fn handle_invite_status(State(state): State<ApiState>) -> Json<serde_json::Value> {
    Json(json!({
        "available_ports": state.available_invite_ports,
    }))
}
