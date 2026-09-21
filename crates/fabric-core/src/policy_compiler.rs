//! Policy compiler pipeline with deterministic snapshots + differential testing.

use crate::error::{FabricError, Result};
use crate::policy::{parse_policy_line, CompiledRule, PolicyEngine, PolicyRuleDecl};
use crate::types::PolicyAction;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Compiled policy snapshot (deterministic).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PolicySnapshot {
    /// Hex digest of canonical compiled rules JSON.
    pub digest: String,
    /// Compiled rules sorted by priority then name.
    pub rules: Vec<CompiledRule>,
}

/// Compile declarations → sorted snapshot.
pub fn compile_snapshot(decls: &[PolicyRuleDecl]) -> Result<PolicySnapshot> {
    let mut rules = PolicyEngine::compile(decls)?;
    rules.sort_by(|a, b| {
        a.priority
            .cmp(&b.priority)
            .then_with(|| a.name.cmp(&b.name))
    });
    let canonical = serde_json::to_string(&rules)?;
    let mut h = Sha256::new();
    h.update(canonical.as_bytes());
    Ok(PolicySnapshot {
        digest: hex::encode(h.finalize()),
        rules,
    })
}

/// Reference evaluator (independent of engine match loop) for differential tests.
pub fn reference_eval(
    rules: &[CompiledRule],
    from: &str,
    to: &str,
    port: u16,
    proto: &str,
    default_deny: bool,
) -> PolicyAction {
    let proto = proto.to_ascii_lowercase();
    for r in rules {
        let from_ok = r
            .from_service
            .as_ref()
            .map(|s| s.as_str() == from)
            .unwrap_or(true);
        let to_ok = r
            .to_service
            .as_ref()
            .map(|s| s.as_str() == to)
            .unwrap_or(true);
        let port_ok = r.port.map(|p| p == port).unwrap_or(true);
        let proto_ok = r
            .protocol
            .as_ref()
            .map(|p| p == &proto)
            .unwrap_or(true);
        if from_ok && to_ok && port_ok && proto_ok {
            return r.action;
        }
    }
    if default_deny {
        PolicyAction::Deny
    } else {
        PolicyAction::Allow
    }
}

/// Differential test: engine vs reference must agree.
pub fn differential_check(
    decls: &[PolicyRuleDecl],
    cases: &[(String, String, u16, String)],
) -> Result<()> {
    let snap = compile_snapshot(decls)?;
    let mut eng = PolicyEngine::new();
    eng.activate(decls)?;
    for (from, to, port, proto) in cases {
        let engine = eng.simulate(from, to, *port, proto, None, None).action;
        let reference = reference_eval(&snap.rules, from, to, *port, proto, true);
        if engine != reference {
            return Err(FabricError::InvalidState(format!(
                "policy differential disagreement: {from}->{to}:{port}/{proto} engine={engine:?} ref={reference:?}"
            )));
        }
    }
    Ok(())
}

/// Randomized differential suite (deterministic seed).
pub fn randomized_differential(seed: u64, iterations: usize) -> Result<()> {
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};
    let mut rng = StdRng::seed_from_u64(seed);
    let services = ["checkout", "payments", "admin", "api", "worker"];
    let mut decls = vec![
        parse_policy_line("deny from * to admin port * proto * priority 1")?,
        parse_policy_line("allow from checkout to payments port 8080 proto http priority 10")?,
    ];
    // Add a few random allows
    for i in 0..5 {
        let a = services[rng.gen_range(0..services.len())];
        let b = services[rng.gen_range(0..services.len())];
        let port = [80u16, 443, 8080, 9000][rng.gen_range(0..4)];
        decls.push(parse_policy_line(&format!(
            "allow from {a} to {b} port {port} proto http priority {}",
            20 + i
        ))?);
    }
    let mut cases = Vec::new();
    for _ in 0..iterations {
        let a = services[rng.gen_range(0..services.len())].to_string();
        let b = services[rng.gen_range(0..services.len())].to_string();
        let port = [80u16, 443, 8080, 9000][rng.gen_range(0..4)];
        cases.push((a, b, port, "http".into()));
    }
    differential_check(&decls, &cases)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshots_deterministic() {
        let decls = vec![
            parse_policy_line("allow from checkout to payments port 8080 proto http priority 10")
                .unwrap(),
            parse_policy_line("deny from * to admin port * proto * priority 1").unwrap(),
        ];
        let a = compile_snapshot(&decls).unwrap();
        let b = compile_snapshot(&decls).unwrap();
        assert_eq!(a.digest, b.digest);
    }

    #[test]
    fn differential_agrees() {
        randomized_differential(42, 200).unwrap();
    }
}
