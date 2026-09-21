//! Shared fabric enums and value types.

use crate::ids::{NetworkID, NodeID, RegionID, ServiceID};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Supported fabric protocol major version.
pub const PROTOCOL_VERSION: u16 = 1;

/// Health of a peer, link, or service instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum HealthState {
    /// Passing probes within thresholds.
    #[default]
    Healthy,
    /// Reachable but degraded (high latency / loss).
    Degraded,
    /// Confirmed unreachable.
    Unreachable,
    /// No recent observations.
    Unknown,
}

impl HealthState {
    /// Whether traffic may still be routed here (possibly with caution).
    pub fn is_routable(self) -> bool {
        matches!(self, HealthState::Healthy | HealthState::Degraded)
    }
}

/// How traffic reaches a peer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TransportMode {
    /// Authenticated direct peer tunnel.
    Direct,
    /// Authenticated path via an authorized relay.
    Relayed,
    /// No usable path.
    Unavailable,
}

impl TransportMode {
    /// Human-readable label; never labels relayed as direct.
    pub fn as_str(self) -> &'static str {
        match self {
            TransportMode::Direct => "DIRECT",
            TransportMode::Relayed => "RELAYED",
            TransportMode::Unavailable => "UNAVAILABLE",
        }
    }
}

/// Topology strategy for the fabric.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TopologyMode {
    /// Every region peers with every other.
    FullMesh,
    /// One hub region; others are spokes.
    HubSpoke,
    /// Deterministic dynamic selection from metrics.
    Dynamic,
}

/// Policy action.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PolicyAction {
    /// Permit the flow.
    Allow,
    /// Deny the flow.
    Deny,
}

/// Registered service instance metadata.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ServiceInstance {
    /// Service logical id.
    pub service: ServiceID,
    /// Hosting node.
    pub node: NodeID,
    /// Region of the instance.
    pub region: RegionID,
    /// Network membership.
    pub network: NetworkID,
    /// Private listen port (logical).
    pub port: u16,
    /// Application protocol (http, grpc, tcp, …).
    pub protocol: String,
    /// Current health.
    pub health: HealthState,
    /// Observed one-way latency hint in milliseconds.
    pub latency_ms: f64,
    /// Packet loss ratio 0.0–1.0.
    pub loss: f64,
    /// Capability tags.
    pub capabilities: Vec<String>,
    /// Registration timestamp.
    pub registered_at: DateTime<Utc>,
    /// Last health observation.
    pub last_seen: DateTime<Utc>,
}

impl ServiceInstance {
    /// Create a new healthy instance.
    pub fn new(
        service: impl Into<ServiceID>,
        node: impl Into<NodeID>,
        region: impl Into<RegionID>,
        network: impl Into<NetworkID>,
        port: u16,
    ) -> Self {
        let now = Utc::now();
        Self {
            service: service.into(),
            node: node.into(),
            region: region.into(),
            network: network.into(),
            port,
            protocol: "http".into(),
            health: HealthState::Healthy,
            latency_ms: 1.0,
            loss: 0.0,
            capabilities: Vec::new(),
            registered_at: now,
            last_seen: now,
        }
    }
}

/// Peer membership record after successful handshake.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PeerInfo {
    /// Peer node id.
    pub node: NodeID,
    /// Peer's region.
    pub region: RegionID,
    /// Shared network.
    pub network: NetworkID,
    /// Services the peer may represent.
    pub services: Vec<ServiceID>,
    /// Negotiated protocol version.
    pub protocol_version: u16,
    /// Current transport.
    pub transport: TransportMode,
    /// Health of the peer link.
    pub health: HealthState,
    /// Round-trip time milliseconds.
    pub rtt_ms: f64,
    /// Packet loss on the link.
    pub loss: f64,
    /// Active logical connections via this peer.
    pub connection_count: u32,
    /// Bytes transferred (simulated / measured).
    pub bytes_sent: u64,
    /// Bytes received.
    pub bytes_recv: u64,
}

/// Link metrics between two regions/nodes used by topology & routing.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LinkMetrics {
    /// Source region.
    pub from: RegionID,
    /// Destination region.
    pub to: RegionID,
    /// Latency milliseconds.
    pub latency_ms: f64,
    /// Loss ratio.
    pub loss: f64,
    /// Link health.
    pub health: HealthState,
    /// Relative traffic demand score (higher = more demand).
    pub traffic_demand: f64,
}

/// Failover timing breakdown.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct FailoverMetrics {
    /// Time to detect failure (ms).
    pub failure_detection_ms: f64,
    /// Time to recalculate routes (ms).
    pub route_recalculation_ms: f64,
    /// Time to re-establish sessions (ms).
    pub reconnection_ms: f64,
    /// Total end-to-end failover (ms).
    pub total_failover_ms: f64,
    /// Previous target node.
    pub from_node: Option<NodeID>,
    /// New target node.
    pub to_node: Option<NodeID>,
    /// Reason string.
    pub reason: String,
}
