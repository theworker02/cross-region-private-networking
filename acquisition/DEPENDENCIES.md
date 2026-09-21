# Dependencies (summary)

Primary direct dependencies (see `Cargo.toml` / lockfile):

- serde / serde_json  
- ed25519-dalek, x25519-dalek, sha2, hkdf, hex  
- chrono, uuid, parking_lot, thiserror, anyhow  
- clap, tracing, tracing-subscriber (CLI)  
- rand  

Operators should run `cargo tree` and license review before distribution.
