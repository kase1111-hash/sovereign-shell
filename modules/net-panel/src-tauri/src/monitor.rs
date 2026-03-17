//! Real-time bandwidth monitoring per adapter.

use serde::Serialize;
use sysinfo::Networks;

/// Bandwidth snapshot for all interfaces.
#[derive(Debug, Clone, Serialize)]
pub struct BandwidthSnapshot {
    pub interfaces: Vec<InterfaceBandwidth>,
    pub total_rx_bytes_sec: u64,
    pub total_tx_bytes_sec: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct InterfaceBandwidth {
    pub name: String,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub rx_bytes_sec: u64,
    pub tx_bytes_sec: u64,
}

/// Collect bandwidth data. Call periodically (every 1s) for rate calculation.
pub fn collect_bandwidth(prev: &Option<BandwidthSnapshot>) -> BandwidthSnapshot {
    let networks = Networks::new_with_refreshed_list();
    let mut interfaces = Vec::new();
    let mut total_rx = 0u64;
    let mut total_tx = 0u64;

    for (name, data) in networks.iter() {
        let rx = data.received();
        let tx = data.transmitted();

        // Calculate rate from previous snapshot
        let (rx_sec, tx_sec) = if let Some(prev) = prev {
            if let Some(prev_iface) = prev.interfaces.iter().find(|i| i.name == *name) {
                (
                    rx.saturating_sub(prev_iface.rx_bytes),
                    tx.saturating_sub(prev_iface.tx_bytes),
                )
            } else {
                (0, 0)
            }
        } else {
            (0, 0)
        };

        total_rx += rx_sec;
        total_tx += tx_sec;

        interfaces.push(InterfaceBandwidth {
            name: name.to_string(),
            rx_bytes: rx,
            tx_bytes: tx,
            rx_bytes_sec: rx_sec,
            tx_bytes_sec: tx_sec,
        });
    }

    BandwidthSnapshot {
        interfaces,
        total_rx_bytes_sec: total_rx,
        total_tx_bytes_sec: total_tx,
    }
}
