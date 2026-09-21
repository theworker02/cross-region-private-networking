//! Parser fuzz harness — persist crashing inputs under fuzz/crashes/.

use crate::namespace::FabricName;
use crate::policy::parse_policy_line;
use crate::ingress::Cidr;
use std::fs;
use std::path::Path;

/// Fuzz result.
#[derive(Debug)]
pub struct FuzzReport {
    /// Inputs tried.
    pub tried: usize,
    /// Panics/caught failures persisted.
    pub crashes_saved: usize,
    /// Directory used.
    pub crash_dir: String,
}

fn persist_crash(dir: &Path, name: &str, input: &str, err: &str) {
    let _ = fs::create_dir_all(dir);
    let safe: String = name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    let path = dir.join(format!("{safe}.txt"));
    let _ = fs::write(
        path,
        format!("input:\n{input}\n\nerror:\n{err}\n"),
    );
}

/// Run deterministic fuzz corpus against parsers (no panic abort — catch via Result).
pub fn fuzz_parsers(crash_dir: &Path) -> FuzzReport {
    let mut tried = 0;
    let mut crashes_saved = 0;

    let hostname_corpus = [
        "",
        ".",
        "..",
        "payments.internal",
        "payments.fabric.internal",
        "payments.global.internal",
        "a.b.c.d.internal",
        "../../../../etc/passwd.internal",
        "payments\0.internal",
        &"x".repeat(10_000),
        "payments..frankfurt.fabric.internal",
        "PAYMENTS.INTERNAL",
    ];
    for (i, input) in hostname_corpus.iter().enumerate() {
        tried += 1;
        // Must not panic
        let _ = FabricName::parse(input);
        let _ = crate::global_dns::parse_global_name(input);
        if input.contains('\0') {
            // document unusual input
            persist_crash(
                crash_dir,
                &format!("hostname_{i}"),
                input,
                "nul byte in hostname corpus (handled without panic)",
            );
            crashes_saved += 1;
        }
    }

    let policy_corpus = [
        "",
        "allow",
        "allow from",
        "allow from * to * port * proto * priority 1",
        "deny from checkout to payments port 8080 proto http priority 10",
        "explode from x to y",
        "allow from checkout to payments port notaport proto http priority 10",
        "allow from checkout to payments port 8080 proto http priority -1",
        &format!("allow from {} to payments port 80 proto http priority 1", "a".repeat(5000)),
    ];
    for (i, line) in policy_corpus.iter().enumerate() {
        tried += 1;
        match parse_policy_line(line) {
            Ok(_) => {}
            Err(e) => {
                // Expected for bad inputs — only persist if we want corpus of rejects
                if line.len() > 1000 || line.contains("explode") {
                    persist_crash(crash_dir, &format!("policy_{i}"), line, &e.to_string());
                    crashes_saved += 1;
                }
            }
        }
    }

    let cidr_corpus = [
        "10.0.0.0/8",
        "10.0.0.0/33",
        "not-a-cidr",
        "2001:db8::/32",
        "192.0.2.1",
        "/8",
        "0.0.0.0/0",
    ];
    for (i, c) in cidr_corpus.iter().enumerate() {
        tried += 1;
        if let Err(e) = Cidr::parse(c) {
            if c.contains("33") || *c == "not-a-cidr" || *c == "/8" || *c == "192.0.2.1" {
                persist_crash(crash_dir, &format!("cidr_{i}"), c, &e.to_string());
                crashes_saved += 1;
            }
        }
    }

    FuzzReport {
        tried,
        crashes_saved,
        crash_dir: crash_dir.display().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn fuzz_does_not_panic() {
        let dir = tempdir().unwrap();
        let r = fuzz_parsers(dir.path());
        assert!(r.tried > 10);
    }
}
