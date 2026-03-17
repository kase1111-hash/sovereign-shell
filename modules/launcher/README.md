# Sovereign Launcher

Keyboard-driven app launcher replacing the Windows Start Menu.

## Usage

Press `Alt+Space` to activate. Type to search. `Enter` to launch. `Escape` to dismiss.

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| Alt+Space | Toggle launcher |
| Arrow Up/Down | Navigate results |
| Enter | Launch selected app |
| Ctrl+Enter | Open containing folder |
| Tab | Cycle through results |
| Escape | Hide launcher |

## Build

```
cd modules/launcher/src-tauri
cargo build --release
```

## Install

```powershell
.\install.ps1
```

## Configuration

Config file: `%APPDATA%\SovereignShell\launcher\config.toml`

See `config.default.toml` for all options.
