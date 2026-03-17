//! Audio event handling — device arrival/removal, session changes.
//!
//! On Windows, uses IMMNotificationClient for device events and
//! IAudioSessionNotification for session lifecycle events.
//! These would need COM callback implementations; for now we poll.

use serde::Serialize;

/// An audio event for the frontend.
#[derive(Debug, Clone, Serialize)]
pub struct AudioEvent {
    pub event_type: String,  // "device_added", "device_removed", "session_created", "session_expired"
    pub device_id: Option<String>,
    pub process_id: Option<u32>,
    pub description: String,
}

/// Check for device/session changes (poll-based).
/// Returns a list of changes since last check.
///
/// A future version will use COM callbacks for real-time events:
/// - IMMNotificationClient::OnDeviceAdded / OnDeviceRemoved
/// - IAudioSessionNotification::OnSessionCreated
pub fn poll_changes(
    prev_device_ids: &[String],
    prev_session_pids: &[u32],
) -> Result<(Vec<AudioEvent>, Vec<String>, Vec<u32>), String> {
    let mut events = Vec::new();

    // Get current devices
    let devices = crate::devices::enumerate_devices().unwrap_or_default();
    let current_device_ids: Vec<String> = devices.iter().map(|d| d.id.clone()).collect();

    // Detect new devices
    for id in &current_device_ids {
        if !prev_device_ids.contains(id) {
            let name = devices.iter().find(|d| &d.id == id).map(|d| d.name.clone()).unwrap_or_default();
            events.push(AudioEvent {
                event_type: "device_added".to_string(),
                device_id: Some(id.clone()),
                process_id: None,
                description: format!("Device connected: {}", name),
            });
        }
    }

    // Detect removed devices
    for id in prev_device_ids {
        if !current_device_ids.contains(id) {
            events.push(AudioEvent {
                event_type: "device_removed".to_string(),
                device_id: Some(id.clone()),
                process_id: None,
                description: format!("Device disconnected: {}", id),
            });
        }
    }

    // Get current sessions
    let sessions = crate::sessions::enumerate_sessions().unwrap_or_default();
    let current_pids: Vec<u32> = sessions.iter().map(|s| s.process_id).collect();

    // Detect new sessions
    for &pid in &current_pids {
        if !prev_session_pids.contains(&pid) {
            let name = sessions.iter().find(|s| s.process_id == pid).map(|s| s.process_name.clone()).unwrap_or_default();
            events.push(AudioEvent {
                event_type: "session_created".to_string(),
                device_id: None,
                process_id: Some(pid),
                description: format!("Audio session started: {}", name),
            });
        }
    }

    // Detect expired sessions
    for &pid in prev_session_pids {
        if !current_pids.contains(&pid) {
            events.push(AudioEvent {
                event_type: "session_expired".to_string(),
                device_id: None,
                process_id: Some(pid),
                description: format!("Audio session ended: PID {}", pid),
            });
        }
    }

    Ok((events, current_device_ids, current_pids))
}
