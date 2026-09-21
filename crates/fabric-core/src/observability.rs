//! Machine-readable observability events.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// Fabric event kinds.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FabricEvent {
    /// Peer joined / session established.
    PeerUp {
        /// Peer node.
        node: String,
        /// Transport label.
        transport: String,
        /// RTT ms if known.
        rtt_ms: Option<f64>,
    },
    /// Peer down.
    PeerDown {
        /// Peer node.
        node: String,
        /// Reason.
        reason: String,
    },
    /// Service registered.
    ServiceRegistered {
        /// Service.
        service: String,
        /// Node.
        node: String,
        /// Region.
        region: String,
    },
    /// Service deregistered.
    ServiceDeregistered {
        /// Service.
        service: String,
        /// Node.
        node: String,
    },
    /// Route changed.
    RouteChanged {
        /// Service.
        service: String,
        /// From node.
        from: Option<String>,
        /// To node.
        to: String,
        /// Reason.
        reason: String,
    },
    /// Failed authentication / ingress.
    AuthFailed {
        /// Detail.
        detail: String,
        /// Client IP if any.
        client_ip: Option<String>,
    },
    /// Tunnel / path metrics sample.
    PathMetrics {
        /// Peer or path id.
        path: String,
        /// RTT.
        rtt_ms: f64,
        /// Loss.
        loss: f64,
        /// Bytes sent.
        bytes_sent: u64,
        /// Bytes recv.
        bytes_recv: u64,
        /// Connection count.
        connections: u32,
    },
    /// DNS resolve.
    DnsResolve {
        /// Name.
        name: String,
        /// Target node.
        target: String,
        /// Cross-region?
        cross_region: bool,
    },
}

/// Envelope with timestamp.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EventRecord {
    /// When.
    pub at: DateTime<Utc>,
    /// Payload.
    pub event: FabricEvent,
}

/// In-memory event bus (also serializable for export).
#[derive(Clone, Default)]
pub struct EventLog {
    inner: Arc<Mutex<Vec<EventRecord>>>,
}

impl EventLog {
    /// Create.
    pub fn new() -> Self {
        Self::default()
    }

    /// Emit.
    pub fn emit(&self, event: FabricEvent) {
        self.inner.lock().unwrap().push(EventRecord {
            at: Utc::now(),
            event,
        });
    }

    /// Snapshot.
    pub fn snapshot(&self) -> Vec<EventRecord> {
        self.inner.lock().unwrap().clone()
    }

    /// JSON lines export.
    pub fn to_jsonl(&self) -> String {
        self.snapshot()
            .iter()
            .filter_map(|e| serde_json::to_string(e).ok())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Count auth failures.
    pub fn auth_failure_count(&self) -> usize {
        self.snapshot()
            .iter()
            .filter(|e| matches!(e.event, FabricEvent::AuthFailed { .. }))
            .count()
    }
}
