//! Audio device enumeration via Core Audio (IMMDeviceEnumerator).

use serde::Serialize;

/// An audio endpoint device (playback or capture).
#[derive(Debug, Clone, Serialize)]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
    pub device_type: String,   // "playback" or "capture"
    pub is_default: bool,
    pub is_enabled: bool,
    pub volume: f32,           // 0.0 - 1.0
    pub is_muted: bool,
}

/// Enumerate all audio devices.
pub fn enumerate_devices() -> Result<Vec<AudioDevice>, String> {
    #[cfg(windows)]
    {
        enumerate_devices_windows()
    }

    #[cfg(not(windows))]
    {
        enumerate_devices_stub()
    }
}

#[cfg(windows)]
fn enumerate_devices_windows() -> Result<Vec<AudioDevice>, String> {
    use windows::Win32::Media::Audio::*;
    use windows::Win32::System::Com::*;
    use windows::core::*;

    unsafe {
        // Initialize COM
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);

        let enumerator: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)
                .map_err(|e| format!("Failed to create device enumerator: {e}"))?;

        let mut devices = Vec::new();

        // Get default devices for comparison
        let default_playback = enumerator
            .GetDefaultAudioEndpoint(eRender, eConsole)
            .ok()
            .and_then(|d| d.GetId().ok())
            .map(|id| id.to_string().unwrap_or_default());

        let default_capture = enumerator
            .GetDefaultAudioEndpoint(eCapture, eConsole)
            .ok()
            .and_then(|d| d.GetId().ok())
            .map(|id| id.to_string().unwrap_or_default());

        // Enumerate playback devices
        if let Ok(collection) = enumerator.EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE) {
            if let Ok(count) = collection.GetCount() {
                for i in 0..count {
                    if let Ok(device) = collection.Item(i) {
                        if let Some(info) = device_info(&device, "playback", &default_playback) {
                            devices.push(info);
                        }
                    }
                }
            }
        }

        // Enumerate capture devices
        if let Ok(collection) = enumerator.EnumAudioEndpoints(eCapture, DEVICE_STATE_ACTIVE) {
            if let Ok(count) = collection.GetCount() {
                for i in 0..count {
                    if let Ok(device) = collection.Item(i) {
                        if let Some(info) = device_info(&device, "capture", &default_capture) {
                            devices.push(info);
                        }
                    }
                }
            }
        }

        Ok(devices)
    }
}

#[cfg(windows)]
unsafe fn device_info(
    device: &windows::Win32::Media::Audio::IMMDevice,
    device_type: &str,
    default_id: &Option<String>,
) -> Option<AudioDevice> {
    use windows::Win32::Media::Audio::*;
    use windows::Win32::UI::Shell::PropertiesSystem::*;
    use windows::Win32::Devices::FunctionDiscovery::*;

    let id = device.GetId().ok()?.to_string().unwrap_or_default();
    let is_default = default_id.as_ref().map(|d| d == &id).unwrap_or(false);

    // Get friendly name from property store
    let name = device
        .OpenPropertyStore(STGM_READ)
        .ok()
        .and_then(|store| {
            store
                .GetValue(&PKEY_Device_FriendlyName)
                .ok()
                .and_then(|prop| {
                    // Extract string from PROPVARIANT
                    prop.Anonymous.Anonymous.Anonymous.pwszVal
                        .to_string()
                        .ok()
                })
        })
        .unwrap_or_else(|| format!("Device {}", &id[..8.min(id.len())]));

    // Get volume and mute state
    let (volume, is_muted) = device
        .Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None)
        .ok()
        .map(|vol| {
            let level = vol.GetMasterVolumeLevelScalar().unwrap_or(1.0);
            let muted = vol.GetMute().unwrap_or_default().as_bool();
            (level, muted)
        })
        .unwrap_or((1.0, false));

    Some(AudioDevice {
        id,
        name,
        device_type: device_type.to_string(),
        is_default,
        is_enabled: true,
        volume,
        is_muted,
    })
}

#[cfg(not(windows))]
fn enumerate_devices_stub() -> Result<Vec<AudioDevice>, String> {
    // On non-Windows, return placeholder devices for development
    Ok(vec![
        AudioDevice {
            id: "default-output".to_string(),
            name: "Default Output".to_string(),
            device_type: "playback".to_string(),
            is_default: true,
            is_enabled: true,
            volume: 0.8,
            is_muted: false,
        },
        AudioDevice {
            id: "default-input".to_string(),
            name: "Default Input".to_string(),
            device_type: "capture".to_string(),
            is_default: true,
            is_enabled: true,
            volume: 1.0,
            is_muted: false,
        },
    ])
}

/// Set the system default playback or capture device.
pub fn set_default_device(_device_id: &str) -> Result<(), String> {
    // Windows 10+ doesn't expose a public API for this.
    // Users typically use the Settings app. This would require
    // IPolicyConfig (undocumented COM interface) or registry manipulation.
    Err("Setting default device requires IPolicyConfig — not yet implemented".to_string())
}
