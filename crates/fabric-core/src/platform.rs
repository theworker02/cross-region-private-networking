//! Platform abstraction: FabricCore → PlatformAdapter → Render / Local.
//!
//! No persistent volume / storage-replication features (out of ownership scope).

use crate::ingress::IpAllowlist;
use crate::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Hooks platforms may provide without baking vendor terms into core logic.
pub trait PlatformHooks: Send + Sync {
    /// Platform name (`local`, `render`, `railway`, …).
    fn name(&self) -> &str;

    /// Suggested data directory (may be ephemeral on some PaaS).
    fn data_dir(&self) -> PathBuf;

    /// HTTP bind address for any control listener (`0.0.0.0:$PORT` on Render).
    fn http_bind(&self) -> String;

    /// Optional path to IP allowlist file for public fabric ingress.
    fn ip_allowlist_path(&self) -> Option<PathBuf>;

    /// Load allowlist using platform conventions.
    fn load_ip_allowlist(&self) -> Result<IpAllowlist> {
        if let Some(list) = IpAllowlist::from_env()? {
            return Ok(list);
        }
        if let Some(p) = self.ip_allowlist_path() {
            if p.exists() {
                return IpAllowlist::load_file(&p);
            }
        }
        Ok(IpAllowlist::deny_all())
    }

    /// Whether the platform filesystem is typically ephemeral.
    fn ephemeral_filesystem(&self) -> bool {
        false
    }
}

/// Object-safe adapter wrapper used by FabricCore.
pub struct PlatformAdapter {
    hooks: Box<dyn PlatformHooks>,
}

impl PlatformAdapter {
    /// Wrap hooks.
    pub fn new(hooks: Box<dyn PlatformHooks>) -> Self {
        Self { hooks }
    }

    /// Platform name.
    pub fn name(&self) -> &str {
        self.hooks.name()
    }

    /// Data dir.
    pub fn data_dir(&self) -> PathBuf {
        self.hooks.data_dir()
    }

    /// Bind.
    pub fn http_bind(&self) -> String {
        self.hooks.http_bind()
    }

    /// Allowlist.
    pub fn load_ip_allowlist(&self) -> Result<IpAllowlist> {
        self.hooks.load_ip_allowlist()
    }

    /// Ephemeral?
    pub fn ephemeral_filesystem(&self) -> bool {
        self.hooks.ephemeral_filesystem()
    }
}

/// Local development adapter.
#[derive(Clone, Debug)]
pub struct LocalAdapter {
    /// Root data path.
    pub root: PathBuf,
}

impl LocalAdapter {
    /// Create under `root`.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }
}

impl PlatformHooks for LocalAdapter {
    fn name(&self) -> &str {
        "local"
    }

    fn data_dir(&self) -> PathBuf {
        self.root.join("fabric-data")
    }

    fn http_bind(&self) -> String {
        "127.0.0.1:8787".into()
    }

    fn ip_allowlist_path(&self) -> Option<PathBuf> {
        Some(self.root.join("config").join("ip_allowlist.json"))
    }

    fn ephemeral_filesystem(&self) -> bool {
        false
    }
}

/// Render platform adapter (connectivity / bind / allowlist only — no disks/volumes).
#[derive(Clone, Debug, Default)]
pub struct RenderAdapter {
    /// Optional override data dir.
    pub data_dir_override: Option<PathBuf>,
}

/// Render-oriented metadata (V2) — populated by operators/CI, never by HTML scrape.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RenderServiceMetadata {
    /// Render region label.
    pub region: Option<String>,
    /// Service id.
    pub service_id: Option<String>,
    /// Instance id (ephemeral — not fabric canonical identity).
    pub instance_id: Option<String>,
    /// Platform discovery hostname if any.
    pub discovery_hostname: Option<String>,
    /// Environment name (production/staging).
    pub environment: Option<String>,
    /// In-region private hostname.
    pub internal_hostname: Option<String>,
}

impl RenderServiceMetadata {
    /// Load best-effort from common env keys (documented; may be unset).
    pub fn from_env() -> Self {
        Self {
            region: std::env::var("RENDER_REGION").ok().or_else(|| std::env::var("FABRIC_REGION").ok()),
            service_id: std::env::var("RENDER_SERVICE_ID").ok(),
            instance_id: std::env::var("RENDER_INSTANCE_ID").ok(),
            discovery_hostname: std::env::var("RENDER_DISCOVERY_HOSTNAME").ok(),
            environment: std::env::var("RENDER_ENVIRONMENT")
                .ok()
                .or_else(|| std::env::var("FABRIC_ENVIRONMENT").ok()),
            internal_hostname: std::env::var("RENDER_INTERNAL_HOSTNAME").ok(),
        }
    }
}

impl RenderAdapter {
    /// Default Render-shaped adapter.
    pub fn new() -> Self {
        Self::default()
    }
}

impl PlatformHooks for RenderAdapter {
    fn name(&self) -> &str {
        "render"
    }

    fn data_dir(&self) -> PathBuf {
        self.data_dir_override
            .clone()
            .unwrap_or_else(|| PathBuf::from("/var/data/fabric"))
    }

    fn http_bind(&self) -> String {
        let port = std::env::var("PORT").unwrap_or_else(|_| "10000".into());
        format!("0.0.0.0:{port}")
    }

    fn ip_allowlist_path(&self) -> Option<PathBuf> {
        Some(PathBuf::from("config/render_outbound_cidrs.json"))
    }

    fn ephemeral_filesystem(&self) -> bool {
        true
    }
}

/// Placeholder hooks for future platforms (compile-time discovery).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FuturePlatformStub {
    /// Name: railway | fly | aws | gcp | azure | k8s
    pub name: String,
}

impl PlatformHooks for FuturePlatformStub {
    fn name(&self) -> &str {
        &self.name
    }
    fn data_dir(&self) -> PathBuf {
        PathBuf::from("./fabric-data")
    }
    fn http_bind(&self) -> String {
        "0.0.0.0:8080".into()
    }
    fn ip_allowlist_path(&self) -> Option<PathBuf> {
        Some(Path::new("config").join("ip_allowlist.json"))
    }
}
