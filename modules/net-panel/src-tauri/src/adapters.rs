//! Network adapter enumeration and configuration.
//!
//! Uses `ipconfig` / `netsh` for portable access. A future version could
//! use GetAdaptersAddresses via the windows crate for richer data.

use serde::Serialize;

/// Sanitize a string for use in PowerShell single-quoted strings.
/// Single quotes inside single-quoted strings are escaped by doubling them.
/// Also reject characters that could break out of the command context.
fn sanitize_ps_arg(s: &str) -> Result<String, String> {
    // Reject null bytes and control characters
    if s.bytes().any(|b| b == 0 || (b < 0x20 && b != b'\t')) {
        return Err("Input contains invalid characters".to_string());
    }
    // Escape single quotes by doubling them (PowerShell escaping for single-quoted strings)
    Ok(s.replace('\'', "''"))
}

/// Validate a hostname/IP: only alphanumeric, dots, colons, hyphens.
fn validate_network_name(s: &str) -> Result<(), String> {
    if s.is_empty() || s.len() > 253 {
        return Err("Invalid network name length".to_string());
    }
    if s.chars().all(|c| c.is_alphanumeric() || c == '.' || c == ':' || c == '-' || c == '_') {
        Ok(())
    } else {
        Err(format!("Invalid characters in network name: {}", s))
    }
}

/// A network adapter/interface.
#[derive(Debug, Clone, Serialize)]
pub struct NetworkAdapter {
    pub name: String,
    pub description: String,
    pub adapter_type: String, // "Ethernet", "Wi-Fi", "Loopback", "VPN", "Other"
    pub status: String,       // "Up", "Down"
    pub mac_address: String,
    pub ipv4: Vec<String>,
    pub ipv6: Vec<String>,
    pub subnet_mask: String,
    pub gateway: String,
    pub dns_servers: Vec<String>,
    pub speed_mbps: u64,
    pub dhcp_enabled: bool,
}

/// Enumerate all network adapters.
pub fn enumerate_adapters() -> Result<Vec<NetworkAdapter>, String> {
    #[cfg(windows)]
    {
        enumerate_adapters_ipconfig()
    }

    #[cfg(not(windows))]
    {
        enumerate_adapters_ip()
    }
}

#[cfg(windows)]
fn enumerate_adapters_ipconfig() -> Result<Vec<NetworkAdapter>, String> {
    let output = std::process::Command::new("powershell")
        .args([
            "-NoProfile", "-Command",
            "Get-NetAdapter | Select-Object Name, InterfaceDescription, Status, MacAddress, LinkSpeed, ifIndex | ConvertTo-Json -Compress"
        ])
        .output()
        .map_err(|e| format!("PowerShell failed: {e}"))?;

    let json_str = String::from_utf8_lossy(&output.stdout);
    let raw: serde_json::Value = serde_json::from_str(&json_str).unwrap_or(serde_json::Value::Array(vec![]));

    let items = match &raw {
        serde_json::Value::Array(arr) => arr.clone(),
        obj @ serde_json::Value::Object(_) => vec![obj.clone()],
        _ => vec![],
    };

    let mut adapters = Vec::new();
    for item in &items {
        let name = item.get("Name").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let description = item.get("InterfaceDescription").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let status = item.get("Status").and_then(|v| v.as_str()).unwrap_or("Down").to_string();
        let mac = item.get("MacAddress").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let speed_str = item.get("LinkSpeed").and_then(|v| v.as_str()).unwrap_or("0");
        let speed_mbps = parse_speed(speed_str);
        let if_index = item.get("ifIndex").and_then(|v| v.as_u64()).unwrap_or(0);

        // Get IP config for this adapter
        let (ipv4, ipv6, subnet, gateway, dns, dhcp) = get_adapter_ip_config(&name, if_index);

        let adapter_type = classify_adapter(&name, &description);

        adapters.push(NetworkAdapter {
            name,
            description,
            adapter_type,
            status,
            mac_address: mac,
            ipv4,
            ipv6,
            subnet_mask: subnet,
            gateway,
            dns_servers: dns,
            speed_mbps,
            dhcp_enabled: dhcp,
        });
    }

    Ok(adapters)
}

#[cfg(windows)]
fn get_adapter_ip_config(_name: &str, if_index: u64) -> (Vec<String>, Vec<String>, String, String, Vec<String>, bool) {
    let output = std::process::Command::new("powershell")
        .args([
            "-NoProfile", "-Command",
            &format!(
                "Get-NetIPConfiguration -InterfaceIndex {} -ErrorAction SilentlyContinue | ConvertTo-Json -Compress -Depth 3",
                if_index
            ),
        ])
        .output()
        .ok();

    let mut ipv4 = Vec::new();
    let mut ipv6 = Vec::new();
    let mut subnet = String::new();
    let mut gateway = String::new();
    let mut dns = Vec::new();
    let dhcp = false;

    if let Some(out) = output {
        let json_str = String::from_utf8_lossy(&out.stdout);
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&json_str) {
            // IPv4
            if let Some(v4) = val.get("IPv4Address") {
                let addrs = if v4.is_array() { v4.as_array().unwrap().clone() } else { vec![v4.clone()] };
                for addr in &addrs {
                    if let Some(ip) = addr.get("IPAddress").and_then(|v| v.as_str()) {
                        ipv4.push(ip.to_string());
                    }
                    if subnet.is_empty() {
                        if let Some(pl) = addr.get("PrefixLength").and_then(|v| v.as_u64()) {
                            subnet = prefix_to_mask(pl as u8);
                        }
                    }
                }
            }

            // IPv6
            if let Some(v6) = val.get("IPv6Address") {
                let addrs = if v6.is_array() { v6.as_array().unwrap().clone() } else { vec![v6.clone()] };
                for addr in &addrs {
                    if let Some(ip) = addr.get("IPAddress").and_then(|v| v.as_str()) {
                        ipv6.push(ip.to_string());
                    }
                }
            }

            // Gateway
            if let Some(gw) = val.get("IPv4DefaultGateway") {
                let gws = if gw.is_array() { gw.as_array().unwrap().clone() } else { vec![gw.clone()] };
                for g in &gws {
                    if let Some(ip) = g.get("NextHop").and_then(|v| v.as_str()) {
                        gateway = ip.to_string();
                    }
                }
            }

            // DNS
            if let Some(dns_val) = val.get("DNSServer") {
                let servers = if dns_val.is_array() { dns_val.as_array().unwrap().clone() } else { vec![dns_val.clone()] };
                for s in &servers {
                    if let Some(addr) = s.get("ServerAddresses").and_then(|v| v.as_array()) {
                        for a in addr {
                            if let Some(ip) = a.as_str() {
                                dns.push(ip.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    (ipv4, ipv6, subnet, gateway, dns, dhcp)
}

#[cfg(windows)]
fn parse_speed(s: &str) -> u64 {
    let s = s.trim().to_lowercase();
    if s.contains("gbps") {
        s.replace("gbps", "").trim().parse::<f64>().unwrap_or(0.0) as u64 * 1000
    } else if s.contains("mbps") {
        s.replace("mbps", "").trim().parse::<f64>().unwrap_or(0.0) as u64
    } else {
        0
    }
}

#[cfg(windows)]
fn prefix_to_mask(prefix: u8) -> String {
    if prefix > 32 { return String::new(); }
    let mask: u32 = if prefix == 0 { 0 } else { !0u32 << (32 - prefix) };
    format!("{}.{}.{}.{}",
        (mask >> 24) & 0xff, (mask >> 16) & 0xff, (mask >> 8) & 0xff, mask & 0xff)
}

#[cfg(windows)]
fn classify_adapter(name: &str, desc: &str) -> String {
    let lower = format!("{} {}", name, desc).to_lowercase();
    if lower.contains("wi-fi") || lower.contains("wireless") || lower.contains("wlan") {
        "Wi-Fi".to_string()
    } else if lower.contains("loopback") {
        "Loopback".to_string()
    } else if lower.contains("vpn") || lower.contains("tap") || lower.contains("tun") {
        "VPN".to_string()
    } else if lower.contains("ethernet") || lower.contains("realtek") || lower.contains("intel") {
        "Ethernet".to_string()
    } else {
        "Other".to_string()
    }
}

#[cfg(not(windows))]
fn enumerate_adapters_ip() -> Result<Vec<NetworkAdapter>, String> {
    let output = std::process::Command::new("ip")
        .args(["-j", "addr", "show"])
        .output()
        .map_err(|e| format!("ip command failed: {e}"))?;

    let json_str = String::from_utf8_lossy(&output.stdout);
    let items: Vec<serde_json::Value> = serde_json::from_str(&json_str).unwrap_or_default();

    let mut adapters = Vec::new();
    for item in &items {
        let name = item.get("ifname").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let status = item.get("operstate").and_then(|v| v.as_str()).unwrap_or("DOWN").to_string();

        let mut ipv4 = Vec::new();
        let mut ipv6 = Vec::new();
        let mut subnet = String::new();

        if let Some(addr_info) = item.get("addr_info").and_then(|v| v.as_array()) {
            for ai in addr_info {
                let family = ai.get("family").and_then(|v| v.as_str()).unwrap_or("");
                let local = ai.get("local").and_then(|v| v.as_str()).unwrap_or("");
                match family {
                    "inet" => {
                        ipv4.push(local.to_string());
                        if subnet.is_empty() {
                            if let Some(pl) = ai.get("prefixlen").and_then(|v| v.as_u64()) {
                                subnet = format!("/{}", pl);
                            }
                        }
                    }
                    "inet6" => ipv6.push(local.to_string()),
                    _ => {}
                }
            }
        }

        let mac = item.get("address").and_then(|v| v.as_str()).unwrap_or("").to_string();

        let adapter_type = if name == "lo" { "Loopback" }
            else if name.starts_with("wl") { "Wi-Fi" }
            else if name.starts_with("tun") || name.starts_with("tap") { "VPN" }
            else { "Ethernet" };

        adapters.push(NetworkAdapter {
            name,
            description: String::new(),
            adapter_type: adapter_type.to_string(),
            status: if status == "UP" { "Up".to_string() } else { "Down".to_string() },
            mac_address: mac,
            ipv4,
            ipv6,
            subnet_mask: subnet,
            gateway: String::new(),
            dns_servers: Vec::new(),
            speed_mbps: 0,
            dhcp_enabled: false,
        });
    }

    Ok(adapters)
}

/// Enable or disable a network adapter.
pub fn set_adapter_state(name: &str, enabled: bool) -> Result<(), String> {
    #[cfg(windows)]
    {
        let safe_name = sanitize_ps_arg(name)?;
        let action = if enabled { "Enable-NetAdapter" } else { "Disable-NetAdapter" };
        let output = std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", &format!("{} -Name '{}' -Confirm:$false", action, safe_name)])
            .output()
            .map_err(|e| format!("{e}"))?;
        if output.status.success() { Ok(()) }
        else { Err(String::from_utf8_lossy(&output.stderr).to_string()) }
    }
    #[cfg(not(windows))]
    {
        validate_network_name(name)?;
        let action = if enabled { "up" } else { "down" };
        let output = std::process::Command::new("ip")
            .args(["link", "set", name, action])
            .output()
            .map_err(|e| format!("{e}"))?;
        if output.status.success() { Ok(()) }
        else { Err(String::from_utf8_lossy(&output.stderr).to_string()) }
    }
}

/// Set DNS servers for an adapter.
pub fn set_dns(adapter_name: &str, servers: &[String]) -> Result<(), String> {
    #[cfg(windows)]
    {
        let safe_name = sanitize_ps_arg(adapter_name)?;
        // Validate each server is a valid IP/hostname
        for s in servers {
            validate_network_name(s)?;
        }
        let servers_str = servers.iter()
            .map(|s| format!("'{}'", sanitize_ps_arg(s).unwrap_or_default()))
            .collect::<Vec<_>>()
            .join(",");
        let output = std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command",
                &format!("Set-DnsClientServerAddress -InterfaceAlias '{}' -ServerAddresses @({})", safe_name, servers_str)])
            .output()
            .map_err(|e| format!("{e}"))?;
        if output.status.success() { Ok(()) }
        else { Err(String::from_utf8_lossy(&output.stderr).to_string()) }
    }
    #[cfg(not(windows))]
    {
        Err("DNS configuration on Linux requires editing /etc/resolv.conf — not yet automated".to_string())
    }
}
