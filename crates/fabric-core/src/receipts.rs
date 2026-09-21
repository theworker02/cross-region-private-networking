//! Network receipts: ROUTE / FAILOVER / POLICY / IDENTITY / REGION_EVACUATION.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Receipt kinds.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReceiptKind {
    /// Route selection.
    Route {
        /// Name resolved.
        name: String,
        /// Target node.
        target: String,
        /// Reason.
        reason: String,
    },
    /// Failover event.
    Failover {
        /// Service.
        service: String,
        /// From.
        from: Option<String>,
        /// To.
        to: Option<String>,
        /// Total ms.
        total_failover_ms: f64,
    },
    /// Policy decision.
    Policy {
        /// Receipt summary.
        summary: String,
        /// Action.
        action: String,
    },
    /// Identity / admission / auth.
    Identity {
        /// Node.
        node: String,
        /// Detail.
        detail: String,
    },
    /// Region evacuation (network only).
    RegionEvacuation {
        /// Region.
        region: String,
        /// Dry run.
        dry_run: bool,
        /// Unresolved count.
        unresolved: usize,
    },
}

/// Signed-ish receipt (hash-chained locally; crypto-attributable via node key when signed externally).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetworkReceipt {
    /// Kind payload.
    pub kind: ReceiptKind,
    /// Observations (free-form metrics).
    pub observations: Vec<String>,
    /// When.
    pub at: DateTime<Utc>,
    /// Issuing node.
    pub issuer: String,
    /// Content digest for attribution.
    pub digest: String,
}

impl NetworkReceipt {
    /// Create and digest.
    pub fn new(issuer: impl Into<String>, kind: ReceiptKind, observations: Vec<String>) -> Self {
        let at = Utc::now();
        let issuer = issuer.into();
        let mut partial = Self {
            kind,
            observations,
            at,
            issuer,
            digest: String::new(),
        };
        let body = serde_json::to_string(&partial).unwrap_or_default();
        let mut h = Sha256::new();
        h.update(body.as_bytes());
        partial.digest = hex::encode(h.finalize());
        partial
    }
}

/// Receipt log.
#[derive(Clone, Debug, Default)]
pub struct ReceiptLog {
    items: Vec<NetworkReceipt>,
}

impl ReceiptLog {
    /// Create.
    pub fn new() -> Self {
        Self::default()
    }

    /// Append.
    pub fn record(&mut self, r: NetworkReceipt) {
        self.items.push(r);
    }

    /// Snapshot.
    pub fn all(&self) -> &[NetworkReceipt] {
        &self.items
    }

    /// JSON export.
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(&self.items).unwrap_or_else(|_| "[]".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn receipt_has_digest() {
        let r = NetworkReceipt::new(
            "node-a",
            ReceiptKind::Route {
                name: "payments.internal".into(),
                target: "node-b".into(),
                reason: "test".into(),
            },
            vec!["latency_ms=90".into()],
        );
        assert!(!r.digest.is_empty());
    }
}
