//! Regional failure drills + private endpoint drills with receipts.

use crate::connector::{ExternalNetworkConnector, LocalMockConnector, PrivateLinkComplementPlan};
use crate::fabric::FabricCore;
use crate::receipts::{NetworkReceipt, ReceiptKind, ReceiptLog};
use crate::sim::scenarios;
use crate::types::HealthState;
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Drill measurement.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DrillMetrics {
    /// Drill name.
    pub name: String,
    /// Detection ms.
    pub detection_ms: f64,
    /// Recompute ms.
    pub recompute_ms: f64,
    /// Total ms.
    pub total_ms: f64,
    /// Label.
    pub label: String,
    /// Notes.
    pub notes: String,
}

/// VA / FRA / SIN failure drill (SIN simulated as third registry region if present).
pub fn regional_failure_drill(fabric: &FabricCore, receipts: &mut ReceiptLog) -> Vec<DrillMetrics> {
    let mut out = Vec::new();
    for region in ["frankfurt", "virginia", "singapore"] {
        let t0 = Instant::now();
        // detection
        let before = fabric.registry().all().len();
        scenarios::region_disappear(fabric, &region.into());
        let detect = t0.elapsed().as_secs_f64() * 1000.0;
        let t1 = Instant::now();
        let _ = fabric.resolve("payments.internal");
        let recompute = t1.elapsed().as_secs_f64() * 1000.0;
        let total = t0.elapsed().as_secs_f64() * 1000.0;
        receipts.record(NetworkReceipt::new(
            fabric.node_id().as_str(),
            ReceiptKind::Failover {
                service: "payments".into(),
                from: Some(region.into()),
                to: None,
                total_failover_ms: total,
            },
            vec![
                format!("detection_ms={detect:.3}"),
                format!("recompute_ms={recompute:.3}"),
                format!("instances_before={before}"),
                "label=SIMULATED_DRILL".into(),
            ],
        ));
        out.push(DrillMetrics {
            name: format!("region_disappear_{region}"),
            detection_ms: detect,
            recompute_ms: recompute,
            total_ms: total,
            label: "SIMULATED".into(),
            notes: "In-process failure injection; not live Render multi-region".into(),
        });
        // restore nothing — drill is destructive to demo fabric copy; caller uses fresh fabric
        let _ = HealthState::Healthy;
    }
    out
}

/// Private endpoint drill: A → fabric → B → gateway → mock resource.
pub fn private_endpoint_drill(receipts: &mut ReceiptLog) -> DrillMetrics {
    let t0 = Instant::now();
    let plan = PrivateLinkComplementPlan::new("virginia", "frankfurt", "aurora");
    let connector = LocalMockConnector::new().with_mock_resource(
        "aurora",
        "vpce-docs-example.amazonaws.com",
        5432,
    );
    let detect = t0.elapsed().as_secs_f64() * 1000.0;
    let t1 = Instant::now();
    let conn = connector.connect("aurora").expect("mock connect");
    let recompute = t1.elapsed().as_secs_f64() * 1000.0;
    let total = t0.elapsed().as_secs_f64() * 1000.0;
    receipts.record(NetworkReceipt::new(
        "drill",
        ReceiptKind::Route {
            name: "aurora.external".into(),
            target: plan.gateway_region.clone(),
            reason: conn.clone(),
        },
        vec![plan.underlay_note.clone(), format!("path={}", conn)],
    ));
    DrillMetrics {
        name: "privatelink_complement_mock".into(),
        detection_ms: detect,
        recompute_ms: recompute,
        total_ms: total,
        label: "SIMULATED".into(),
        notes: plan.underlay_note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoint_drill_runs() {
        let mut r = ReceiptLog::new();
        let m = private_endpoint_drill(&mut r);
        assert!(m.total_ms >= 0.0);
        assert!(!r.all().is_empty());
    }

    #[test]
    fn regional_drill_runs() {
        let f = FabricCore::demo_va_fra().unwrap();
        let mut r = ReceiptLog::new();
        let m = regional_failure_drill(&f, &mut r);
        assert_eq!(m.len(), 3);
    }
}
