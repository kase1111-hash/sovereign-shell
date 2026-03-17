# Sovereign Search Daemon

Headless file indexing and search daemon replacing Windows Search (WSearch).

## What It Does

- Recursively crawls configured directories and indexes file metadata + text content
- Watches for filesystem changes and updates the index incrementally
- Serves search queries over a named pipe IPC interface
- Uses SQLite FTS5 with porter stemming for ranked, full-text search

## Build

```
cd modules/search-daemon
cargo build --release
```

## Run

```
sovereign-search-daemon.exe
```

Set `RUST_LOG=debug` for verbose output.

## IPC Interface

Named pipe: `\\.\pipe\sovereign-shell-search-daemon`

### Search

```json
{"type": "search", "payload": {"query": "quarterly report", "max_results": 20, "file_types": [".xlsx", ".pdf"]}}
```

### Status

```json
{"type": "status", "payload": {}}
```

### Reindex

```json
{"type": "reindex", "payload": {}}
```

## Configuration

Config file: `%APPDATA%\SovereignShell\search-daemon\config.toml`

See `config.default.toml` for all options.

## Install

```powershell
.\install.ps1
```
