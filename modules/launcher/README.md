# Sovereign Launcher

Keyboard-driven app launcher replacing the Windows Start Menu.

## Usage

Press `Alt+Space` to activate. Type to search. `Enter` to launch. `Escape` to dismiss.

### Calculator Mode

Type `=` followed by a math expression to evaluate it inline. Press `Enter` to copy the result to your clipboard.

Examples: `=2+3*4`, `=sqrt(144)`, `=sin(pi/2)`, `=log(1000)`

Supports: `+`, `-`, `*`, `/`, `%`, `^` (power), parentheses, and functions (`sqrt`, `sin`, `cos`, `tan`, `abs`, `ln`, `log`, `ceil`, `floor`, `round`, `min`, `max`, `pow`). Constants: `pi`, `e`, `tau`.

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
