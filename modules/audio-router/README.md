# Sovereign Audio Router

Per-application audio routing and volume control replacing Windows Volume Mixer.

## Features

- **Device Enumeration** — All playback and capture devices with volume/mute
- **Per-App Volume** — Independent volume sliders per audio session
- **Audio Routing** — Route each app to a different output device (dropdown selector)
- **Live Level Meters** — Real-time audio level visualization with peak hold
- **Hot-Plug Detection** — Poll-based device/session change detection
- **Routing Presets** — Save and load named routing configurations

## Build

```
cd modules/audio-router/src-tauri
cargo tauri build
```

## Run (dev)

```
cd modules/audio-router/src-tauri
cargo tauri dev
```

## Windows Audio APIs Used

- `IMMDeviceEnumerator` — Device enumeration
- `IAudioSessionManager2` / `IAudioSessionEnumerator` — Per-app session discovery
- `ISimpleAudioVolume` — Per-session volume control
- `IAudioEndpointVolume` — Device-level volume control
- `IAudioMeterInformation` — Real-time audio level metering
- `IAudioSessionControl2` — Session metadata (PID, display name, state)

## Configuration

Config file: `%APPDATA%\SovereignShell\audio-router\config.toml`

See `config.default.toml` for all options.
