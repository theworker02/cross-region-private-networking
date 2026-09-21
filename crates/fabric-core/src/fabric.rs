//! FabricCore — integrated control/dataplane orchestration (platform-independent).

use crate::admission::{AdmissionController, AdmissionState};
use crate::dns::{FabricRoute, PrivateDns};
use crate::error::{FabricError, Result};
use crate::global_dns::{resolve_global, GlobalResolve, NativeHostnameMap};
use crate::graph::{NetworkGraph, ProbeScheduler};
use crate::handshake::{mutual_handshake, HandshakeEngine, Session};
use crate::health::{DrainTracker, HealthManager, StandbyPath};
use crate::identity::{NodeIdentity, TrustStore};
use crate::ids::{NetworkID, NodeID, RegionID, ServiceID};
use crate::ingress::{FabricIngress, IngressTransport, IpAllowlist};
use crate::membership::{assess_partition, MemberView, MembershipConsensus, PartitionStatus};
use crate::observability::{EventLog, FabricEvent};
use crate::policy::{parse_policy_line, PolicyEngine, PolicyReceipt, PolicyRuleDecl};
use crate::platform::{LocalAdapter, PlatformAdapter};
use crate::quarantine::{QuarantineBoard, QuarantineForensics};
use crate::receipts::ReceiptLog;
use crate::registry::ServiceRegistry;
use crate::relay::{resolve_restricted_path, RelayRegistry};
use crate::routing::RoutingEngine;
use crate::types::{
    FailoverMetrics, HealthState, ServiceInstance, TopologyMode, TransportMode, PROTOCOL_VERSION,
};
use chrono::Duration;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;

/// Integrated fabric node runtime.
pub struct FabricCore {
    /// Local identity.
    pub identity: NodeIdentity,
    /// Trust store.
    trust: RwLock<TrustStore>,
    /// Admission.
    admission: RwLock<AdmissionController>,
    /// Registry.
    registry: ServiceRegistry,
    /// DNS.
    dns: Arc<PrivateDns>,
    /// Router.
    router: RwLock<RoutingEngine>,
    /// Health.
    health: HealthManager,
    /// Policy.
    policy: RwLock<PolicyEngine>,
    /// Ingress gate.
    ingress: RwLock<FabricIngress>,
    /// Relays.
    relays: RwLock<RelayRegistry>,
    /// Sessions by peer.
    sessions: RwLock<HashMap<String, Session>>,
    /// Events.
    events: EventLog,
    /// Platform.
    platform: PlatformAdapter,
    /// Graph.
    graph: RwLock<NetworkGraph>,
    /// Probes.
    probes: ProbeScheduler,
    /// Membership.
    membership: RwLock<MembershipConsensus>,
    /// Drain.
    drain: RwLock<DrainTracker>,
    /// Standby paths.
    standbys: RwLock<HashMap<String, StandbyPath>>,
    /// Handshake engine.
    handshake: RwLock<HandshakeEngine>,
    /// Topology mode.
    topology_mode: RwLock<TopologyMode>,
    /// Render private hostnames for LOCAL_NATIVE bypass.
    native_hostnames: RwLock<NativeHostnameMap>,
    /// Quarantine board + forensics.
    quarantine: RwLock<QuarantineBoard>,
    /// Network receipts.
    receipts: RwLock<ReceiptLog>,
}

impl FabricCore {
    /// Bootstrap a new local fabric node.
    pub fn bootstrap(
        network: NetworkID,
        region: RegionID,
        platform: PlatformAdapter,
        allowlist: IpAllowlist,
    ) -> Result<Self> {
        let identity = NodeIdentity::generate(network.clone(), region.clone(), Duration::hours(24));
        let mut trust = TrustStore::new();
        trust.admit(identity.public.clone())?;
        let mut admission = AdmissionController::new();
        admission.request_join(identity.public.clone());
        admission.authorize(&identity.public.node_id, "local bootstrap")?;
        admission.mark_active(&identity.public.node_id)?;

        let mut router = RoutingEngine::default();
        router.preferred_region = Some(region.clone());

        let core = Self {
            identity,
            trust: RwLock::new(trust),
            admission: RwLock::new(admission),
            registry: ServiceRegistry::new(),
            dns: Arc::new(PrivateDns::new(Duration::seconds(30))),
            router: RwLock::new(router),
            health: HealthManager::new(),
            policy: RwLock::new(PolicyEngine::new()),
            ingress: RwLock::new(FabricIngress::new(allowlist, true)),
            relays: RwLock::new(RelayRegistry::new()),
            sessions: RwLock::new(HashMap::new()),
            events: EventLog::new(),
            platform,
            graph: RwLock::new(NetworkGraph::new()),
            probes: ProbeScheduler::new(),
            membership: RwLock::new(MembershipConsensus::new()),
            drain: RwLock::new(DrainTracker::default()),
            standbys: RwLock::new(HashMap::new()),
            handshake: RwLock::new(HandshakeEngine::new()),
            topology_mode: RwLock::new(TopologyMode::Dynamic),
            native_hostnames: RwLock::new(NativeHostnameMap::new()),
            quarantine: RwLock::new(QuarantineBoard::new()),
            receipts: RwLock::new(ReceiptLog::new()),
        };
        Ok(core)
    }

    /// Demo fabric: Virginia node with Frankfurt-only `payments` (cross-region proof).
    pub fn demo_va_fra() -> Result<Self> {
        let platform = PlatformAdapter::new(Box::new(LocalAdapter::new(".")));
        let allowlist = IpAllowlist::example_sample_for_tests();
        let mut core = Self::bootstrap(
            NetworkID::new("prod"),
            RegionID::new("virginia"),
            platform,
            allowlist,
        )?;

        // Peer FRA identity admitted + authorized.
        let fra = NodeIdentity::generate(
            NetworkID::new("prod"),
            RegionID::new("frankfurt"),
            Duration::hours(24),
        );
        core.trust.write().admit(fra.public.clone())?;
        {
            let mut adm = core.admission.write();
            adm.request_join(fra.public.clone());
            adm.authorize(&fra.public.node_id, "demo peer")?;
        }

        // Mutual handshake VA ↔ FRA over simulated public ingress path.
        let (sess_a, sess_b) = mutual_handshake(
            &core.identity,
            vec!["checkout".into()],
            &fra,
            vec!["payments".into()],
            &core.trust.read(),
        )?;
        // Mark transport as public fabric ingress (cross-region over Internet).
        let mut s = sess_a;
        s.peer.transport = TransportMode::Relayed; // ingress is not DIRECT private net
        s.peer.rtt_ms = 90.0;
        core.sessions
            .write()
            .insert(s.peer.node.to_string(), s);
        let _ = sess_b;
        core.admission.write().mark_active(&fra.public.node_id)?;

        // Frankfurt-only payments.
        let mut payments = ServiceInstance::new(
            "payments",
            fra.public.node_id.as_str(),
            "frankfurt",
            "prod",
            8080,
        );
        payments.latency_ms = 90.0;
        payments.protocol = "http".into();
        core.registry.register(payments);

        // Local checkout (VA).
        let mut checkout = ServiceInstance::new(
            "checkout",
            core.identity.public.node_id.as_str(),
            "virginia",
            "prod",
            9000,
        );
        checkout.latency_ms = 2.0;
        core.registry.register(checkout);

        // Policy: checkout → payments.
        core.policy.write().activate(&[parse_policy_line(
            "allow from checkout to payments port 8080 proto http priority 10",
        )?])?;

        core.events.emit(FabricEvent::PeerUp {
            node: fra.public.node_id.to_string(),
            transport: IngressTransport::PublicFabricIngress.label().into(),
            rtt_ms: Some(90.0),
        });
        core.events.emit(FabricEvent::ServiceRegistered {
            service: "payments".into(),
            node: fra.public.node_id.to_string(),
            region: "frankfurt".into(),
        });

        // Precompute standby (none second instance — standby = self note).
        core.standbys.write().insert(
            "payments".into(),
            StandbyPath {
                primary: fra.public.node_id.clone(),
                standby: fra.public.node_id.clone(),
                service: ServiceID::new("payments"),
                computed_at: chrono::Utc::now(),
            },
        );

        core.graph.write().upsert_vertex(
            core.identity.public.node_id.as_str(),
            crate::graph::VertexKind::Node {
                region: RegionID::new("virginia"),
            },
        );
        core.graph.write().record_probe(
            &core.identity.public.node_id,
            &fra.public.node_id,
            90.0,
            4.0,
            0.001,
            200.0,
        );

        Ok(core)
    }

    /// Registry accessor.
    pub fn registry(&self) -> &ServiceRegistry {
        &self.registry
    }

    /// DNS accessor.
    pub fn dns(&self) -> &PrivateDns {
        &self.dns
    }

    /// Events.
    pub fn events(&self) -> &EventLog {
        &self.events
    }

    /// Local node id.
    pub fn node_id(&self) -> &NodeID {
        &self.identity.public.node_id
    }

    /// Local region.
    pub fn region(&self) -> &RegionID {
        &self.identity.public.region
    }

    /// Join peer by admitting identity + handshake (requires prior authorization).
    pub fn join_peer(&self, peer: &NodeIdentity, peer_services: Vec<ServiceID>) -> Result<Session> {
        if !self.admission.read().may_handshake(&peer.public.node_id) {
            // Auto-admit only if already in trust from operator; else require admission.
            return Err(FabricError::AuthFailed(
                "peer not AUTHORIZED — config alone is insufficient; complete admission".into(),
            ));
        }
        self.trust.write().admit(peer.public.clone())?;
        let (local_sess, _) = mutual_handshake(
            &self.identity,
            vec![],
            peer,
            peer_services,
            &self.trust.read(),
        )?;
        self.admission.write().mark_active(&peer.public.node_id)?;
        self.sessions
            .write()
            .insert(local_sess.peer.node.to_string(), local_sess.clone());
        self.events.emit(FabricEvent::PeerUp {
            node: local_sess.peer.node.to_string(),
            transport: local_sess.peer.transport.as_str().into(),
            rtt_ms: Some(local_sess.peer.rtt_ms),
        });
        Ok(local_sess)
    }

    /// Leave / revoke peer.
    pub fn leave_peer(&self, node: &NodeID) -> Result<()> {
        self.sessions.write().remove(node.as_str());
        self.trust.write().revoke(node);
        self.admission
            .write()
            .revoke(node, "leave/revoke")?;
        self.registry.deregister_node(node);
        self.dns.invalidate_node(node);
        self.events.emit(FabricEvent::PeerDown {
            node: node.to_string(),
            reason: "leave/revoke".into(),
        });
        // Membership: bump revocation epoch.
        let mut m = self.membership.write();
        let epoch = m
            .get(node)
            .map(|v| v.revocation_epoch + 1)
            .unwrap_or(1);
        m.observe(MemberView {
            node: node.clone(),
            admission: AdmissionState::Revoked,
            policy_version: self.policy.read().version,
            registry_version: self.registry.version(),
            revocation_epoch: epoch,
        });
        Ok(())
    }

    /// List peers.
    pub fn peers(&self) -> Vec<crate::types::PeerInfo> {
        self.sessions
            .read()
            .values()
            .map(|s| s.peer.clone())
            .collect()
    }

    /// Register service.
    pub fn register_service(&self, instance: ServiceInstance) -> u64 {
        let v = self.registry.register(instance.clone());
        self.dns.invalidate_service(&instance.service);
        self.events.emit(FabricEvent::ServiceRegistered {
            service: instance.service.to_string(),
            node: instance.node.to_string(),
            region: instance.region.to_string(),
        });
        v
    }

    /// Resolve fabric hostname to fabric route.
    pub fn resolve(&self, name: &str) -> Result<FabricRoute> {
        let mut route = self.dns.resolve(
            name,
            &self.registry,
            &self.router.read(),
            self.node_id(),
            self.region(),
        )?;
        // Refine transport from peer / connectivity.
        if route.cross_region {
            let peer_direct = self
                .sessions
                .read()
                .get(route.target_node.as_str())
                .map(|s| s.peer.transport == TransportMode::Direct)
                .unwrap_or(false);
            let path = resolve_restricted_path(
                peer_direct,
                true, // public fabric ingress available in demo
                None,
                &self.relays.read(),
            )?;
            route.transport = path.mode;
            route.reason = format!("{}; {}", route.reason, path.reason);
        } else {
            route.transport = TransportMode::Direct;
        }
        self.events.emit(FabricEvent::DnsResolve {
            name: name.into(),
            target: route.target_node.to_string(),
            cross_region: route.cross_region,
        });
        Ok(route)
    }

    /// Authorize then resolve (for diagnose).
    pub fn resolve_authorized(
        &self,
        from: &ServiceID,
        name: &str,
    ) -> Result<(FabricRoute, PolicyReceipt)> {
        let route = self.resolve(name)?;
        let receipt = self.policy.read().authorize(
            from,
            &route.service,
            route.port,
            &route.protocol,
        )?;
        Ok((route, receipt))
    }

    /// Explain route for a name.
    pub fn explain_route(&self, name: &str) -> Result<Vec<String>> {
        let route = self.resolve(name)?;
        let candidates = self.registry.lookup(&route.service);
        let decision = self.router.read().choose(
            &route.service,
            &candidates,
            self.region(),
            None,
        )?;
        let mut lines = RoutingEngine::explain(&decision);
        lines.insert(
            0,
            format!(
                "name={} cross_region={} transport={} via_local={}",
                route.name,
                route.cross_region,
                // Honest label for ingress
                if route.cross_region {
                    IngressTransport::PublicFabricIngress.label()
                } else {
                    route.transport.as_str()
                },
                route.via_local_node
            ),
        );
        lines.push(format!("policy_version={}", self.policy.read().version));
        lines.push(format!("protocol_version={PROTOCOL_VERSION}"));
        Ok(lines)
    }

    /// Diagnose full connectivity path for a name.
    pub fn diagnose(&self, name: &str) -> Result<String> {
        let mut out = String::new();
        out.push_str(&format!("fabric diagnose {name}\n"));
        out.push_str(&format!(
            "local_node={} region={} network={} platform={}\n",
            self.node_id(),
            self.region(),
            self.identity.public.network,
            self.platform.name()
        ));
        match self.resolve(name) {
            Ok(route) => {
                out.push_str(&format!(
                    "resolve: OK → node={} region={} port={} cross_region={}\n",
                    route.target_node, route.target_region, route.port, route.cross_region
                ));
                out.push_str(&format!(
                    "transport: {} (encrypted authenticated transit; cross-region is NOT Render private networking)\n",
                    if route.cross_region {
                        IngressTransport::PublicFabricIngress.label()
                    } else {
                        "DIRECT"
                    }
                ));
                out.push_str(&format!("reason: {}\n", route.reason));
                out.push_str(&format!("health: {:?}\n", route.health));
                out.push_str(&format!("latency_hint_ms: {:.1}\n", route.latency_ms));
                if let Ok(pol) = self.policy.read().authorize(
                    &ServiceID::new("checkout"),
                    &route.service,
                    route.port,
                    &route.protocol,
                ) {
                    out.push_str(&format!(
                        "policy(checkout→{}): {:?} via {}\n",
                        route.service, pol.action, pol.matched_rule
                    ));
                } else if let Err(e) = self.policy.read().authorize(
                    &ServiceID::new("checkout"),
                    &route.service,
                    route.port,
                    &route.protocol,
                ) {
                    out.push_str(&format!("policy(checkout→{}): {e}\n", route.service));
                }
                let peers = self.peers();
                out.push_str(&format!("peers_up: {}\n", peers.len()));
                for p in peers {
                    out.push_str(&format!(
                        "  peer {} region={} transport={} rtt_ms={:.1} health={:?}\n",
                        p.node, p.region, p.transport.as_str(), p.rtt_ms, p.health
                    ));
                }
                let part = self.partition_status();
                out.push_str(&format!("partition_status: {part:?}\n"));
            }
            Err(e) => {
                out.push_str(&format!("resolve: FAIL — {e}\n"));
                out.push_str(
                    "hint: before fabric, cross-region private IPs are unreachable; \
                     after fabric, names resolve to fabric routes via mTLS ingress.\n",
                );
            }
        }
        Ok(out)
    }

    /// Doctor: whole-network diagnosis from observed state.
    pub fn doctor(&self) -> String {
        let mut lines = Vec::new();
        lines.push(format!("fabric doctor ({})", self.platform.name()));
        lines.push(format!(
            "identity: {} gen={} expires={}",
            self.node_id(),
            self.identity.public.generation,
            self.identity.public.not_after
        ));
        let peers = self.peers();
        lines.push(format!("peers: {} active sessions", peers.len()));
        let services = self.registry.all();
        lines.push(format!("services: {} instances", services.len()));
        let unhealthy: Vec<_> = services
            .iter()
            .filter(|s| !s.health.is_routable())
            .collect();
        if unhealthy.is_empty() {
            lines.push("health: all registered instances routable".into());
        } else {
            lines.push(format!("health: {} non-routable instances", unhealthy.len()));
            for u in unhealthy {
                lines.push(format!(
                    "  - {} @ {} {:?}",
                    u.service, u.node, u.health
                ));
            }
        }
        lines.push(format!("partition: {:?}", self.partition_status()));
        lines.push(format!(
            "auth_failures: {}",
            self.events.auth_failure_count()
        ));
        lines.push(format!(
            "ingress_enabled: {} allowlist_cidrs: {} ({})",
            self.ingress.read().enabled,
            self.ingress.read().allowlist.cidrs.len(),
            self.ingress.read().allowlist.source_note
        ));
        lines.push(format!("policy_version: {}", self.policy.read().version));
        // payments.internal specifically
        match self.resolve("payments.internal") {
            Ok(r) => lines.push(format!(
                "payments.internal: OK → {} ({}) cross_region={}",
                r.target_node, r.target_region, r.cross_region
            )),
            Err(e) => lines.push(format!("payments.internal: FAIL — {e}")),
        }
        // Graph observed edges
        let observed = self
            .graph
            .read()
            .edges()
            .filter(|e| e.source == crate::graph::MetricSource::Observed)
            .count();
        lines.push(format!("graph_edges_observed: {observed}"));
        lines.join("\n")
    }

    /// Partition status.
    pub fn partition_status(&self) -> PartitionStatus {
        let expected = self.sessions.read().len();
        let reachable = self
            .sessions
            .read()
            .values()
            .filter(|s| s.peer.health.is_routable())
            .count();
        let degraded = self
            .sessions
            .read()
            .values()
            .filter(|s| s.peer.health == HealthState::Degraded)
            .count();
        assess_partition(expected.max(1), reachable, degraded)
    }

    /// Policy test simulator.
    pub fn policy_test(
        &self,
        from: &str,
        to: &str,
        port: u16,
        protocol: &str,
    ) -> PolicyReceipt {
        self.policy
            .read()
            .simulate(from, to, port, protocol, None, None)
    }

    /// Activate policy declarations.
    pub fn set_policy(&self, decls: &[PolicyRuleDecl]) -> Result<u64> {
        self.policy.write().activate(decls)
    }

    /// Public ingress admit.
    pub fn admit_ingress(
        &self,
        client_ip: IpAddr,
        client_identity: Option<&crate::identity::PublicIdentity>,
    ) -> crate::ingress::IngressDecision {
        let d = self
            .ingress
            .read()
            .admit(client_ip, client_identity, &self.trust.read());
        if !d.allowed {
            self.events.emit(FabricEvent::AuthFailed {
                detail: d.reason.clone(),
                client_ip: Some(d.client_ip.clone()),
            });
        }
        d
    }

    /// Failover with metrics.
    pub fn failover_service(&self, service: &ServiceID) -> Result<FailoverMetrics> {
        let current = self
            .registry
            .lookup(service)
            .into_iter()
            .find(|i| i.health.is_routable());
        let from = current.as_ref().map(|c| c.node.clone());
        let metrics = self.health.measure_failover(
            from.clone(),
            || {
                if let Some(c) = &current {
                    self.registry.update_health(
                        service,
                        &c.node,
                        HealthState::Unreachable,
                    );
                    self.dns.on_health_failure(service, &c.node);
                    self.router.write().clear_active(service);
                }
            },
            || {
                let candidates = self.registry.lookup(service);
                self.router
                    .read()
                    .choose(service, &candidates, self.region(), None)
                    .ok()
                    .map(|d| d.instance.node)
            },
            || {},
            "health failure failover",
        );
        if let Some(to) = &metrics.to_node {
            self.events.emit(FabricEvent::RouteChanged {
                service: service.to_string(),
                from: from.map(|n| n.to_string()),
                to: to.to_string(),
                reason: metrics.reason.clone(),
            });
        }
        Ok(metrics)
    }

    /// Promote standby path.
    pub fn promote_standby(&self, service: &ServiceID) -> Result<FailoverMetrics> {
        let sb = self
            .standbys
            .read()
            .get(service.as_str())
            .cloned()
            .ok_or_else(|| FabricError::NoRoute("no standby".into()))?;
        Ok(self.health.measure_failover(
            Some(sb.primary.clone()),
            || {},
            || Some(sb.standby.clone()),
            || {
                let mut d = self.drain.write();
                d.begin_drain(sb.primary.as_str(), 1);
            },
            "standby promotion",
        ))
    }

    /// List services.
    pub fn services(&self) -> Vec<ServiceInstance> {
        self.registry.all()
    }

    /// Routes snapshot via resolve of all service FQDNs.
    pub fn routes(&self) -> Vec<FabricRoute> {
        let mut seen = std::collections::HashSet::new();
        let mut out = Vec::new();
        for s in self.registry.all() {
            let name = s.service.fqdn();
            if seen.insert(name.clone()) {
                if let Ok(r) = self.resolve(&name) {
                    out.push(r);
                }
            }
        }
        out
    }

    /// Rotate local identity keys with overlap (old remains in trust until expiry handling).
    pub fn rotate_identity(&self, ttl: Duration) -> Result<()> {
        // Keep old key fingerprint trusted until natural expiry path — for demo we keep node id.
        let mut id = self.identity.clone();
        let old = id.rotate(ttl);
        // Re-admit new public material under same node id.
        self.trust.write().admit(id.public.clone())?;
        // Old fingerprint remains known until revoked explicitly.
        let _ = old;
        // Note: FabricCore holds identity by value — rotation needs interior mutability.
        // For this build we expose rotate via replacing through unsafe pattern avoided:
        Err(FabricError::InvalidState(
            "use FabricCore::rotate_identity_inplace in mutable context".into(),
        ))
    }

    /// Rotate when holding `&mut self`.
    pub fn rotate_identity_inplace(&mut self, ttl: Duration) -> Result<()> {
        let old = self.identity.rotate(ttl);
        self.trust.write().admit(self.identity.public.clone())?;
        let _ = old;
        Ok(())
    }

    /// Trust store access for tests.
    pub fn trust_store(&self) -> parking_lot::RwLockReadGuard<'_, TrustStore> {
        self.trust.read()
    }

    /// Register a Render private hostname for LOCAL_NATIVE bypass.
    pub fn set_native_hostname(&self, service: &ServiceID, region: &RegionID, hostname: &str) {
        self.native_hostnames
            .write()
            .set(service, region, hostname);
    }

    /// Resolve `*.global.internal` with local-native preference.
    pub fn resolve_global(&self, name: &str) -> Result<GlobalResolve> {
        resolve_global(
            name,
            self.node_id(),
            self.region(),
            &self.native_hostnames.read(),
            &self.registry,
            &self.dns,
            &self.router.read(),
            IngressTransport::PublicFabricIngress,
        )
    }

    /// Quarantine a node (revoke, drop routes, keep forensics).
    pub fn quarantine_node(&self, node: &NodeID, reason: &str) -> Result<QuarantineForensics> {
        let mut trust = self.trust.write();
        let mut admission = self.admission.write();
        let mut membership = self.membership.write();
        let mut receipts = self.receipts.write();
        let mut board = self.quarantine.write();
        board.quarantine(
            node,
            reason,
            &self.registry,
            &self.dns,
            &mut trust,
            &mut admission,
            &mut membership,
            &self.events,
            &mut receipts,
            &mut |n| {
                self.sessions.write().remove(n.as_str());
            },
        )
    }

    /// Receipt log snapshot JSON.
    pub fn receipts_json(&self) -> String {
        self.receipts.read().to_json()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn payments_internal_va_to_frankfurt() {
        let fabric = FabricCore::demo_va_fra().unwrap();
        let route = fabric.resolve("payments.internal").unwrap();
        assert!(route.cross_region);
        assert_eq!(route.target_region.as_str(), "frankfurt");
        assert_eq!(route.service.as_str(), "payments");
        // Must be fabric route, not public app hostname
        assert!(!route.target_node.as_str().is_empty());
        // Global namespace form
        let g = fabric.resolve("payments.fabric.internal").unwrap();
        assert_eq!(g.target_region.as_str(), "frankfurt");
        let rq = fabric
            .resolve("payments.frankfurt.fabric.internal")
            .unwrap();
        assert_eq!(rq.target_region.as_str(), "frankfurt");
        let (r, pol) = fabric
            .resolve_authorized(&"checkout".into(), "payments.internal")
            .unwrap();
        assert_eq!(r.port, 8080);
        assert_eq!(pol.action, crate::types::PolicyAction::Allow);
    }

    #[test]
    fn before_fabric_no_registration_fails() {
        let platform = PlatformAdapter::new(Box::new(LocalAdapter::new(".")));
        let core = FabricCore::bootstrap(
            "prod".into(),
            "virginia".into(),
            platform,
            IpAllowlist::example_sample_for_tests(),
        )
        .unwrap();
        assert!(core.resolve("payments.internal").is_err());
    }

    #[test]
    fn ingress_rejects_bad_ip() {
        let fabric = FabricCore::demo_va_fra().unwrap();
        let id = fabric.identity.public.clone();
        let d = fabric.admit_ingress(
            IpAddr::V4(Ipv4Addr::new(198, 51, 100, 1)),
            Some(&id),
        );
        assert!(!d.allowed);
    }

    #[test]
    fn global_name_cross_region() {
        let fabric = FabricCore::demo_va_fra().unwrap();
        let g = fabric.resolve_global("payments.global.internal").unwrap();
        assert_eq!(
            g.path_class,
            crate::bypass::PathClass::FabricRelayed
        );
    }

    #[test]
    fn global_name_local_native() {
        let fabric = FabricCore::demo_va_fra().unwrap();
        fabric.set_native_hostname(
            &"checkout".into(),
            &"virginia".into(),
            "checkout-native:9000",
        );
        let g = fabric.resolve_global("checkout.global.internal").unwrap();
        assert_eq!(g.path_class, crate::bypass::PathClass::LocalNative);
    }
}
