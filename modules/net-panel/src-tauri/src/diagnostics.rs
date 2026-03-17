//! Network diagnostics: ping, traceroute, DNS lookup.

use serde::Serialize;

/// Validate a hostname or IP address: only safe characters allowed.
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

/// A single ping result.
#[derive(Debug, Clone, Serialize)]
pub struct PingResult {
    pub host: String,
    pub seq: u32,
    pub rtt_ms: f64,
    pub ttl: u32,
    pub success: bool,
    pub error: Option<String>,
}

/// A traceroute hop.
#[derive(Debug, Clone, Serialize)]
pub struct TracerouteHop {
    pub hop: u32,
    pub address: String,
    pub hostname: String,
    pub rtt_ms: Vec<f64>,
    pub timed_out: bool,
}

/// Run a ping and return all results.
pub fn ping(host: &str, count: u32) -> Result<Vec<PingResult>, String> {
    validate_host(host)?;
    let count = count.min(100); // Cap to prevent abuse

    #[cfg(windows)]
    let output = std::process::Command::new("ping")
        .args(["-n", &count.to_string(), host])
        .output()
        .map_err(|e| format!("Ping failed: {e}"))?;

    #[cfg(not(windows))]
    let output = std::process::Command::new("ping")
        .args(["-c", &count.to_string(), host])
        .output()
        .map_err(|e| format!("Ping failed: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut results = Vec::new();
    let mut seq = 0u32;

    for line in stdout.lines() {
        let line_lower = line.to_lowercase();

        if line_lower.contains("time=") || line_lower.contains("time<") {
            seq += 1;

            let rtt = extract_rtt(line);
            let ttl = extract_ttl(line);

            results.push(PingResult {
                host: host.to_string(),
                seq,
                rtt_ms: rtt,
                ttl,
                success: true,
                error: None,
            });
        } else if line_lower.contains("request timed out") || line_lower.contains("100% packet loss") {
            seq += 1;
            results.push(PingResult {
                host: host.to_string(),
                seq,
                rtt_ms: 0.0,
                ttl: 0,
                success: false,
                error: Some("Request timed out".to_string()),
            });
        }
    }

    if results.is_empty() && !output.status.success() {
        return Err(format!("Ping failed: {}", stdout));
    }

    Ok(results)
}

/// Run a traceroute.
pub fn traceroute(host: &str, max_hops: u32) -> Result<Vec<TracerouteHop>, String> {
    validate_host(host)?;
    let max_hops = max_hops.min(64); // Cap to prevent abuse

    #[cfg(windows)]
    let output = std::process::Command::new("tracert")
        .args(["-h", &max_hops.to_string(), "-d", host])
        .output()
        .map_err(|e| format!("Traceroute failed: {e}"))?;

    #[cfg(not(windows))]
    let output = std::process::Command::new("traceroute")
        .args(["-m", &max_hops.to_string(), "-n", host])
        .output()
        .map_err(|e| format!("Traceroute failed: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut hops = Vec::new();

    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() { continue; }

        // Try to parse hop number at start of line
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() { continue; }

        let hop_num: u32 = match parts[0].parse() {
            Ok(n) => n,
            Err(_) => continue,
        };

        if line.contains("* * *") || line.contains("Request timed out") {
            hops.push(TracerouteHop {
                hop: hop_num,
                address: "*".to_string(),
                hostname: String::new(),
                rtt_ms: Vec::new(),
                timed_out: true,
            });
            continue;
        }

        // Extract address and RTTs
        let mut address = String::new();
        let mut rtts = Vec::new();

        for part in &parts[1..] {
            if part.contains('.') && !part.contains("ms") {
                if address.is_empty() {
                    address = part.to_string();
                }
            }
            if part.ends_with("ms") || *part == "ms" {
                // Previous part was the number
            }
            if let Ok(rtt) = part.replace("ms", "").parse::<f64>() {
                rtts.push(rtt);
            }
        }

        hops.push(TracerouteHop {
            hop: hop_num,
            address: address.clone(),
            hostname: address,
            rtt_ms: rtts,
            timed_out: false,
        });
    }

    Ok(hops)
}

fn extract_rtt(line: &str) -> f64 {
    // Match "time=XX.Xms" or "time<1ms"
    if let Some(idx) = line.to_lowercase().find("time=") {
        let after = &line[idx + 5..];
        let num: String = after.chars().take_while(|c| c.is_ascii_digit() || *c == '.').collect();
        num.parse().unwrap_or(0.0)
    } else if line.to_lowercase().contains("time<1ms") {
        0.5
    } else {
        0.0
    }
}

fn extract_ttl(line: &str) -> u32 {
    if let Some(idx) = line.to_lowercase().find("ttl=") {
        let after = &line[idx + 4..];
        let num: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
        num.parse().unwrap_or(0)
    } else {
        0
    }
}
