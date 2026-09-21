//! Cross-region private networking fabric core.
//!
//! Extends Render's same-region private networking with a global identity-aware fabric.
//! Does **not** implement volume/storage migration.

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod admission;
pub mod bypass;
pub mod connector;
pub mod control_plane;
pub mod dns;
pub mod drills;
pub mod endpoint;
pub mod environments;
pub mod error;
pub mod evacuate;
pub mod events_schema;
pub mod export;
pub mod fabric;
pub mod fuzz;
pub mod gateway;
pub mod global_dns;
pub mod graph;
pub mod handshake;
pub mod health;
pub mod identity;
pub mod ids;
pub mod ingress;
pub mod membership;
pub mod mtls;
pub mod namespace;
pub mod observability;
pub mod perf_gates;
pub mod platform;
pub mod policy;
pub mod policy_compiler;
pub mod policy_dist;
pub mod quarantine;
pub mod receipts;
pub mod registry;
pub mod relay;
pub mod route_security;
pub mod routing;
pub mod sim;
pub mod topology;
pub mod types;

pub use error::{FabricError, Result};
pub use fabric::FabricCore;
pub use ids::{NetworkID, NodeID, RegionID, ServiceID};
pub use platform::{PlatformAdapter, PlatformHooks, RenderAdapter};
pub use types::*;
