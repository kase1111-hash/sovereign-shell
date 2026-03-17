//! Process actions: kill, kill tree, suspend, resume, set priority.

use sysinfo::{Pid, Signal, System};

/// Kill a single process by PID.
pub fn kill_process(sys: &System, pid: u32) -> Result<(), String> {
    let sysinfo_pid = Pid::from_u32(pid);
    let proc = sys.process(sysinfo_pid)
        .ok_or_else(|| format!("Process {} not found", pid))?;

    if proc.kill_with(Signal::Term).is_none() {
        // Try harder
        proc.kill();
    }

    Ok(())
}

/// Kill a process and all its descendants.
pub fn kill_tree(sys: &System, root_pid: u32) -> Result<usize, String> {
    let processes = crate::processes::enumerate(sys);
    let pids = crate::processes::get_tree_pids(&processes, root_pid);

    let mut killed = 0;
    // Kill children first (reverse order — deepest first)
    for &pid in pids.iter().rev() {
        let sysinfo_pid = Pid::from_u32(pid);
        if let Some(proc) = sys.process(sysinfo_pid) {
            proc.kill();
            killed += 1;
        }
    }

    Ok(killed)
}

/// Suspend (SIGSTOP) a process — Windows implementation uses NtSuspendProcess.
pub fn suspend_process(_pid: u32) -> Result<(), String> {
    #[cfg(windows)]
    {
        // On Windows, use NtSuspendProcess via windows crate
        // For now, return unsupported on non-Windows
        Err("Suspend requires Windows NtSuspendProcess — implementation pending".to_string())
    }

    #[cfg(not(windows))]
    {
        use std::process::Command;
        let output = Command::new("kill")
            .args(["-STOP", &_pid.to_string()])
            .output()
            .map_err(|e| format!("Suspend failed: {e}"))?;

        if output.status.success() {
            Ok(())
        } else {
            Err(format!("Suspend failed: {}", String::from_utf8_lossy(&output.stderr)))
        }
    }
}

/// Resume a suspended process.
pub fn resume_process(_pid: u32) -> Result<(), String> {
    #[cfg(windows)]
    {
        Err("Resume requires Windows NtResumeProcess — implementation pending".to_string())
    }

    #[cfg(not(windows))]
    {
        use std::process::Command;
        let output = Command::new("kill")
            .args(["-CONT", &_pid.to_string()])
            .output()
            .map_err(|e| format!("Resume failed: {e}"))?;

        if output.status.success() {
            Ok(())
        } else {
            Err(format!("Resume failed: {}", String::from_utf8_lossy(&output.stderr)))
        }
    }
}

/// Set process priority class.
/// Priority: "idle", "below_normal", "normal", "above_normal", "high", "realtime"
pub fn set_priority(_pid: u32, _priority: &str) -> Result<(), String> {
    #[cfg(windows)]
    {
        Err("Set priority via SetPriorityClass — implementation pending".to_string())
    }

    #[cfg(not(windows))]
    {
        let nice_val = match _priority {
            "idle" => "19",
            "below_normal" => "10",
            "normal" => "0",
            "above_normal" => "-5",
            "high" => "-10",
            "realtime" => "-20",
            _ => return Err(format!("Unknown priority: {}", _priority)),
        };

        let output = std::process::Command::new("renice")
            .args([nice_val, "-p", &_pid.to_string()])
            .output()
            .map_err(|e| format!("Set priority failed: {e}"))?;

        if output.status.success() {
            Ok(())
        } else {
            Err(format!("Set priority failed: {}", String::from_utf8_lossy(&output.stderr)))
        }
    }
}
