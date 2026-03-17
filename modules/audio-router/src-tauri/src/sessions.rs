//! Audio session enumeration — per-application audio streams.
//!
//! Uses IAudioSessionEnumerator to discover active audio sessions
//! and map them to their owning processes.

use serde::Serialize;

/// An active audio session (one per application producing audio).
#[derive(Debug, Clone, Serialize)]
pub struct AudioSession {
    pub session_id: String,
    pub process_id: u32,
    pub process_name: String,
    pub display_name: String,
    pub volume: f32,        // 0.0 - 1.0
    pub is_muted: bool,
    pub peak_level: f32,    // 0.0 - 1.0 current audio level
    pub state: String,      // "active", "inactive", "expired"
    pub device_id: String,  // Which output device this session is on
}

/// Enumerate all active audio sessions across all devices.
pub fn enumerate_sessions() -> Result<Vec<AudioSession>, String> {
    #[cfg(windows)]
    {
        enumerate_sessions_windows()
    }

    #[cfg(not(windows))]
    {
        enumerate_sessions_stub()
    }
}

#[cfg(windows)]
fn enumerate_sessions_windows() -> Result<Vec<AudioSession>, String> {
    use windows::Win32::Media::Audio::*;
    use windows::Win32::System::Com::*;
    use windows::core::*;

    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);

        let enumerator: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)
                .map_err(|e| format!("Failed to create enumerator: {e}"))?;

        let collection = enumerator
            .EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)
            .map_err(|e| format!("Failed to enumerate endpoints: {e}"))?;

        let count = collection.GetCount().unwrap_or(0);
        let mut sessions = Vec::new();

        for i in 0..count {
            let device = match collection.Item(i) {
                Ok(d) => d,
                Err(_) => continue,
            };

            let device_id = device
                .GetId()
                .ok()
                .and_then(|id| id.to_string().ok())
                .unwrap_or_default();

            let manager: IAudioSessionManager2 = match device.Activate(CLSCTX_ALL, None) {
                Ok(m) => m,
                Err(_) => continue,
            };

            let session_enum = match manager.GetSessionEnumerator() {
                Ok(e) => e,
                Err(_) => continue,
            };

            let session_count = session_enum.GetCount().unwrap_or(0);

            for j in 0..session_count {
                let control = match session_enum.GetSession(j) {
                    Ok(c) => c,
                    Err(_) => continue,
                };

                let control2: IAudioSessionControl2 = match control.cast() {
                    Ok(c) => c,
                    Err(_) => continue,
                };

                let pid = control2.GetProcessId().unwrap_or(0);
                if pid == 0 { continue; } // Skip system session

                let state = match control2.GetState() {
                    Ok(AudioSessionStateActive) => "active",
                    Ok(AudioSessionStateInactive) => "inactive",
                    _ => "expired",
                };

                // Get process name
                let process_name = get_process_name(pid);

                // Get display name
                let display_name = control2
                    .GetDisplayName()
                    .ok()
                    .and_then(|s| s.to_string().ok())
                    .unwrap_or_default();

                let display_name = if display_name.is_empty() {
                    process_name.clone()
                } else {
                    display_name
                };

                // Get volume
                let simple_vol: ISimpleAudioVolume = match control.cast() {
                    Ok(v) => v,
                    Err(_) => continue,
                };

                let volume = simple_vol.GetMasterVolume().unwrap_or(1.0);
                let is_muted = simple_vol.GetMute().unwrap_or_default().as_bool();

                // Get peak level
                let peak_level = control
                    .cast::<IAudioMeterInformation>()
                    .ok()
                    .and_then(|meter| meter.GetPeakValue().ok())
                    .unwrap_or(0.0);

                let session_id = control2
                    .GetSessionIdentifier()
                    .ok()
                    .and_then(|s| s.to_string().ok())
                    .unwrap_or_else(|| format!("session-{}-{}", pid, j));

                sessions.push(AudioSession {
                    session_id,
                    process_id: pid,
                    process_name,
                    display_name,
                    volume,
                    is_muted,
                    peak_level,
                    state: state.to_string(),
                    device_id: device_id.clone(),
                });
            }
        }

        Ok(sessions)
    }
}

#[cfg(windows)]
fn get_process_name(pid: u32) -> String {
    use windows::Win32::System::Threading::*;
    use windows::Win32::Foundation::*;

    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid);
        if let Ok(handle) = handle {
            let mut buffer = [0u16; 260];
            let mut size = buffer.len() as u32;
            if QueryFullProcessImageNameW(handle, PROCESS_NAME_WIN32, &mut buffer, &mut size).is_ok() {
                let _ = CloseHandle(handle);
                let path = String::from_utf16_lossy(&buffer[..size as usize]);
                return path.rsplit('\\').next().unwrap_or(&path).to_string();
            }
            let _ = CloseHandle(handle);
        }
        format!("PID {}", pid)
    }
}

#[cfg(not(windows))]
fn enumerate_sessions_stub() -> Result<Vec<AudioSession>, String> {
    Ok(vec![
        AudioSession {
            session_id: "stub-1".to_string(),
            process_id: 1234,
            process_name: "firefox".to_string(),
            display_name: "Firefox".to_string(),
            volume: 0.7,
            is_muted: false,
            peak_level: 0.3,
            state: "active".to_string(),
            device_id: "default-output".to_string(),
        },
        AudioSession {
            session_id: "stub-2".to_string(),
            process_id: 5678,
            process_name: "spotify".to_string(),
            display_name: "Spotify".to_string(),
            volume: 0.6,
            is_muted: false,
            peak_level: 0.5,
            state: "active".to_string(),
            device_id: "default-output".to_string(),
        },
    ])
}
