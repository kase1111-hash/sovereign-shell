//! Windows notification listener.
//!
//! Attempts to intercept system notifications via `UserNotificationListener`.
//! V1 focuses on the IPC channel for Sovereign Shell modules; native Windows
//! notification interception is best-effort and platform-gated.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemNotification {
    pub app_id: String,
    pub title: String,
    pub body: String,
    pub timestamp: String,
}

/// Attempt to register for Windows notification access.
/// Returns `true` if the platform supports it and access was granted.
#[cfg(windows)]
pub fn request_notification_access() -> Result<bool, String> {
    // UserNotificationListener requires notification access capability
    // which is only available to UWP/packaged apps. For a Win32 Tauri app,
    // we rely on the IPC channel for v1 and document this limitation.
    log::info!("Windows notification listener: UWP UserNotificationListener not available for Win32 apps");
    log::info!("Using IPC channel for Sovereign Shell module notifications (v1)");
    Ok(false)
}

#[cfg(not(windows))]
pub fn request_notification_access() -> Result<bool, String> {
    log::info!("Notification listener not available on this platform");
    Ok(false)
}

/// Poll for new system notifications.
/// V1: returns empty — native interception is a v2 feature.
/// All notifications come through the IPC channel instead.
#[cfg(windows)]
pub fn poll_system_notifications() -> Vec<SystemNotification> {
    Vec::new()
}

#[cfg(not(windows))]
pub fn poll_system_notifications() -> Vec<SystemNotification> {
    Vec::new()
}
