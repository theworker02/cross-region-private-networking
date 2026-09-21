//! Performance regression gates with tolerances.

use crate::fabric::FabricCore;
use crate::policy_compiler::randomized_differential;
use crate::sim::{measure_rtt_overhead_harness, simulate_large_fabric};
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Gate definition.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PerfGate {
    /// Name.
    pub name: String,
    /// Measured value.
    pub measured: f64,
    /// Max allowed.
    pub max_allowed: f64,
    /// Unit.
    pub unit: String,
    /// Pass?
    pub pass: bool,
}

/// Run gates; returns all results.
pub fn run_perf_gates() -> Vec<PerfGate> {
    let mut gates = Vec::new();

    let t0 = Instant::now();
    let f = FabricCore::demo_va_fra().unwrap();
    for _ in 0..100 {
        let _ = f.resolve("payments.internal");
    }
    let resolve_ms = t0.elapsed().as_secs_f64() * 1000.0 / 100.0;
    gates.push(PerfGate {
        name: "dns_resolve_mean_ms".into(),
        measured: resolve_ms,
        max_allowed: 5.0, // generous for debug builds
        unit: "ms".into(),
        pass: resolve_ms <= 5.0,
    });

    let t1 = Instant::now();
    let _ = randomized_differential(7, 100);
    let diff_ms = t1.elapsed().as_secs_f64() * 1000.0;
    gates.push(PerfGate {
        name: "policy_differential_100_ms".into(),
        measured: diff_ms,
        max_allowed: 2000.0,
        unit: "ms".into(),
        pass: diff_ms <= 2000.0,
    });

    let sim = simulate_large_fabric(500, 1);
    gates.push(PerfGate {
        name: "large_fabric_500_build_ms".into(),
        measured: sim.build_ms,
        max_allowed: 500.0,
        unit: "ms".into(),
        pass: sim.build_ms <= 500.0,
    });

    let rtt = measure_rtt_overhead_harness();
    gates.push(PerfGate {
        name: "simulated_overhead_ms_non_negative".into(),
        measured: rtt.overhead_ms,
        max_allowed: 10_000.0,
        unit: "ms".into(),
        pass: rtt.overhead_ms >= 0.0 && rtt.overhead_ms < 10_000.0,
    });

    gates
}

/// True if all gates pass.
pub fn all_perf_gates_pass() -> bool {
    run_perf_gates().iter().all(|g| g.pass)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perf_gates_pass_locally() {
        assert!(all_perf_gates_pass());
    }
}
