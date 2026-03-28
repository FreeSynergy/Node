#![deny(clippy::all, clippy::pedantic, warnings)]
#![allow(clippy::module_name_repetitions)]
// fs-node-server — FreeSynergy.Node orchestration layer.
//
// Modules:
//   layer/    — NodeLayer trait + 4 concrete layers (Auth, S3, Proxy, Federation)
//   server    — NodeServer: composes layers, owns the HTTP API
//   invite/   — G1.6: InviteSystem, InviteToken, InviteBundle, PortPool
//   error     — NodeServerError

pub mod error;
pub mod invite;
pub mod layer;
pub mod server;

pub use error::NodeServerError;
pub use invite::{InviteBundle, InviteSystem, InviteToken, PortPool};
pub use layer::{AuthGateway, FederationGate, NodeLayer, S3Provider, ServiceProxy};
pub use server::{NodeApiConfig, NodeServer};
