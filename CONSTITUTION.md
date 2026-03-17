# CONSTITUTION.md — Sovereign Shell Governance Kernel

**Project:** Sovereign Shell  
**Author:** Kase — True North Construction LLC / github: kase1111-hash  
**Revision:** 1.0.0  
**Date:** 2026-03-12  
**License:** MIT  

---

## Preamble

This project exists because an operating system should serve its user, not its vendor. Windows 10 ships with advertising in the Start Menu, telemetry pipelines that cannot be fully disabled, forced update cycles that override user intent, and redundant interfaces that exist only as migration funnels toward Microsoft services.

Sovereign Shell is a modular replacement layer for the Windows 10 desktop experience. Each component is an independent, self-contained module that can be understood, maintained, updated, and replaced by a single developer working with AI-assisted tooling (Claude Code). The system is designed to be manually evolved — not auto-updated from a remote authority.

This document is the governance kernel. It defines the principles every module must follow, the contract every module must honor, and the methodology by which the system is maintained.

---

## Article I — Foundational Principles

### §1.1 User Sovereignty
The user is the sole authority over what runs on their machine. No module shall phone home, collect telemetry, display advertising, or communicate with any remote service without explicit, per-instance user consent.

### §1.2 Local-First Operation
Every module must function fully offline. Network features are additive, never required. Data stays on disk in human-readable or inspectable formats.

### §1.3 Transparency
No module shall perform actions invisible to the user. Background processes must be discoverable. Configuration must be editable in plain text. Logs must be human-readable.

### §1.4 Modularity
Each module is a sovereign unit. No module may create hard dependencies on another module. Communication between modules occurs only through documented IPC contracts. Any module can be removed without breaking the rest of the system.

### §1.5 Maintainability by AI-Assisted Workflow
Every module must be small enough that its full source can fit within a single Claude Code context window. Documentation is written in natural language as the primary maintenance interface. The intent of the code is as important as the code itself.

### §1.6 No Vendor Lock-In
No module shall depend on proprietary services, subscription APIs, or closed-source libraries where an open alternative exists. Dependencies are minimized and vendored where practical.

---

## Article II — Module Contract

Every module in the `modules/` directory must satisfy the following contract:

### §2.1 Manifest File
Each module must contain a `manifest.toml` at its root with the following fields:

```toml
[module]
name = "module-name"
version = "0.1.0"
status = "scaffold | development | alpha | beta | stable"
description = "One-line plain English description of what this module does."

[purpose]
narrative = """
A natural language paragraph explaining why this module exists,
what Windows default it replaces, and what design philosophy it follows.
"""

[architecture]
language = "rust | typescript | python | powershell"
framework = "tauri | electron | native | cli"
entry_point = "src/main.rs"
ipc_protocol = "named-pipe | http-local | stdin-stdout | none"

[boundaries]
depends_on = []           # Other sovereign-shell modules (should be empty or minimal)
system_apis = []          # Windows APIs used (Win32, COM, WMI, etc.)
external_deps = []        # Third-party crates/packages

[maintenance]
last_updated = "2026-03-12"
last_updated_by = "claude-code"
context_budget = "small | medium | large"  # How much context Claude Code needs
known_limitations = []
```

### §2.2 README.md
Each module must contain a README written for a human (or Claude Code) encountering it for the first time. It must explain: what the module does, how to build it, how to install it, how to configure it, and what Windows component it replaces.

### §2.3 Source Containment
All module source lives within its directory. No reaching into sibling module directories. Shared utilities, if ever needed, go in a `shared/` directory at the repo root and are treated as vendored code, not a living dependency.

### §2.4 Configuration
User-facing configuration must be a single TOML or JSON file in a predictable location (`%APPDATA%\SovereignShell\<module-name>\config.toml`). Sane defaults ship with every module. No config file required for basic operation.

### §2.5 Install / Uninstall
Each module must provide:
- `install.ps1` — Registers the module, sets up autostart if applicable, creates config directory
- `uninstall.ps1` — Cleanly removes all traces, restores Windows default if applicable

---

## Article III — The Debloat Layer

Before any custom module is installed, the base Windows 10 installation must be stripped of hostile components. The debloat script (`scripts/debloat.ps1`) operates in phases:

### Phase 0 — Snapshot
Create a system restore point before any changes.

### Phase 1 — Telemetry and Advertising
Disable telemetry services, remove advertising IDs, disable Cortana, remove "suggested apps" from Start Menu, disable tips and feedback notifications.

### Phase 2 — Bloatware Removal
Remove pre-installed UWP apps (Xbox, Mail, Calendar, Maps, Weather, News, Solitaire, Skype, etc.). Preserve Windows Calculator, Snipping Tool, and Notepad as fallbacks until custom modules replace them.

### Phase 3 — Service Hardening
Disable non-essential services: Connected User Experiences, WAP Push, Remote Registry, diagnostics services. Set Windows Update to notify-only (no forced restarts).

### Phase 4 — Privacy Lockdown
Disable activity history, timeline, cloud clipboard sync, handwriting data sharing, speech data sharing. Configure Windows Firewall to block known Microsoft telemetry endpoints.

---

## Article IV — Update Methodology

### §4.1 The Human-AI Maintenance Loop
This system is not self-updating. Updates happen through a deliberate human-AI collaboration cycle:

1. **Human identifies need** — A bug, a missing feature, a Windows update that broke something
2. **Human opens Claude Code session** — Points it at the relevant module directory
3. **Claude Code reads the manifest and source** — Understands the module in full context
4. **Collaborative iteration** — Changes are made, tested, committed
5. **Manifest updated** — Version bumped, `last_updated` set, `known_limitations` revised
6. **Git commit** — With a natural language commit message describing intent, not just diff

### §4.2 Version Discipline
Modules follow semver: `MAJOR.MINOR.PATCH`
- PATCH: Bug fixes, minor adjustments
- MINOR: New features, backward compatible
- MAJOR: Breaking changes to config format, IPC contract, or user-facing behavior

### §4.3 No Silent Changes
Every change to a module must be reflected in its manifest and committed to git. There is no mechanism for modules to update themselves.

---

## Article V — The Long Game

Sovereign Shell is not a weekend project. It is a living system that grows module by module, maintained session by session through Claude Code. The build order follows friction — replace the most painful Windows defaults first, leave the tolerable ones for later.

### Priority Queue (Initial)
1. Launcher (replace Start Menu)
2. File Explorer (replace explorer.exe shell)
3. Search Daemon (replace Windows Search / Bing injection)
4. Task Monitor (replace Task Manager)
5. Audio Router (replace Volume Mixer)
6. Network Panel (replace Network Settings)
7. Notification Queue (replace Action Center)

### Design North Star
If a module cannot be fully understood by reading its manifest and skimming its source in a single sitting, it is too complex. Split it.

---

## Article VI — Amendments

This constitution is a living document. Amendments are made by the project author through the same Claude Code workflow used for modules. Each amendment is a git commit with a clear rationale in the commit message.

The constitution governs the project. The project does not govern the constitution.

---

*"A tool should be owned by the hand that holds it."*  
*— Digital Tractor Philosophy*
