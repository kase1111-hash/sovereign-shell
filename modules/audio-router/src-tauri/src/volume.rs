//! Per-session and per-device volume control via WASAPI.

/// Set the master volume for a device (0.0 - 1.0).
pub fn set_device_volume(_device_id: &str, _level: f32) -> Result<(), String> {
    #[cfg(windows)]
    {
        set_device_volume_windows(_device_id, _level)
    }

    #[cfg(not(windows))]
    {
        Ok(()) // Stub
    }
}

/// Set mute state for a device.
pub fn set_device_mute(_device_id: &str, _muted: bool) -> Result<(), String> {
    #[cfg(windows)]
    {
        set_device_mute_windows(_device_id, _muted)
    }

    #[cfg(not(windows))]
    {
        Ok(())
    }
}

/// Set volume for a specific audio session by process ID (0.0 - 1.0).
pub fn set_session_volume(_pid: u32, _level: f32) -> Result<(), String> {
    #[cfg(windows)]
    {
        set_session_volume_windows(_pid, _level)
    }

    #[cfg(not(windows))]
    {
        Ok(())
    }
}

/// Set mute state for a specific audio session by process ID.
pub fn set_session_mute(_pid: u32, _muted: bool) -> Result<(), String> {
    #[cfg(windows)]
    {
        set_session_mute_windows(_pid, _muted)
    }

    #[cfg(not(windows))]
    {
        Ok(())
    }
}

// ── Windows implementations ─────────────────────────────────────────

#[cfg(windows)]
fn set_device_volume_windows(device_id: &str, level: f32) -> Result<(), String> {
    use windows::Win32::Media::Audio::*;
    use windows::Win32::System::Com::*;
    use windows::core::*;

    let level = level.clamp(0.0, 1.0);

    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);

        let enumerator: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)
                .map_err(|e| format!("Enumerator error: {e}"))?;

        let wide_id: Vec<u16> = device_id.encode_utf16().chain(std::iter::once(0)).collect();
        let device = enumerator
            .GetDevice(PCWSTR(wide_id.as_ptr()))
            .map_err(|e| format!("Device not found: {e}"))?;

        let endpoint_vol: IAudioEndpointVolume = device
            .Activate(CLSCTX_ALL, None)
            .map_err(|e| format!("Activate error: {e}"))?;

        endpoint_vol
            .SetMasterVolumeLevelScalar(level, std::ptr::null())
            .map_err(|e| format!("Set volume error: {e}"))?;

        Ok(())
    }
}

#[cfg(windows)]
fn set_device_mute_windows(device_id: &str, muted: bool) -> Result<(), String> {
    use windows::Win32::Media::Audio::*;
    use windows::Win32::System::Com::*;
    use windows::core::*;

    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);

        let enumerator: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)
                .map_err(|e| format!("Enumerator error: {e}"))?;

        let wide_id: Vec<u16> = device_id.encode_utf16().chain(std::iter::once(0)).collect();
        let device = enumerator
            .GetDevice(PCWSTR(wide_id.as_ptr()))
            .map_err(|e| format!("Device not found: {e}"))?;

        let endpoint_vol: IAudioEndpointVolume = device
            .Activate(CLSCTX_ALL, None)
            .map_err(|e| format!("Activate error: {e}"))?;

        endpoint_vol
            .SetMute(muted, std::ptr::null())
            .map_err(|e| format!("Set mute error: {e}"))?;

        Ok(())
    }
}

#[cfg(windows)]
fn set_session_volume_windows(pid: u32, level: f32) -> Result<(), String> {
    use windows::Win32::Media::Audio::*;
    use windows::Win32::System::Com::*;
    use windows::core::*;

    let level = level.clamp(0.0, 1.0);

    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);

        let enumerator: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)
                .map_err(|e| format!("Enumerator error: {e}"))?;

        let collection = enumerator
            .EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)
            .map_err(|e| format!("Enumerate error: {e}"))?;

        let count = collection.GetCount().unwrap_or(0);

        for i in 0..count {
            let device = match collection.Item(i) {
                Ok(d) => d,
                Err(_) => continue,
            };

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

                if control2.GetProcessId().unwrap_or(0) == pid {
                    let simple_vol: ISimpleAudioVolume = control
                        .cast()
                        .map_err(|e| format!("Cast error: {e}"))?;

                    simple_vol
                        .SetMasterVolume(level, std::ptr::null())
                        .map_err(|e| format!("Set volume error: {e}"))?;

                    return Ok(());
                }
            }
        }

        Err(format!("No audio session found for PID {}", pid))
    }
}

#[cfg(windows)]
fn set_session_mute_windows(pid: u32, muted: bool) -> Result<(), String> {
    use windows::Win32::Media::Audio::*;
    use windows::Win32::System::Com::*;
    use windows::core::*;

    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);

        let enumerator: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)
                .map_err(|e| format!("Enumerator error: {e}"))?;

        let collection = enumerator
            .EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)
            .map_err(|e| format!("Enumerate error: {e}"))?;

        let count = collection.GetCount().unwrap_or(0);

        for i in 0..count {
            let device = match collection.Item(i) {
                Ok(d) => d,
                Err(_) => continue,
            };

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

                if control2.GetProcessId().unwrap_or(0) == pid {
                    let simple_vol: ISimpleAudioVolume = control
                        .cast()
                        .map_err(|e| format!("Cast error: {e}"))?;

                    simple_vol
                        .SetMute(muted, std::ptr::null())
                        .map_err(|e| format!("Set mute error: {e}"))?;

                    return Ok(());
                }
            }
        }

        Err(format!("No audio session found for PID {}", pid))
    }
}
