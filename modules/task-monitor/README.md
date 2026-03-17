# Sovereign Task Monitor

Process monitor and system dashboard replacing Windows Task Manager.

## Features

- **Process List** — Sortable table with CPU%, memory, disk I/O, status
- **Process Tree** — Hierarchical parent-child view, collapsible
- **Process Actions** — Kill, kill tree, suspend, resume, set priority
- **Performance Graphs** — Real-time CPU, memory, disk, network (Canvas-based)
- **File Lock Finder** — "Who is locking this file?" via Restart Manager API
- **Service Management** — List, start, stop, restart Windows services

## Build

```
cd modules/task-monitor/src-tauri
cargo tauri build
```

## Run (dev)

```
cd modules/task-monitor/src-tauri
cargo tauri dev
```

## Views

| Tab | Description |
|---|---|
| Processes | Flat table or tree view of all running processes |
| Performance | Real-time graphs: CPU, memory, disk, network |
| Services | Windows services with start/stop/restart |
| File Locks | Find which process has a file locked |

## Configuration

Config file: `%APPDATA%\SovereignShell\task-monitor\config.toml`

See `config.default.toml` for all options.
