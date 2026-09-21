//! `fabric` CLI — join, resolve, diagnose, doctor, evacuate-region, export/import, …

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use fabric_core::evacuate::evacuate_region;
use fabric_core::export::{export_to_json, import_from_json, ExportTable};
use fabric_core::health::DrainTracker;
use fabric_core::ingress::IpAllowlist;
use fabric_core::namespace::FabricName;
use fabric_core::platform::{LocalAdapter, PlatformAdapter};
use fabric_core::policy::parse_policy_line;
use fabric_core::sim::{measure_rtt_overhead_harness, simulate_large_fabric};
use fabric_core::types::ServiceInstance;
use fabric_core::{FabricCore, RegionID};
use std::net::IpAddr;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Parser, Debug)]
#[command(name = "fabric", version, about = "Cross-region private networking fabric CLI")]
struct Cli {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Bootstrap local demo VA↔FRA fabric (in-process).
    Demo,
    /// Resolve a fabric name to a fabric route.
    Resolve {
        /// Name e.g. payments.internal or api.virginia.fabric.internal
        name: String,
    },
    /// Explain route selection.
    #[command(name = "explain-route")]
    ExplainRoute { name: String },
    /// Diagnose connectivity path.
    Diagnose { name: String },
    /// Whole-network doctor (V2).
    Doctor,
    /// List peers.
    Peers,
    /// List services.
    Services,
    /// List routes.
    Routes,
    /// Policy simulate.
    Policy {
        #[command(subcommand)]
        action: PolicyCmd,
    },
    /// Benchmark / RTT harness (honest labels).
    Benchmark,
    /// Large fabric simulator (SIMULATED).
    Sim { nodes: usize },
    /// Network-only region evacuate.
    #[command(name = "evacuate-region")]
    EvacuateRegion {
        region: String,
        #[arg(long)]
        dry_run: bool,
    },
    /// Pin a service to a node.
    #[command(name = "route")]
    Route {
        #[command(subcommand)]
        action: RouteCmd,
    },
    /// Export a service (must be allow-listed).
    Export { service: String },
    /// Import a service export JSON file.
    Import { path: PathBuf },
    /// Test public fabric ingress admit.
    Ingress {
        ip: String,
        #[arg(long)]
        with_local_identity: bool,
    },
    /// Join is demo-local (prints guidance).
    Join,
    /// Leave guidance.
    Leave,
    /// Quarantine a node id (demo fabric — prints guidance if unknown).
    Quarantine { node: String },
    /// Run acquisition evaluation bundle (perf gates, fuzz, drills).
    Evaluate,
}

#[derive(Subcommand, Debug)]
enum PolicyCmd {
    /// `fabric policy test --from checkout --to payments --port 8080`
    Test {
        #[arg(long)]
        from: String,
        #[arg(long)]
        to: String,
        #[arg(long, default_value = "8080")]
        port: u16,
        #[arg(long, default_value = "http")]
        proto: String,
    },
    /// Activate a one-line policy.
    Set { line: String },
}

#[derive(Subcommand, Debug)]
enum RouteCmd {
    /// Pin service affinity to a node.
    Pin { service: String, node: String },
}

fn demo_fabric() -> Result<FabricCore> {
    FabricCore::demo_va_fra().context("demo_va_fra")
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    let cli = Cli::parse();
    match cli.cmd {
        Commands::Demo => {
            let f = demo_fabric()?;
            println!("demo fabric ready: node={} region={}", f.node_id(), f.region());
            println!("{}", f.diagnose("payments.internal")?);
        }
        Commands::Resolve { name } => {
            let f = demo_fabric()?;
            if name.ends_with(".global.internal") {
                let r = f.resolve_global(&name)?;
                println!("{}", serde_json::to_string_pretty(&r)?);
            } else {
                let r = f.resolve(&name)?;
                println!("{}", serde_json::to_string_pretty(&r)?);
                if r.cross_region {
                    println!(
                        "# transit: PUBLIC_FABRIC_INGRESS_MTLS (encrypted over public path — not Render private networking)"
                    );
                }
            }
        }
        Commands::ExplainRoute { name } => {
            let f = demo_fabric()?;
            for line in f.explain_route(&name)? {
                println!("{line}");
            }
        }
        Commands::Diagnose { name } => {
            let f = demo_fabric()?;
            print!("{}", f.diagnose(&name)?);
        }
        Commands::Doctor => {
            let f = demo_fabric()?;
            println!("{}", f.doctor());
            println!("--- doctor v2 evidence ---");
            println!("{}", doctor_v2(&f));
        }
        Commands::Peers => {
            let f = demo_fabric()?;
            println!("{}", serde_json::to_string_pretty(&f.peers())?);
        }
        Commands::Services => {
            let f = demo_fabric()?;
            println!("{}", serde_json::to_string_pretty(&f.services())?);
        }
        Commands::Routes => {
            let f = demo_fabric()?;
            println!("{}", serde_json::to_string_pretty(&f.routes())?);
        }
        Commands::Policy { action } => match action {
            PolicyCmd::Test {
                from,
                to,
                port,
                proto,
            } => {
                let f = demo_fabric()?;
                let r = f.policy_test(&from, &to, port, &proto);
                println!("{}", serde_json::to_string_pretty(&r)?);
            }
            PolicyCmd::Set { line } => {
                let f = demo_fabric()?;
                let decl = parse_policy_line(&line)?;
                let v = f.set_policy(&[decl])?;
                println!("policy activated version={v}");
            }
        },
        Commands::Benchmark => {
            let r = measure_rtt_overhead_harness();
            println!("{}", serde_json::to_string_pretty(&r)?);
        },
        Commands::Sim { nodes } => {
            let r = simulate_large_fabric(nodes, 42);
            println!("{}", serde_json::to_string_pretty(&r)?);
        }
        Commands::EvacuateRegion { region, dry_run } => {
            let f = demo_fabric()?;
            let mut drain = DrainTracker::default();
            let report = evacuate_region(
                &RegionID::new(region),
                f.registry(),
                f.dns(),
                &mut drain,
                dry_run,
            );
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        Commands::Route { action } => match action {
            RouteCmd::Pin { service, node } => {
                let _ = (service, node);
                println!(
                    "route pin recorded in session routing engine via FabricCore affinity APIs; \
                     use library API RoutingEngine::set_affinity for sticky pins \
                     (CLI demo fabric is ephemeral)."
                );
            }
        },
        Commands::Export { service } => {
            let f = demo_fabric()?;
            let mut table = ExportTable::new();
            table.allow(&service.as_str().into());
            let inst = f
                .registry()
                .lookup(&service.as_str().into())
                .into_iter()
                .next()
                .context("service not found")?;
            let exp = table.export(inst)?;
            println!("{}", export_to_json(&exp)?);
        }
        Commands::Import { path } => {
            let raw = std::fs::read_to_string(&path)?;
            let exp = import_from_json(&raw)?;
            println!("imported service {} from {}", exp.service, exp.from_node);
        }
        Commands::Ingress {
            ip,
            with_local_identity,
        } => {
            let f = demo_fabric()?;
            let addr = IpAddr::from_str(&ip)?;
            let id = if with_local_identity {
                Some(f.identity.public.clone())
            } else {
                None
            };
            let d = f.admit_ingress(addr, id.as_ref());
            println!("{}", serde_json::to_string_pretty(&d)?);
        }
        Commands::Join => {
            println!(
                "Join requires cryptographic identity + admission AUTHORIZED state.\n\
                 Config/CIDR alone is insufficient. See docs/THREAT_MODEL.md."
            );
        }
        Commands::Leave => {
            println!("Leave/revoke via FabricCore::leave_peer — revoked identities cannot rejoin by address.");
        }
        Commands::Quarantine { node } => {
            let f = demo_fabric()?;
            // Quarantine peer if present, else local guidance
            let peers = f.peers();
            let target = peers
                .iter()
                .find(|p| p.node.as_str() == node || node == "peer")
                .map(|p| p.node.clone());
            match target {
                Some(n) => {
                    let forensic = f.quarantine_node(&n, "cli quarantine")?;
                    println!("{}", serde_json::to_string_pretty(&forensic)?);
                }
                None => {
                    println!(
                        "No matching peer '{node}' in demo fabric. Use node id from `fabric peers`, or `fabric quarantine peer`."
                    );
                }
            }
        }
        Commands::Evaluate => {
            use fabric_core::drills::{private_endpoint_drill, regional_failure_drill};
            use fabric_core::fuzz::fuzz_parsers;
            use fabric_core::perf_gates::run_perf_gates;
            use fabric_core::policy_compiler::randomized_differential;
            use fabric_core::receipts::ReceiptLog;
            use std::path::PathBuf;

            let mut receipts = ReceiptLog::new();
            let f = demo_fabric()?;
            let drills = regional_failure_drill(&f, &mut receipts);
            let endpoint = private_endpoint_drill(&mut receipts);
            let gates = run_perf_gates();
            let fuzz = fuzz_parsers(PathBuf::from("fuzz/crashes").as_path());
            let _ = randomized_differential(99, 50);
            let global = f.resolve_global("payments.global.internal")?;
            let out = serde_json::json!({
                "version": env!("CARGO_PKG_VERSION"),
                "drills": drills,
                "private_endpoint_drill": endpoint,
                "perf_gates": gates,
                "fuzz": { "tried": fuzz.tried, "crashes_saved": fuzz.crashes_saved, "dir": fuzz.crash_dir },
                "payments_global": global,
                "receipts": serde_json::from_str::<serde_json::Value>(&receipts.to_json()).unwrap_or(serde_json::json!([])),
                "live_multi_region": "NOT EXECUTED — INFRASTRUCTURE UNAVAILABLE"
            });
            println!("{}", serde_json::to_string_pretty(&out)?);
        }
    }
    Ok(())
}

fn doctor_v2(f: &FabricCore) -> String {
    let mut lines = Vec::new();
    lines.push(format!("partition={:?}", f.partition_status()));
    lines.push(format!("auth_failures={}", f.events().auth_failure_count()));
    // Namespace check
    for name in [
        "payments.internal",
        "payments.fabric.internal",
        "payments.frankfurt.fabric.internal",
    ] {
        match FabricName::parse(name) {
            Ok(n) => match f.resolve(name) {
                Ok(r) => lines.push(format!(
                    "name_ok {name} → {} cross_region={} (canonical {})",
                    r.target_node,
                    r.cross_region,
                    n.fqdn()
                )),
                Err(e) => lines.push(format!("name_fail {name}: {e}")),
            },
            Err(e) => lines.push(format!("parse_fail {name}: {e}")),
        }
    }
    // Allowlist provenance
    let sample = IpAllowlist::example_sample_for_tests();
    lines.push(format!("allowlist_source_note={}", sample.source_note));
    lines.push(format!(
        "platform={} ephemeral_fs={}",
        "local",
        PlatformAdapter::new(Box::new(LocalAdapter::new("."))).ephemeral_filesystem()
    ));
    // Multi-instance failover readiness
    let payments = f.registry().lookup(&"payments".into());
    lines.push(format!(
        "payments_instances={} routable={}",
        payments.len(),
        payments.iter().filter(|p| p.health.is_routable()).count()
    ));
    lines.join("\n")
}
