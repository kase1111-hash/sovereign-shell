# Sovereign Shell

**A modular replacement layer for the Windows 10 desktop experience.**

Sovereign Shell strips out the telemetry, advertising, and bloatware baked into Windows 10, then replaces the weakest built-in tools with fast, transparent, user-owned modules — each maintained through [Claude Code](https://docs.anthropic.com/en/docs/claude-code) sessions.

This is the manual version of the Anthropic Box: a human-AI maintained operating environment where every component can be understood, updated, and replaced by a single developer working with AI-assisted tooling.

---

## Philosophy

An operating system should serve its user, not its vendor. See [CONSTITUTION.md](CONSTITUTION.md) for the full governance kernel.

**Core principles:**
- User sovereignty — nothing phones home without explicit consent
- Local-first — every module works fully offline
- Transparency — no invisible background behavior
- Modularity — any module can be removed without breaking the rest
- AI-maintainable — every module fits in a single Claude Code context window

---

## Repository Structure

```
sovereign-shell/
├── CONSTITUTION.md          # Governance kernel — principles, contracts, methodology
├── README.md                # You are here
├── manifests/               # Cross-module configuration (future)
├── modules/
│   ├── launcher/            # Keyboard-driven app/file launcher (replaces Start Menu)
│   ├── explorer/            # Tabbed file explorer with terminal (replaces explorer.exe)
│   ├── search-daemon/       # Local FTS5 search indexer (replaces Windows Search)
│   ├── task-monitor/        # Process explorer + system monitor (replaces Task Manager)
│   ├── audio-router/        # Per-app audio routing (replaces Volume Mixer)
│   ├── net-panel/           # Unified network config + diagnostics (replaces Network Settings)
│   └── notify-queue/        # Filterable notification queue (replaces Action Center)
├── scripts/
│   ├── debloat.ps1          # Phase 1: Strip telemetry, ads, bloatware
│   └── install-modules.ps1  # Module installer/registrar
└── docs/                    # Design docs, architecture notes
```

---

## Quick Start

### 1. Debloat Windows

Run from an elevated PowerShell:

```powershell
# Preview changes without modifying anything
.\scripts\debloat.ps1 -DryRun

# Execute debloat (creates restore point first)
.\scripts\debloat.ps1

# Nuclear option — also disables Defender telemetry
.\scripts\debloat.ps1 -Aggressive
```

### 2. Check Module Status

```powershell
.\scripts\install-modules.ps1 -List
```

### 3. Install Modules

```powershell
# Install all modules at alpha status or above
.\scripts\install-modules.ps1

# Install a specific module
.\scripts\install-modules.ps1 -Module launcher
```

---

## Module Build Priority

| Priority | Module         | Replaces              | Stack        | Status   |
|----------|----------------|-----------------------|--------------|----------|
| 1        | launcher       | Start Menu            | Tauri + Rust | Scaffold |
| 2        | explorer       | File Explorer         | Tauri + Rust | Scaffold |
| 3        | search-daemon  | Windows Search        | Rust native  | Scaffold |
| 4        | task-monitor   | Task Manager          | Tauri + Rust | Scaffold |
| 5        | audio-router   | Volume Mixer          | Tauri + Rust | Scaffold |
| 6        | net-panel      | Network Settings      | Tauri + Rust | Scaffold |
| 7        | notify-queue   | Action Center         | Tauri + Rust | Scaffold |

---

## Update Methodology

This is not a self-updating system. Modules evolve through deliberate human-AI collaboration:

1. Identify what needs to change
2. Open a Claude Code session pointed at the module
3. Claude Code reads the manifest and source, understands the full module
4. Iterate together — code, test, refine
5. Update the manifest, bump the version, commit

See [CONSTITUTION.md — Article IV](CONSTITUTION.md#article-iv--update-methodology) for the full protocol.

---

## Tech Stack

- **Rust** — Module backends, system API integration, performance-critical paths
- **Tauri** — Desktop UI framework (Rust backend + web frontend, ~10x lighter than Electron)
- **SQLite FTS5** — Local full-text search indexing
- **PowerShell** — System configuration, debloat scripts, module installation
- **TOML** — Module manifests and user configuration

---

## Requirements

- Windows 10 (any edition, 21H2+)
- PowerShell 5.1+ (ships with Windows)
- Rust toolchain (for building modules): [rustup.rs](https://rustup.rs)
- Node.js 18+ (for Tauri frontend builds)

---

## Author

**Kase** — [github.com/kase1111-hash](https://github.com/kase1111-hash)  
True North Construction LLC  
Portland/Salem, Oregon

Built with the Digital Tractor philosophy: own your tools, own your infrastructure, own your workflow.

---

## License

MIT — Do what you want with it. Own it.
