//! Real-time audio level metering via IAudioMeterInformation.

use serde::Serialize;

/// Audio level snapshot for all sessions.
#[derive(Debug, Clone, Serialize)]
pub struct LevelSnapshot {
    pub levels: Vec<SessionLevel>,
}

/// Level data for a single session.
#[derive(Debug, Clone, Serialize)]
pub struct SessionLevel {
    pub process_id: u32,
    pub peak: f32,       // 0.0 - 1.0
}

/// Get current peak levels for all active audio sessions.
pub fn get_peak_levels() -> Result<LevelSnapshot, String> {
    #[cfg(windows)]
    {
        get_peak_levels_windows()
    }

    #[cfg(not(windows))]
    {
        Ok(LevelSnapshot { levels: Vec::new() })
    }
}

#[cfg(windows)]
fn get_peak_levels_windows() -> Result<LevelSnapshot, String> {
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
        let mut levels = Vec::new();

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

                let pid = control2.GetProcessId().unwrap_or(0);
                if pid == 0 { continue; }

                let peak = control
                    .cast::<IAudioMeterInformation>()
                    .ok()
                    .and_then(|meter| meter.GetPeakValue().ok())
                    .unwrap_or(0.0);

                levels.push(SessionLevel {
                    process_id: pid,
                    peak,
                });
            }
        }

        Ok(LevelSnapshot { levels })
    }
}
