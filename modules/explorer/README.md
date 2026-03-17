# Sovereign Explorer

A tabbed file manager replacing Windows File Explorer.

## Features

- **Tabbed browsing** — Ctrl+T new tab, Ctrl+W close, Ctrl+Tab cycle
- **List & grid views** — Ctrl+1 list, Ctrl+2 grid
- **File operations** — Copy, cut, paste, delete (recycle bin + permanent), rename
- **Batch rename** — Pattern-based ({name}, {counter}, {date}) and regex modes
- **Archive support** — Browse, extract, and create zip archives
- **Search integration** — Queries the search daemon via IPC for instant file search
- **Preview pane** — Text files, images, metadata (Ctrl+P toggle)
- **Breadcrumb navigation** — Click segments or Ctrl+L for path editing
- **Sidebar** — Quick access bookmarks, drive list
- **Context menu** — Right-click for all operations
- **Keyboard-driven** — Arrow keys, Enter, Backspace, / for search

## Build

```
cd modules/explorer/src-tauri
cargo tauri build
```

## Run (dev)

```
cd modules/explorer/src-tauri
cargo tauri dev
```

## Keyboard Shortcuts

| Shortcut | Action |
|---|---|
| Ctrl+T | New tab |
| Ctrl+W | Close tab |
| Ctrl+Tab | Next tab |
| Ctrl+1 / 2 | List / Grid view |
| Ctrl+F or / | Search |
| Ctrl+L | Edit address bar |
| Ctrl+P | Toggle preview |
| Ctrl+` | Toggle terminal |
| Ctrl+H | Toggle hidden files |
| Ctrl+C/X/V | Copy/Cut/Paste |
| Delete | Delete to recycle bin |
| Shift+Delete | Permanent delete |
| F2 | Rename |
| Ctrl+Shift+N | New folder |
| Ctrl+N | New file |
| Ctrl+A | Select all |
| Backspace | Go up |
| Alt+Left/Right | Back/Forward |
| Enter | Open |

## Configuration

Config file: `%APPDATA%\SovereignShell\explorer\config.toml`

See `config.default.toml` for all options.
