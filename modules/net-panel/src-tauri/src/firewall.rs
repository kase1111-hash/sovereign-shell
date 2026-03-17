//! Windows Firewall rule management via netsh advfirewall.

use serde::{Deserialize, Serialize};

/// Sanitize a string for PowerShell single-quoted strings.
fn sanitize_ps_arg(s: &str) -> Result<String, String> {
    if s.bytes().any(|b| b == 0 || (b < 0x20 && b != b'\t')) {
        return Err("Input contains invalid characters".to_string());
    }
    Ok(s.replace('\'', "''"))
}

/// Validate a firewall rule field: reject shell metacharacters.
fn validate_rule_field(field: &str, name: &str) -> Result<(), String> {
    if field.is_empty() { return Ok(()); }
    // Allow alphanumeric, spaces, hyphens, underscores, dots, colons, slashes (for paths)
    if field.chars().all(|c| c.is_alphanumeric() || " -_.:/\\".contains(c)) {
        Ok(())
    } else {
        Err(format!("Invalid characters in {}: {}", name, field))
    }
}

/// A firewall rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallRule {
    pub name: String,
    pub direction: String,   // "In" or "Out"
    pub action: String,      // "Allow" or "Block"
    pub protocol: String,    // "TCP", "UDP", "Any"
    pub local_port: String,
    pub remote_port: String,
    pub program: String,
    pub enabled: bool,
    pub profile: String,     // "Domain", "Private", "Public", "Any"
}

/// Enumerate all firewall rules.
pub fn get_rules() -> Result<Vec<FirewallRule>, String> {
    #[cfg(windows)]
    {
        get_rules_netsh()
    }

    #[cfg(not(windows))]
    {
        get_rules_iptables()
    }
}

#[cfg(windows)]
fn get_rules_netsh() -> Result<Vec<FirewallRule>, String> {
    let output = std::process::Command::new("powershell")
        .args([
            "-NoProfile", "-Command",
            "Get-NetFirewallRule | Select-Object DisplayName, Direction, Action, Enabled, Profile | ConvertTo-Json -Compress"
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

    let rules: Vec<FirewallRule> = items.iter().filter_map(|item| {
        let name = item.get("DisplayName")?.as_str()?.to_string();
        let direction = match item.get("Direction")?.as_i64()? {
            1 => "In", 2 => "Out", _ => "Any"
        }.to_string();
        let action = match item.get("Action")?.as_i64()? {
            2 => "Allow", 4 => "Block", _ => "Allow"
        }.to_string();
        let enabled = item.get("Enabled").and_then(|v| v.as_i64()).unwrap_or(0) == 1;
        let profile = match item.get("Profile").and_then(|v| v.as_i64()).unwrap_or(0) {
            1 => "Domain", 2 => "Private", 4 => "Public",
            2147483647 => "Any", _ => "Any"
        }.to_string();

        Some(FirewallRule {
            name,
            direction,
            action,
            protocol: "Any".to_string(),
            local_port: "Any".to_string(),
            remote_port: "Any".to_string(),
            program: String::new(),
            enabled,
            profile,
        })
    }).collect();

    Ok(rules)
}

#[cfg(not(windows))]
fn get_rules_iptables() -> Result<Vec<FirewallRule>, String> {
    let output = std::process::Command::new("iptables")
        .args(["-L", "-n", "--line-numbers"])
        .output()
        .map_err(|e| format!("iptables failed: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut rules = Vec::new();
    let mut current_chain = String::new();

    for line in stdout.lines() {
        if line.starts_with("Chain") {
            current_chain = line.split_whitespace().nth(1).unwrap_or("").to_string();
        } else if line.starts_with("num") || line.is_empty() {
            continue;
        } else {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                rules.push(FirewallRule {
                    name: format!("{} #{}", current_chain, parts[0]),
                    direction: if current_chain == "INPUT" { "In" } else { "Out" }.to_string(),
                    action: parts[1].to_string(),
                    protocol: parts[2].to_string(),
                    local_port: "Any".to_string(),
                    remote_port: "Any".to_string(),
                    program: String::new(),
                    enabled: true,
                    profile: "Any".to_string(),
                });
            }
        }
    }

    Ok(rules)
}

/// Toggle a firewall rule on/off.
pub fn toggle_rule(name: &str, enabled: bool) -> Result<(), String> {
    #[cfg(windows)]
    {
        let safe_name = sanitize_ps_arg(name)?;
        let action = if enabled { "Enable" } else { "Disable" };
        let output = std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command",
                &format!("{}-NetFirewallRule -DisplayName '{}'", action, safe_name)])
            .output()
            .map_err(|e| format!("{e}"))?;
        if output.status.success() { Ok(()) }
        else { Err(String::from_utf8_lossy(&output.stderr).to_string()) }
    }
    #[cfg(not(windows))]
    {
        Err("Firewall rule toggle not implemented for this platform".to_string())
    }
}

/// Create a new firewall rule.
pub fn create_rule(rule: &FirewallRule) -> Result<(), String> {
    #[cfg(windows)]
    {
        // Validate all fields before constructing the command
        let safe_name = sanitize_ps_arg(&rule.name)?;
        validate_rule_field(&rule.protocol, "protocol")?;
        validate_rule_field(&rule.local_port, "local_port")?;
        validate_rule_field(&rule.action, "action")?;
        validate_rule_field(&rule.direction, "direction")?;

        let dir = if rule.direction == "In" { "Inbound" } else { "Outbound" };

        // Validate action is one of the allowed values
        if rule.action != "Allow" && rule.action != "Block" {
            return Err("Action must be 'Allow' or 'Block'".to_string());
        }

        let mut cmd = format!(
            "New-NetFirewallRule -DisplayName '{}' -Direction {} -Action {}",
            safe_name, dir, rule.action
        );
        if rule.protocol != "Any" && !rule.protocol.is_empty() {
            cmd += &format!(" -Protocol {}", rule.protocol);
        }
        if rule.local_port != "Any" && !rule.local_port.is_empty() {
            cmd += &format!(" -LocalPort {}", rule.local_port);
        }
        if !rule.program.is_empty() {
            let safe_program = sanitize_ps_arg(&rule.program)?;
            cmd += &format!(" -Program '{}'", safe_program);
        }

        let output = std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", &cmd])
            .output()
            .map_err(|e| format!("{e}"))?;
        if output.status.success() { Ok(()) }
        else { Err(String::from_utf8_lossy(&output.stderr).to_string()) }
    }
    #[cfg(not(windows))]
    {
        Err("Firewall rule creation not implemented for this platform".to_string())
    }
}
