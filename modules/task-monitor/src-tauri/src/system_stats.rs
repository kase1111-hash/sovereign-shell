//! System-wide performance statistics: CPU, RAM, disk, network.

use serde::Serialize;
use sysinfo::{CpuRefreshKind, Disks, Networks, RefreshKind, System};

/// A snapshot of system performance.
#[derive(Debug, Clone, Serialize)]
pub struct SystemStats {
    pub cpu: CpuStats,
    pub memory: MemoryStats,
    pub disks: Vec<DiskStats>,
    pub network: NetworkStats,
}

#[derive(Debug, Clone, Serialize)]
pub struct CpuStats {
    /// Overall CPU usage as a percentage (0-100).
    pub total_percent: f32,
    /// Per-core usage percentages.
    pub per_core: Vec<f32>,
    /// Number of physical cores.
    pub physical_cores: usize,
    /// Number of logical cores.
    pub logical_cores: usize,
    /// CPU brand/model string.
    pub brand: String,
    /// CPU frequency in MHz.
    pub frequency_mhz: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct MemoryStats {
    /// Total physical memory in bytes.
    pub total: u64,
    /// Used memory in bytes.
    pub used: u64,
    /// Available memory in bytes.
    pub available: u64,
    /// Total swap in bytes.
    pub swap_total: u64,
    /// Used swap in bytes.
    pub swap_used: u64,
    /// Usage percentage (0-100).
    pub percent: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiskStats {
    pub name: String,
    pub mount_point: String,
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub fs_type: String,
    pub is_removable: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct NetworkStats {
    /// Total bytes received since last refresh.
    pub rx_bytes: u64,
    /// Total bytes transmitted since last refresh.
    pub tx_bytes: u64,
    /// Per-interface breakdown.
    pub interfaces: Vec<InterfaceStats>,
}

#[derive(Debug, Clone, Serialize)]
pub struct InterfaceStats {
    pub name: String,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub rx_packets: u64,
    pub tx_packets: u64,
}

/// Collect a full system stats snapshot.
pub fn collect(sys: &System) -> SystemStats {
    let cpu = collect_cpu(sys);
    let memory = collect_memory(sys);
    let disks = collect_disks();
    let network = collect_network();

    SystemStats { cpu, memory, disks, network }
}

fn collect_cpu(sys: &System) -> CpuStats {
    let cpus = sys.cpus();
    let total_percent = sys.global_cpu_usage();
    let per_core: Vec<f32> = cpus.iter().map(|c| c.cpu_usage()).collect();

    let brand = cpus.first()
        .map(|c| c.brand().to_string())
        .unwrap_or_default();
    let frequency_mhz = cpus.first()
        .map(|c| c.frequency())
        .unwrap_or(0);

    CpuStats {
        total_percent,
        per_core,
        physical_cores: sys.physical_core_count().unwrap_or(0),
        logical_cores: cpus.len(),
        brand,
        frequency_mhz,
    }
}

fn collect_memory(sys: &System) -> MemoryStats {
    let total = sys.total_memory();
    let used = sys.used_memory();
    let available = sys.available_memory();
    let percent = if total > 0 { (used as f32 / total as f32) * 100.0 } else { 0.0 };

    MemoryStats {
        total,
        used,
        available,
        swap_total: sys.total_swap(),
        swap_used: sys.used_swap(),
        percent,
    }
}

fn collect_disks() -> Vec<DiskStats> {
    let disks = Disks::new_with_refreshed_list();
    disks.iter().map(|d| {
        DiskStats {
            name: d.name().to_string_lossy().to_string(),
            mount_point: d.mount_point().to_string_lossy().to_string(),
            total_bytes: d.total_space(),
            available_bytes: d.available_space(),
            fs_type: d.file_system().to_string_lossy().to_string(),
            is_removable: d.is_removable(),
        }
    }).collect()
}

fn collect_network() -> NetworkStats {
    let networks = Networks::new_with_refreshed_list();
    let mut total_rx = 0u64;
    let mut total_tx = 0u64;
    let mut interfaces = Vec::new();

    for (name, data) in networks.iter() {
        let rx = data.received();
        let tx = data.transmitted();
        total_rx += rx;
        total_tx += tx;

        interfaces.push(InterfaceStats {
            name: name.to_string(),
            rx_bytes: rx,
            tx_bytes: tx,
            rx_packets: data.packets_received(),
            tx_packets: data.packets_transmitted(),
        });
    }

    NetworkStats {
        rx_bytes: total_rx,
        tx_bytes: total_tx,
        interfaces,
    }
}
