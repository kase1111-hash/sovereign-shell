//! Process enumeration and tree building.

use serde::Serialize;
use sysinfo::{Pid, ProcessStatus, System};
use std::collections::HashMap;

/// A single process entry for the frontend.
#[derive(Debug, Clone, Serialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub ppid: u32,
    pub name: String,
    pub exe: String,
    pub cmd: Vec<String>,
    pub cpu_percent: f32,
    pub memory_bytes: u64,
    pub virtual_memory: u64,
    pub disk_read_bytes: u64,
    pub disk_written_bytes: u64,
    pub status: String,
    pub user: Option<String>,
    pub start_time: u64,
    pub threads: Option<u32>,
}

/// A process tree node (process + children).
#[derive(Debug, Clone, Serialize)]
pub struct ProcessTreeNode {
    pub process: ProcessInfo,
    pub children: Vec<ProcessTreeNode>,
}

/// Snapshot of all running processes.
pub fn enumerate(sys: &System) -> Vec<ProcessInfo> {
    sys.processes()
        .iter()
        .map(|(&pid, proc)| {
            let ppid = proc.parent().map(|p| p.as_u32()).unwrap_or(0);

            ProcessInfo {
                pid: pid.as_u32(),
                ppid,
                name: proc.name().to_string_lossy().to_string(),
                exe: proc.exe().map(|p| p.to_string_lossy().to_string()).unwrap_or_default(),
                cmd: proc.cmd().iter().map(|s| s.to_string_lossy().to_string()).collect(),
                cpu_percent: proc.cpu_usage(),
                memory_bytes: proc.memory(),
                virtual_memory: proc.virtual_memory(),
                disk_read_bytes: proc.disk_usage().read_bytes,
                disk_written_bytes: proc.disk_usage().written_bytes,
                status: format_status(proc.status()),
                user: proc.user_id().map(|u| u.to_string()),
                start_time: proc.start_time(),
                threads: None, // sysinfo doesn't expose thread count directly on all platforms
            }
        })
        .collect()
}

/// Build a parent-child process tree.
pub fn build_tree(processes: &[ProcessInfo]) -> Vec<ProcessTreeNode> {
    let mut children_map: HashMap<u32, Vec<&ProcessInfo>> = HashMap::new();
    let mut all_pids: std::collections::HashSet<u32> = std::collections::HashSet::new();

    for proc in processes {
        all_pids.insert(proc.pid);
        children_map.entry(proc.ppid).or_default().push(proc);
    }

    // Root processes: those whose parent is 0 or whose parent isn't in the list
    let roots: Vec<&ProcessInfo> = processes
        .iter()
        .filter(|p| p.ppid == 0 || !all_pids.contains(&p.ppid))
        .collect();

    fn build_node(proc: &ProcessInfo, children_map: &HashMap<u32, Vec<&ProcessInfo>>) -> ProcessTreeNode {
        let children = children_map
            .get(&proc.pid)
            .map(|kids| {
                kids.iter()
                    .map(|child| build_node(child, children_map))
                    .collect()
            })
            .unwrap_or_default();

        ProcessTreeNode {
            process: proc.clone(),
            children,
        }
    }

    roots.iter().map(|r| build_node(r, &children_map)).collect()
}

/// Find a process and all its descendants.
pub fn get_tree_pids(processes: &[ProcessInfo], root_pid: u32) -> Vec<u32> {
    let mut result = vec![root_pid];
    let mut children_map: HashMap<u32, Vec<u32>> = HashMap::new();

    for proc in processes {
        children_map.entry(proc.ppid).or_default().push(proc.pid);
    }

    let mut stack = vec![root_pid];
    while let Some(pid) = stack.pop() {
        if let Some(kids) = children_map.get(&pid) {
            for &kid in kids {
                result.push(kid);
                stack.push(kid);
            }
        }
    }

    result
}

fn format_status(status: ProcessStatus) -> String {
    match status {
        ProcessStatus::Run => "Running".to_string(),
        ProcessStatus::Sleep => "Sleeping".to_string(),
        ProcessStatus::Stop => "Stopped".to_string(),
        ProcessStatus::Zombie => "Zombie".to_string(),
        ProcessStatus::Idle => "Idle".to_string(),
        _ => "Unknown".to_string(),
    }
}
