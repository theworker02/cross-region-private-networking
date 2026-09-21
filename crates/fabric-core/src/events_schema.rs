//! Versioned machine-readable event schema helpers + JSON Schema documents path.

use crate::observability::{EventRecord, FabricEvent};
use serde_json::{json, Value};

/// Event schema version.
pub const EVENT_SCHEMA_VERSION: &str = "1.0.0";

/// Wrap an event record with schema metadata.
pub fn envelope(record: &EventRecord) -> Value {
    json!({
        "schema_version": EVENT_SCHEMA_VERSION,
        "at": record.at.to_rfc3339(),
        "event": record.event,
    })
}

/// JSON Schema (draft-07 style subset) for FabricEvent tagged enum — for diligence.
pub fn fabric_event_json_schema() -> Value {
    json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "title": "FabricEvent",
        "version": EVENT_SCHEMA_VERSION,
        "type": "object",
        "required": ["kind"],
        "properties": {
            "kind": {
                "type": "string",
                "enum": [
                    "peer_up", "peer_down", "service_registered", "service_deregistered",
                    "route_changed", "auth_failed", "path_metrics", "dns_resolve"
                ]
            }
        }
    })
}

/// Validate minimal shape of exported JSONL line.
pub fn validate_event_json(v: &Value) -> bool {
    v.get("schema_version").and_then(|x| x.as_str()) == Some(EVENT_SCHEMA_VERSION)
        && v.get("event").is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observability::EventLog;

    #[test]
    fn schema_version_on_envelope() {
        let log = EventLog::new();
        log.emit(FabricEvent::AuthFailed {
            detail: "x".into(),
            client_ip: None,
        });
        let rec = &log.snapshot()[0];
        let env = envelope(rec);
        assert!(validate_event_json(&env));
    }
}
