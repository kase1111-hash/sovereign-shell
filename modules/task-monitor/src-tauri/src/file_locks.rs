//! File lock finder — "Who is locking this file?"
//!
//! On Windows, uses the Restart Manager API (RmStartSession, RmRegisterResources,
//! RmGetList) to identify which processes have a file locked.
//! On other platforms, uses `lsof` as a fallback.

use serde::Serialize;

/// A process that has a file locked.
#[derive(Debug, Clone, Serialize)]
pub struct LockingProcess {
    pub pid: u32,
    pub name: String,
    pub description: String,
}

/// Find all processes locking a given file path.
pub fn find_locking_processes(file_path: &str) -> Result<Vec<LockingProcess>, String> {
    #[cfg(windows)]
    {
        find_locking_processes_windows(file_path)
    }

    #[cfg(not(windows))]
    {
        find_locking_processes_lsof(file_path)
    }
}

#[cfg(windows)]
fn find_locking_processes_windows(file_path: &str) -> Result<Vec<LockingProcess>, String> {
    use windows::core::PCWSTR;
    use windows::Win32::System::RestartManager::*;

    unsafe {
        let mut session: u32 = 0;
        let mut session_key = [0u16; 256]; // CCH_RM_SESSION_KEY + 1

        let result = RmStartSession(&mut session, 0, session_key.as_mut_ptr());
        if result != 0 {
            return Err(format!("RmStartSession failed with error {}", result));
        }

        // Register the file
        let wide_path: Vec<u16> = file_path.encode_utf16().chain(std::iter::once(0)).collect();
        let file_paths = [PCWSTR(wide_path.as_ptr())];

        let result = RmRegisterResources(
            session,
            Some(&file_paths),
            None,
            None,
        );

        if result != 0 {
            RmEndSession(session);
            return Err(format!("RmRegisterResources failed with error {}", result));
        }

        // Get the list of processes
        let mut needed: u32 = 0;
        let mut count: u32 = 0;
        let mut reason: u32 = 0;

        // First call to get count
        let result = RmGetList(
            session,
            &mut needed,
            &mut count,
            None,
            &mut reason,
        );

        // ERROR_MORE_DATA = 234
        if result != 234 && result != 0 {
            RmEndSession(session);
            return Err(format!("RmGetList sizing failed with error {}", result));
        }

        let mut processes = Vec::new();

        if needed > 0 {
            let mut infos: Vec<RM_PROCESS_INFO> = vec![std::mem::zeroed(); needed as usize];
            count = needed;

            let result = RmGetList(
                session,
                &mut needed,
                &mut count,
                Some(infos.as_mut_ptr()),
                &mut reason,
            );

            if result != 0 {
                RmEndSession(session);
                return Err(format!("RmGetList failed with error {}", result));
            }

            for i in 0..count as usize {
                let info = &infos[i];
                let name = String::from_utf16_lossy(
                    &info.strAppName[..info.strAppName.iter().position(|&c| c == 0).unwrap_or(info.strAppName.len())]
                );
                let desc = String::from_utf16_lossy(
                    &info.strServiceShortName[..info.strServiceShortName.iter().position(|&c| c == 0).unwrap_or(info.strServiceShortName.len())]
                );

                processes.push(LockingProcess {
                    pid: info.Process.dwProcessId,
                    name: name.trim().to_string(),
                    description: if desc.trim().is_empty() {
                        "Application".to_string()
                    } else {
                        desc.trim().to_string()
                    },
                });
            }
        }

        RmEndSession(session);
        Ok(processes)
    }
}

#[cfg(not(windows))]
fn find_locking_processes_lsof(file_path: &str) -> Result<Vec<LockingProcess>, String> {
    let output = std::process::Command::new("lsof")
        .args(["-t", file_path])
        .output()
        .map_err(|e| format!("lsof failed: {e}"))?;

    if !output.status.success() {
        // No processes found is not an error
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut processes = Vec::new();

    for line in stdout.lines() {
        if let Ok(pid) = line.trim().parse::<u32>() {
            // Get process name
            let name = std::process::Command::new("ps")
                .args(["-p", &pid.to_string(), "-o", "comm="])
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .unwrap_or_else(|| format!("PID {}", pid))
                .trim()
                .to_string();

            processes.push(LockingProcess {
                pid,
                name,
                description: "Process".to_string(),
            });
        }
    }

    Ok(processes)
}
