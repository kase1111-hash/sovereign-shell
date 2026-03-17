//! Active TCP/UDP connection enumeration with process attribution.

use serde::Serialize;

/// An active network connection.
#[derive(Debug, Clone, Serialize)]
pub struct Connection {
    pub protocol: String,       // "TCP" or "UDP"
    pub local_address: String,
    pub local_port: u16,
    pub remote_address: String,
    pub remote_port: u16,
    pub state: String,          // "ESTABLISHED", "LISTEN", "TIME_WAIT", etc.
    pub pid: u32,
    pub process_name: String,
}

/// Get all active connections.
pub fn get_connections() -> Result<Vec<Connection>, String> {
    #[cfg(windows)]
    {
        get_connections_netstat()
    }

    #[cfg(not(windows))]
    {
        get_connections_ss()
    }
}

fn get_connections_netstat() -> Result<Vec<Connection>, String> {
    let output = std::process::Command::new("netstat")
        .args(["-ano"])
        .output()
        .map_err(|e| format!("netstat failed: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut connections = Vec::new();

    for line in stdout.lines().skip(4) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 { continue; }

        let protocol = parts[0].to_uppercase();
        if protocol != "TCP" && protocol != "UDP" { continue; }

        let (local_addr, local_port) = parse_endpoint(parts[1]);
        let (remote_addr, remote_port) = if protocol == "TCP" && parts.len() >= 5 {
            parse_endpoint(parts[2])
        } else {
            (String::new(), 0)
        };

        let (state, pid_str) = if protocol == "TCP" && parts.len() >= 5 {
            (parts[3].to_string(), parts[4])
        } else if parts.len() >= 4 {
            ("".to_string(), parts.last().unwrap_or(&"0"))
        } else {
            continue;
        };

        let pid: u32 = pid_str.parse().unwrap_or(0);
        let process_name = get_process_name(pid);

        connections.push(Connection {
            protocol,
            local_address: local_addr,
            local_port,
            remote_address: remote_addr,
            remote_port,
            state,
            pid,
            process_name,
        });
    }

    Ok(connections)
}

#[cfg(not(windows))]
fn get_connections_ss() -> Result<Vec<Connection>, String> {
    let output = std::process::Command::new("ss")
        .args(["-tunap"])
        .output()
        .map_err(|e| format!("ss failed: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut connections = Vec::new();

    for line in stdout.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 5 { continue; }

        let protocol = parts[0].to_uppercase();
        let state = parts[1].to_string();
        let (local_addr, local_port) = parse_endpoint_unix(parts[4]);
        let (remote_addr, remote_port) = if parts.len() > 5 {
            parse_endpoint_unix(parts[5])
        } else {
            (String::new(), 0)
        };

        // Extract PID from the last field
        let pid_info = parts.last().unwrap_or(&"");
        let pid: u32 = pid_info
            .split("pid=").nth(1)
            .and_then(|s| s.split(',').next())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        let process_name = if pid > 0 {
            std::fs::read_to_string(format!("/proc/{}/comm", pid))
                .unwrap_or_default().trim().to_string()
        } else {
            String::new()
        };

        connections.push(Connection {
            protocol, local_address: local_addr, local_port,
            remote_address: remote_addr, remote_port,
            state, pid, process_name,
        });
    }

    Ok(connections)
}

fn parse_endpoint(s: &str) -> (String, u16) {
    // Handle IPv4 "addr:port" and IPv6 "[addr]:port"
    if let Some(last_colon) = s.rfind(':') {
        let addr = &s[..last_colon];
        let port: u16 = s[last_colon + 1..].parse().unwrap_or(0);
        (addr.to_string(), port)
    } else {
        (s.to_string(), 0)
    }
}

#[cfg(not(windows))]
fn parse_endpoint_unix(s: &str) -> (String, u16) {
    parse_endpoint(s)
}

fn get_process_name(pid: u32) -> String {
    if pid == 0 { return "System".to_string(); }

    #[cfg(windows)]
    {
        use std::process::Command;
        Command::new("powershell")
            .args(["-NoProfile", "-Command",
                &format!("(Get-Process -Id {} -ErrorAction SilentlyContinue).ProcessName", pid)])
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_else(|| format!("PID {}", pid))
    }

    #[cfg(not(windows))]
    {
        std::fs::read_to_string(format!("/proc/{}/comm", pid))
            .unwrap_or_else(|_| format!("PID {}", pid))
            .trim()
            .to_string()
    }
}
