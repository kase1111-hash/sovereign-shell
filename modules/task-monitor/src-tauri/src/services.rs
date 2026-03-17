//! Windows service enumeration and management.
//!
//! Uses `sc query` as a portable approach. A future version could use
//! EnumServicesStatusEx via the windows crate for richer data.

use serde::Serialize;

/// Validate a service name: only alphanumeric, hyphens, underscores, dots.
fn validate_service_name(name: &str) -> Result<(), String> {
    if name.is_empty() || name.len() > 256 {
        return Err("Invalid service name length".to_string());
    }
    if name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.') {
        Ok(())
    } else {
        Err(format!("Invalid characters in service name: {}", name))
    }
}

/// A Windows service entry.
#[derive(Debug, Clone, Serialize)]
pub struct ServiceInfo {
    pub name: String,
    pub display_name: String,
    pub status: String,
    pub startup_type: String,
    pub pid: u32,
}

/// A service dependency relationship.
#[derive(Debug, Clone, Serialize)]
pub struct ServiceDependency {
    pub service: String,
    pub depends_on: Vec<String>,
}

/// Enumerate all services.
pub fn enumerate_services() -> Result<Vec<ServiceInfo>, String> {
    #[cfg(windows)]
    {
        enumerate_services_windows()
    }

    #[cfg(not(windows))]
    {
        enumerate_services_systemd()
    }
}

#[cfg(windows)]
fn enumerate_services_windows() -> Result<Vec<ServiceInfo>, String> {
    let output = std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "Get-Service | Select-Object Name, DisplayName, Status, StartType | ConvertTo-Json -Compress",
        ])
        .output()
        .map_err(|e| format!("PowerShell failed: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "Get-Service failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let json_str = String::from_utf8_lossy(&output.stdout);
    let raw: serde_json::Value = serde_json::from_str(&json_str)
        .map_err(|e| format!("JSON parse error: {e}"))?;

    let items = match &raw {
        serde_json::Value::Array(arr) => arr.clone(),
        obj @ serde_json::Value::Object(_) => vec![obj.clone()],
        _ => return Err("Unexpected JSON format".to_string()),
    };

    let services = items.iter().filter_map(|item| {
        Some(ServiceInfo {
            name: item.get("Name")?.as_str()?.to_string(),
            display_name: item.get("DisplayName")?.as_str()?.to_string(),
            status: format_service_status(item.get("Status")?.as_i64().unwrap_or(0)),
            startup_type: format_startup_type(item.get("StartType")?.as_i64().unwrap_or(0)),
            pid: 0,
        })
    }).collect();

    Ok(services)
}

#[cfg(windows)]
fn format_service_status(code: i64) -> String {
    match code {
        1 => "Stopped".to_string(),
        2 => "Start Pending".to_string(),
        3 => "Stop Pending".to_string(),
        4 => "Running".to_string(),
        5 => "Continue Pending".to_string(),
        6 => "Pause Pending".to_string(),
        7 => "Paused".to_string(),
        _ => format!("Unknown ({})", code),
    }
}

#[cfg(windows)]
fn format_startup_type(code: i64) -> String {
    match code {
        0 => "Boot".to_string(),
        1 => "System".to_string(),
        2 => "Automatic".to_string(),
        3 => "Manual".to_string(),
        4 => "Disabled".to_string(),
        _ => format!("Unknown ({})", code),
    }
}

#[cfg(not(windows))]
fn enumerate_services_systemd() -> Result<Vec<ServiceInfo>, String> {
    let output = std::process::Command::new("systemctl")
        .args(["list-units", "--type=service", "--all", "--no-pager", "--plain", "--no-legend"])
        .output()
        .map_err(|e| format!("systemctl failed: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut services = Vec::new();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 {
            services.push(ServiceInfo {
                name: parts[0].trim_end_matches(".service").to_string(),
                display_name: parts.get(4..).unwrap_or(&[]).join(" "),
                status: parts.get(2).unwrap_or(&"unknown").to_string(),
                startup_type: parts.get(3).unwrap_or(&"unknown").to_string(),
                pid: 0,
            });
        }
    }

    Ok(services)
}

/// Start a service (requires elevation on Windows).
pub fn start_service(name: &str) -> Result<(), String> {
    validate_service_name(name)?;

    #[cfg(windows)]
    let cmd = std::process::Command::new("sc")
        .args(["start", name])
        .output();

    #[cfg(not(windows))]
    let cmd = std::process::Command::new("systemctl")
        .args(["start", name])
        .output();

    let output = cmd.map_err(|e| format!("Start service failed: {e}"))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "Start service failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

/// Stop a service.
pub fn stop_service(name: &str) -> Result<(), String> {
    validate_service_name(name)?;

    #[cfg(windows)]
    let cmd = std::process::Command::new("sc")
        .args(["stop", name])
        .output();

    #[cfg(not(windows))]
    let cmd = std::process::Command::new("systemctl")
        .args(["stop", name])
        .output();

    let output = cmd.map_err(|e| format!("Stop service failed: {e}"))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "Stop service failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

/// Restart a service.
pub fn restart_service(name: &str) -> Result<(), String> {
    stop_service(name).ok(); // Ignore stop error — may already be stopped
    start_service(name)
}
