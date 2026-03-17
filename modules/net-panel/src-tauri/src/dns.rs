//! DNS configuration and lookup tools.

use serde::Serialize;

/// Validate a hostname/IP: only safe characters.
fn validate_host(host: &str) -> Result<(), String> {
    if host.is_empty() || host.len() > 253 {
        return Err("Invalid host length".to_string());
    }
    if host.chars().all(|c| c.is_alphanumeric() || c == '.' || c == ':' || c == '-') {
        Ok(())
    } else {
        Err(format!("Invalid characters in host: {}", host))
    }
}

/// Validate DNS record type.
fn validate_record_type(rt: &str) -> Result<(), String> {
    const ALLOWED: &[&str] = &["A", "AAAA", "MX", "NS", "TXT", "CNAME", "SOA", "PTR", "SRV", "CAA"];
    if ALLOWED.contains(&rt.to_uppercase().as_str()) {
        Ok(())
    } else {
        Err(format!("Invalid record type: {}", rt))
    }
}

/// Result of a DNS lookup.
#[derive(Debug, Clone, Serialize)]
pub struct DnsResult {
    pub query: String,
    pub record_type: String,
    pub answers: Vec<DnsAnswer>,
    pub server: String,
    pub elapsed_ms: u128,
}

#[derive(Debug, Clone, Serialize)]
pub struct DnsAnswer {
    pub name: String,
    pub record_type: String,
    pub value: String,
    pub ttl: u32,
}

/// Perform a DNS lookup using nslookup or dig.
pub fn lookup(host: &str, record_type: &str, server: Option<&str>) -> Result<DnsResult, String> {
    validate_host(host)?;
    validate_record_type(record_type)?;
    if let Some(s) = server {
        validate_host(s)?;
    }
    let start = std::time::Instant::now();

    #[cfg(windows)]
    {
        dns_lookup_nslookup(host, record_type, server, start)
    }

    #[cfg(not(windows))]
    {
        dns_lookup_dig(host, record_type, server, start)
    }
}

#[cfg(windows)]
fn dns_lookup_nslookup(host: &str, record_type: &str, server: Option<&str>, start: std::time::Instant) -> Result<DnsResult, String> {
    let mut args = vec!["-type=".to_string() + record_type, host.to_string()];
    let srv = server.unwrap_or("default");
    if let Some(s) = server {
        args.push(s.to_string());
    }

    let output = std::process::Command::new("nslookup")
        .args(&args)
        .output()
        .map_err(|e| format!("nslookup failed: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut answers = Vec::new();

    // Parse nslookup output (basic parsing)
    for line in stdout.lines() {
        let line = line.trim();
        if line.starts_with("Address:") && !line.contains('#') {
            let addr = line.replace("Address:", "").trim().to_string();
            if !addr.is_empty() {
                answers.push(DnsAnswer {
                    name: host.to_string(),
                    record_type: record_type.to_uppercase(),
                    value: addr,
                    ttl: 0,
                });
            }
        } else if line.contains("mail exchanger") {
            answers.push(DnsAnswer {
                name: host.to_string(),
                record_type: "MX".to_string(),
                value: line.to_string(),
                ttl: 0,
            });
        } else if line.contains("text =") {
            answers.push(DnsAnswer {
                name: host.to_string(),
                record_type: "TXT".to_string(),
                value: line.replace("text =", "").trim().to_string(),
                ttl: 0,
            });
        }
    }

    Ok(DnsResult {
        query: host.to_string(),
        record_type: record_type.to_uppercase(),
        answers,
        server: srv.to_string(),
        elapsed_ms: start.elapsed().as_millis(),
    })
}

#[cfg(not(windows))]
fn dns_lookup_dig(host: &str, record_type: &str, server: Option<&str>, start: std::time::Instant) -> Result<DnsResult, String> {
    let mut args = vec![host.to_string(), record_type.to_uppercase()];
    let srv = server.unwrap_or("default");
    if let Some(s) = server {
        args.push(format!("@{}", s));
    }
    args.push("+short".to_string());

    let output = std::process::Command::new("dig")
        .args(&args)
        .output()
        .map_err(|e| format!("dig failed: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let answers: Vec<DnsAnswer> = stdout
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| DnsAnswer {
            name: host.to_string(),
            record_type: record_type.to_uppercase(),
            value: l.trim().to_string(),
            ttl: 0,
        })
        .collect();

    Ok(DnsResult {
        query: host.to_string(),
        record_type: record_type.to_uppercase(),
        answers,
        server: srv.to_string(),
        elapsed_ms: start.elapsed().as_millis(),
    })
}
