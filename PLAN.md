# PLAN.md — Sovereign Shell Development Roadmap

**Author:** Kase — github.com/kase1111-hash  
**Version:** 1.0.0  
**Date:** 2026-03-12  
**Methodology:** Natural language programming via Claude Code sessions  

---

## How to Use This Document

This is the build bible. Each module section is written so that you can open a Claude Code session, point it at the relevant section, and start coding. Every section contains: what to build, why, what APIs to use, what the file structure should look like, what the UI should do, what to test, and what to skip for now.

Build order is strict. Dependencies flow downward. Don't jump ahead — each phase produces infrastructure the next phase consumes.

---

## Prerequisites — Local Dev Environment

Before any module work, the development machine needs:

```
# Rust toolchain
rustup default stable
rustup target add x86_64-pc-windows-msvc

# Tauri CLI
cargo install tauri-cli

# Node.js (for Tauri frontend builds)
# Install Node 18+ LTS via winget or direct download

# SQLite (bundled via rusqlite with bundled feature — no system install needed)

# Git
git init sovereign-shell
```

Every module uses a shared Tauri pattern (except search-daemon which is headless Rust). Establish the Tauri boilerplate once, then clone it per module.

### Shared Tauri Template

Create a reusable Tauri starter at `templates/tauri-module/` with:
- Minimal `src-tauri/` with `tauri.conf.json` configured for borderless window, single instance, system tray support
- Frontend scaffold using vanilla HTML/CSS/JS (no framework — keeps bundle tiny, keeps context budget small)
- Shared CSS variables for consistent dark theme across all modules
- IPC bridge boilerplate (Tauri `invoke` and `listen` patterns)

The template is NOT a dependency — it's a copy-paste starting point. Each module owns its copy and diverges freely.

---

## Shared Infrastructure

### Config Library (`shared/config/`)

A tiny Rust crate (~100 lines) used by all modules for reading/writing TOML config files.

```
shared/config/
├── Cargo.toml
├── src/
│   └── lib.rs
```

**What it does:**
- Reads `%APPDATA%\SovereignShell\<module-name>\config.toml`
- If file doesn't exist, writes a default config and returns it
- Typed deserialization via `serde` + `toml` crates
- Each module defines its own config struct; this crate provides the read/write/default plumbing

**API surface:**
```rust
pub fn load_or_default<T: Default + Serialize + DeserializeOwned>(module_name: &str) -> Result<T>;
pub fn save<T: Serialize>(module_name: &str, config: &T) -> Result<()>;
pub fn config_path(module_name: &str) -> PathBuf;
```

**Claude Code session prompt:** "Build a small Rust library crate at shared/config/ that loads and saves per-module TOML config files from %APPDATA%\SovereignShell\<module-name>\config.toml. Use serde and toml crates. Provide load_or_default, save, and config_path functions. Under 100 lines."

### IPC Protocol (`shared/ipc/`)

A tiny Rust crate for named pipe IPC between modules (search-daemon ↔ launcher, search-daemon ↔ explorer, notify-queue ↔ other modules).

```
shared/ipc/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── server.rs    # Named pipe server (for daemons)
│   └── client.rs    # Named pipe client (for consumers)
```

**Protocol:**
- Transport: Windows Named Pipes (`\\.\pipe\sovereign-shell-<module>`)
- Format: Newline-delimited JSON messages
- Pattern: Request-response (client sends JSON query, server responds with JSON result)
- Each message has a `type` field for routing and a `payload` field for data

**Message schema:**
```json
{"type": "search", "payload": {"query": "budget.xlsx", "max_results": 10}}
{"type": "result", "payload": {"hits": [{"path": "...", "score": 0.95, "snippet": "..."}]}}
{"type": "error", "payload": {"message": "Index not ready"}}
```

**Claude Code session prompt:** "Build a Rust library crate at shared/ipc/ for Windows named pipe IPC. Server side: create a named pipe, accept connections, read newline-delimited JSON, dispatch to a handler callback, write response. Client side: connect to pipe by name, send JSON query, read response. Use the windows crate for named pipe APIs. Keep it under 200 lines total."

### Shared UI Theme (`templates/theme.css`)

A single CSS file defining the visual language for all module frontends:

```css
:root {
    --bg-primary: #1a1a2e;
    --bg-secondary: #16213e;
    --bg-tertiary: #0f3460;
    --text-primary: #e0e0e0;
    --text-secondary: #a0a0a0;
    --accent: #00d4ff;
    --accent-hover: #00b8d9;
    --danger: #ff4757;
    --success: #2ed573;
    --warning: #ffa502;
    --border: #2a2a4a;
    --radius: 6px;
    --font-mono: 'Cascadia Code', 'Consolas', monospace;
    --font-sans: 'Segoe UI', system-ui, sans-serif;
    --transition: 150ms ease;
}
```

Dark theme only for v1. Light theme is a future nicety. Every module imports this file.

---

## Phase 1 — Launcher

**Priority:** 1 (highest daily friction)  
**Complexity:** Small  
**Estimated Claude Code sessions:** 3–5  
**Target status:** Alpha  

### What It Replaces
The Windows Start Menu — specifically the act of pressing a key, typing a name, and launching something.

### Architecture

```
modules/launcher/
├── manifest.toml              # Already exists
├── README.md                  # Already exists
├── install.ps1                # Registers hotkey, adds to startup
├── uninstall.ps1              # Removes startup entry, restores defaults
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json        # Borderless, always-on-top, starts hidden
│   ├── src/
│   │   ├── main.rs            # Tauri entry, hotkey registration, window toggle
│   │   ├── indexer.rs         # Scans Start Menu, PATH, custom dirs → SQLite
│   │   ├── search.rs          # FTS5 query engine with fuzzy matching
│   │   └── db.rs              # SQLite connection, schema, migrations
│   └── icons/
├── src/                       # Frontend
│   ├── index.html             # Single input field + results list
│   ├── style.css              # Imports theme.css, launcher-specific styles
│   └── main.js                # Key handling, Tauri invoke calls, result rendering
└── config.default.toml        # Default configuration
```

### Behavior Specification

**Activation:**
- Global hotkey: `Alt+Space` (configurable in config.toml)
- On press: window appears centered on active monitor, input field focused
- On press again (or `Escape`): window hides
- On focus loss: window hides

**Window:**
- Borderless, frameless, floating, always-on-top
- Width: 600px, height: dynamic (grows with results, max 500px)
- Centered horizontally, ~30% from top of screen
- Slight drop shadow, rounded corners
- Transparent background with blur effect if available (acrylic/mica via Tauri window vibrancy)

**Indexing (runs on startup and periodically):**
1. Scan all `.lnk` files in:
   - `%ProgramData%\Microsoft\Windows\Start Menu\Programs\`
   - `%APPDATA%\Microsoft\Windows\Start Menu\Programs\`
   - Any custom directories from config.toml
2. Scan all executables on `%PATH%`
3. For each entry, store in SQLite:
   - `name` (display name, extracted from .lnk or filename)
   - `path` (full path to executable or .lnk)
   - `keywords` (name tokens, parent folder name)
   - `launch_count` (incremented on each launch, used for ranking boost)
   - `last_launched` (timestamp)
   - `icon_path` (extracted from .lnk target or .exe)
4. Re-index every 5 minutes (configurable) using filesystem watcher for changes

**Search:**
- User types → frontend sends query string to Rust backend via Tauri invoke
- Backend runs FTS5 `MATCH` query with fuzzy prefix matching
- Results ranked by: FTS5 relevance score × (1 + 0.1 × launch_count) × recency_boost
- Return top 8 results
- Each result shows: icon, name, path (truncated), launch count badge

**Launch:**
- `Enter` launches the top result
- `Arrow keys` navigate results, `Enter` on highlighted result launches it
- `Ctrl+Enter` opens containing folder instead
- On launch: increment `launch_count`, update `last_launched`, hide window
- Launch via `ShellExecuteW` to respect .lnk targets, working directories, and elevation

**Config (config.default.toml):**
```toml
[hotkey]
modifier = "alt"
key = "space"

[indexing]
extra_dirs = []
refresh_interval_seconds = 300
max_results = 8

[appearance]
width = 600
position_from_top_pct = 30
```

### Session-by-Session Build Plan

**Session 1 — Tauri scaffold + hotkey + show/hide:**
- Initialize Tauri project from template
- Register global hotkey via `RegisterHotKey` Win32 API (called from Rust side)
- Toggle window visibility on hotkey press
- Borderless centered window with a single text input
- Goal: press Alt+Space, window appears, press Escape, window hides

**Session 2 — Indexer + database:**
- Create SQLite database at `%APPDATA%\SovereignShell\launcher\index.db`
- Write the indexer: scan Start Menu .lnk files, resolve targets, extract names
- Scan PATH for executables
- Populate FTS5 virtual table
- Background re-index on timer
- Goal: on startup, database is populated with all launchable apps

**Session 3 — Search + results UI:**
- Wire up the search: frontend sends keystrokes, backend queries FTS5
- Debounce input (100ms)
- Return results as JSON array to frontend
- Frontend renders results list with name, path, and launch count
- Arrow key navigation, Enter to launch
- ShellExecuteW for launching, launch_count tracking
- Goal: fully functional launcher — type, see results, launch apps

**Session 4 — Polish:**
- Icon extraction from .lnk / .exe targets (use `SHGetFileInfo` or `IExtractIcon`)
- Window blur/vibrancy effect
- Smooth show/hide animation (fade + slight vertical slide)
- Config file loading from config.toml
- Install/uninstall scripts

**Session 5 (optional) — Calculator mode:**
- If input starts with `=`, evaluate as math expression (use `meval` crate)
- Display result inline, Enter copies to clipboard
- Low effort, high utility addition

### Testing Checklist
- [ ] Alt+Space shows window on correct monitor
- [ ] Escape / focus loss hides window
- [ ] Typing produces results within 50ms
- [ ] All installed programs appear in results
- [ ] Enter launches correct application
- [ ] Launch count persists across restarts
- [ ] Ctrl+Enter opens containing folder
- [ ] Works with elevated (admin) applications
- [ ] Re-indexing picks up newly installed programs
- [ ] Config changes take effect after restart

---

## Phase 2 — Search Daemon

**Priority:** 2 (foundation for Explorer, enhances Launcher)  
**Complexity:** Medium  
**Estimated Claude Code sessions:** 4–6  
**Target status:** Alpha  

### What It Replaces
Windows Search service (`WSearch`) — the indexer behind the broken search in File Explorer and Start Menu.

### Why Before Explorer
The Explorer module depends on search-daemon for file search. Building the daemon first also lets us backfill the launcher with file search results (Phase 1 launcher only indexes apps; after this phase it can also surface files).

### Architecture

```
modules/search-daemon/
├── manifest.toml
├── README.md
├── install.ps1               # Installs as Windows Service or scheduled task
├── uninstall.ps1
├── src/
│   ├── main.rs               # Service entry point, daemon lifecycle
│   ├── indexer.rs             # Filesystem crawler, content extraction
│   ├── watcher.rs             # ReadDirectoryChangesW file change monitor
│   ├── db.rs                  # SQLite FTS5 schema, queries, migrations
│   ├── ipc_server.rs          # Named pipe server for query interface
│   └── config.rs              # Configuration loading
├── Cargo.toml
└── config.default.toml
```

### Behavior Specification

**Daemon lifecycle:**
- Runs as a background process (started via Task Scheduler or as a Windows Service)
- On first run: performs full index of configured directories
- After initial index: watches for changes and incrementally updates
- On shutdown: saves index state, closes cleanly

**Indexing strategy:**
1. **Metadata index (fast, everything):** For every file in watched directories, index:
   - File name (tokenized for FTS5)
   - Full path
   - Extension
   - Size
   - Modified timestamp
   - Parent directory name
2. **Content index (slow, selective):** For supported file types, extract and index text content:
   - `.txt`, `.md`, `.log`, `.csv` — read raw text
   - `.pdf` — extract via `pdf-extract` crate (best effort, skip on failure)
   - `.docx` — extract via `docx-rs` crate (unzip + XML parse)
   - `.html` — strip tags, index text content
   - Source code files (`.rs`, `.py`, `.js`, `.toml`, `.json`, `.yaml`, `.ps1`, `.bat`, `.sh`) — index as text
3. **Exclusions (hardcoded + configurable):**
   - Always skip: `node_modules/`, `.git/`, `target/`, `__pycache__/`, Windows system dirs
   - Configurable exclude patterns in config.toml
   - Max file size for content indexing: 10MB (configurable)

**FTS5 schema:**
```sql
CREATE VIRTUAL TABLE files_fts USING fts5(
    name,
    path,
    extension,
    content,
    parent_dir,
    tokenize='porter unicode61'
);

-- Metadata table (non-FTS, for filtering and display)
CREATE TABLE files_meta (
    id INTEGER PRIMARY KEY,
    path TEXT UNIQUE NOT NULL,
    name TEXT NOT NULL,
    extension TEXT,
    size INTEGER,
    modified INTEGER,        -- Unix timestamp
    content_indexed INTEGER DEFAULT 0,  -- 0 = metadata only, 1 = content indexed
    last_seen INTEGER        -- Last crawler pass that confirmed this file exists
);
```

**File watching:**
- Use `ReadDirectoryChangesW` (via `notify` crate on Windows) on each watched root
- On file create/modify: re-index that file
- On file delete: remove from index
- On file rename: update path in index
- Buffer changes for 500ms before acting (debounce rapid saves)

**IPC interface:**
- Named pipe: `\\.\pipe\sovereign-shell-search`
- Request types:

```json
// Search by filename/content
{"type": "search", "payload": {"query": "quarterly report", "max_results": 20, "file_types": [".xlsx", ".pdf"]}}

// Get indexing status
{"type": "status", "payload": {}}

// Force re-index
{"type": "reindex", "payload": {"path": "C:\\Users\\kase\\Documents"}}
```

- Response types:
```json
// Search results
{"type": "results", "payload": {"hits": [
    {"path": "C:\\...\\Q3-report.xlsx", "name": "Q3-report.xlsx", "score": 0.92, "snippet": "...", "size": 245000, "modified": 1710288000}
], "total": 1, "query_ms": 12}}

// Status
{"type": "status", "payload": {"indexed_files": 142853, "index_size_mb": 380, "watching_dirs": 3, "last_full_index": "2026-03-12T08:30:00Z", "state": "idle"}}
```

**Config (config.default.toml):**
```toml
[indexing]
watched_dirs = ["C:\\Users"]
exclude_patterns = ["node_modules", ".git", "target", "__pycache__", "AppData\\Local\\Temp"]
content_index_extensions = [".txt", ".md", ".log", ".csv", ".pdf", ".docx", ".html", ".rs", ".py", ".js", ".toml", ".json", ".yaml", ".ps1"]
max_content_size_mb = 10
reindex_interval_hours = 24

[performance]
batch_size = 500              # Files per indexing batch before yielding
index_throttle_ms = 10        # Pause between batches to avoid disk thrash
max_memory_mb = 256           # Soft limit — pause indexing if exceeded

[ipc]
pipe_name = "sovereign-shell-search"
```

### Session-by-Session Build Plan

**Session 1 — Database + crawler:**
- Set up Rust project (no Tauri — this is a headless daemon)
- Create SQLite database with FTS5 schema
- Write recursive directory crawler with exclusion patterns
- Index file metadata for a test directory
- Goal: run binary, it crawls a directory and populates SQLite

**Session 2 — Content extraction + FTS5 search:**
- Add text content extraction for supported file types
- Populate FTS5 virtual table with name + content
- Write query function: take a search string, return ranked results with snippets
- Goal: search "quarterly report" and get ranked file matches with context snippets

**Session 3 — File watcher + incremental updates:**
- Add `notify` crate for filesystem watching
- On change events: update/insert/delete affected files in index
- Debounce rapid changes (500ms window)
- Background indexer thread + watcher thread coordination
- Goal: create a new file, it appears in search results within 1 second

**Session 4 — IPC server:**
- Implement named pipe server using shared/ipc crate
- Wire up search, status, and reindex message handlers
- Write a simple CLI test client
- Goal: from a separate terminal, query the daemon via named pipe and get results

**Session 5 — Service installation + lifecycle:**
- Write install.ps1 to register as a scheduled task (runs on login, restarts on failure)
- Graceful shutdown on SIGTERM / service stop
- Save/restore index state across restarts
- Logging to `%APPDATA%\SovereignShell\search-daemon\daemon.log`
- Goal: daemon starts on login, survives user session, logs activity

**Session 6 (optional) — Launcher integration:**
- Update launcher's search to also query search-daemon via IPC
- Show file results below app results, visually separated
- Goal: launcher can find both apps AND files

### Testing Checklist
- [ ] Full index of user directory completes without crashing
- [ ] FTS5 search returns relevant results within 50ms for metadata, 200ms for content
- [ ] File watcher detects create/modify/delete/rename
- [ ] IPC server handles concurrent clients
- [ ] Content extraction works for .txt, .md, .pdf, .docx
- [ ] Exclusion patterns respected
- [ ] Memory usage stays under configured limit
- [ ] Index survives daemon restart
- [ ] Install/uninstall scripts work cleanly

---

## Phase 3 — File Explorer

**Priority:** 3 (second highest daily friction after launcher)  
**Complexity:** Large (most complex module)  
**Estimated Claude Code sessions:** 8–12  
**Target status:** Alpha  

### What It Replaces
`explorer.exe` as a file manager — not as the desktop shell (that's a bridge too far for v1).

### Architecture

```
modules/explorer/
├── manifest.toml
├── README.md
├── install.ps1
├── uninstall.ps1
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── src/
│   │   ├── main.rs            # Window management, tab state
│   │   ├── fs_ops.rs          # File operations: copy, move, delete, rename, create
│   │   ├── fs_read.rs         # Directory listing, file metadata, icon extraction
│   │   ├── archive.rs         # Zip/tar/7z read and extract
│   │   ├── search_client.rs   # IPC client to search-daemon
│   │   ├── terminal.rs        # PTY integration for embedded terminal
│   │   ├── bookmarks.rs       # Quick-access locations / favorites
│   │   └── batch.rs           # Batch rename, batch operations
├── src/
│   ├── index.html
│   ├── style.css
│   ├── app.js                 # Main app state, tab management
│   ├── components/
│   │   ├── sidebar.js         # Navigation tree, bookmarks, drives
│   │   ├── file-list.js       # Main file listing (list/grid/miller views)
│   │   ├── breadcrumb.js      # Path breadcrumb bar (editable)
│   │   ├── preview.js         # File preview pane (text, images, metadata)
│   │   ├── terminal.js        # Embedded terminal panel
│   │   ├── search-bar.js      # Search UI, queries search-daemon
│   │   ├── tab-bar.js         # Tab management
│   │   └── context-menu.js    # Right-click context menu
│   └── utils/
│       ├── keybindings.js     # Keyboard shortcut handler
│       └── drag-drop.js       # Drag and drop between panels and external
└── config.default.toml
```

### Behavior Specification

**Window layout (default, all panels resizable and toggleable):**
```
┌─[Tab Bar]──────────────────────────────────────────────┐
│ [Breadcrumb / Path Bar (editable)]            [Search] │
├────────┬───────────────────────────────┬───────────────┤
│        │                               │               │
│ Side   │   Main File Listing           │   Preview     │
│ bar    │   (list / grid / miller)      │   Pane        │
│        │                               │               │
│ Drives │                               │  (optional)   │
│ Favs   │                               │               │
│ Tree   │                               │               │
│        │                               │               │
├────────┴───────────────────────────────┴───────────────┤
│ [Terminal Pane (toggle with Ctrl+`)]                    │
├────────────────────────────────────────────────────────┤
│ Status: 142 items | 3 selected | 2.4 GB               │
└────────────────────────────────────────────────────────┘
```

**Tab system:**
- `Ctrl+T` new tab (opens home directory)
- `Ctrl+W` close tab
- `Ctrl+Tab` / `Ctrl+Shift+Tab` cycle tabs
- Middle-click folder opens in new tab
- Tabs show folder name + icon, truncated
- Drag tabs to reorder

**Views:**
- **List view** (default): filename, size, modified date, type. Sortable columns.
- **Grid view**: thumbnails for images, icons for everything else. Adjustable icon size.
- **Miller column view**: three-column cascade. Click folder in left column, contents appear in middle, click again, rightmost updates. Ideal for deep navigation.
- Toggle with `Ctrl+1` (list), `Ctrl+2` (grid), `Ctrl+3` (miller)

**File operations:**
- Copy/paste: `Ctrl+C`, `Ctrl+V` (with progress dialog for large ops)
- Cut: `Ctrl+X`
- Delete: `Delete` key → recycle bin, `Shift+Delete` → permanent with confirmation
- Rename: `F2` or slow double-click on name
- New folder: `Ctrl+Shift+N`
- New file: `Ctrl+N` → type selector (empty file, text file, etc.)
- Batch rename: select multiple → `Ctrl+F2` → pattern-based rename dialog
  - Pattern: `{name}_{counter:3}` → `photo_001.jpg`, `photo_002.jpg`
  - Regex replace mode
  - Live preview before applying

**Breadcrumb bar:**
- Shows path as clickable segments: `C: > Users > kase > Documents`
- Click any segment to navigate there
- Click the empty area or press `Ctrl+L` to switch to editable text mode (type a path, press Enter)

**Search integration:**
- `Ctrl+F` opens search bar
- Queries search-daemon via IPC
- Results appear in main file listing with match highlighting
- Search scope: current directory (default) or all indexed locations

**Terminal pane:**
- `Ctrl+`` toggles terminal pane at bottom
- Opens PowerShell (or configured shell) with CWD set to current directory
- When you navigate to a new folder, terminal CWD follows (configurable)
- Uses PTY via `conpty` on Windows (Tauri can host this via a pseudo-terminal bridge)

**Preview pane:**
- `Ctrl+P` or `Alt+P` toggles preview pane on right side
- Text files: syntax-highlighted preview (first 200 lines)
- Images: thumbnail preview with dimensions
- PDFs: first page render (if feasible) or metadata summary
- Audio/video: metadata (duration, codec, resolution)
- Everything else: file metadata (size, dates, attributes, hash)

**Context menu (right-click):**
- Open / Open With...
- Copy / Cut / Paste
- Rename
- Delete / Permanent Delete
- Copy Path (full path to clipboard)
- Copy Name
- Open in Terminal (opens terminal pane at that location)
- Properties (size, dates, attributes, hash)
- Archive: Create Zip / Extract Here

**Keyboard shortcuts (vim-influenced optional layer):**
```
hjkl or arrows    Navigate
Enter              Open / Enter directory
Backspace          Go up one directory
Space              Toggle selection
Ctrl+A             Select all
Ctrl+Shift+A       Invert selection
/                  Focus search
```

**Config (config.default.toml):**
```toml
[general]
default_view = "list"          # list | grid | miller
show_hidden_files = false
show_file_extensions = true
confirm_delete = true
terminal_follows_navigation = true
default_shell = "powershell"

[sidebar]
show_drives = true
bookmarks = ["C:\\Users\\kase\\Documents", "C:\\Users\\kase\\Desktop", "C:\\Users\\kase\\Downloads"]

[appearance]
icon_size_grid = 48
preview_pane_default = false
terminal_pane_default = false
```

### Session-by-Session Build Plan

**Session 1 — Window + basic file listing:**
- Initialize Tauri project
- Simple window with file list rendering
- Read directory contents via Rust (`std::fs::read_dir`)
- Display: name, size, modified date, type
- Navigate into directories (double-click / Enter)
- Breadcrumb path display
- Goal: open a window, see files, navigate directories

**Session 2 — Sidebar + navigation:**
- Drive list (enumerate volumes)
- Bookmarks / favorites list
- Directory tree (lazy-loaded — only expand on click)
- Back / Forward / Up navigation with history stack
- Breadcrumb click-to-navigate
- Goal: full navigation without typing paths

**Session 3 — Tab system:**
- Tab bar UI
- Tab state management (each tab has its own path, history, selection)
- Ctrl+T, Ctrl+W, Ctrl+Tab, middle-click
- Tab drag reorder
- Goal: multiple tabs, independent navigation per tab

**Session 4 — File operations:**
- Copy/cut/paste with progress tracking
- Delete (recycle bin + permanent)
- Rename (inline edit)
- New folder / new file
- Drag and drop within the explorer (move between panes)
- Goal: manage files without needing the old explorer

**Session 5 — Batch rename:**
- Multi-select → batch rename dialog
- Pattern-based: `{name}`, `{counter}`, `{ext}`, `{date}`
- Regex replace mode
- Live preview column showing before → after
- Goal: rename 50 files in one operation

**Session 6 — Archive handling:**
- Open .zip, .tar.gz, .7z as if they were folders (read-only browsing)
- Extract here / extract to folder
- Create archive from selection
- Uses `zip` and `tar` Rust crates; `7z` via command-line fallback
- Goal: no more right-click → Extract All wizard

**Session 7 — Search integration:**
- Search bar UI (`Ctrl+F`)
- IPC client connecting to search-daemon
- Results rendered in main file list with score/snippet
- Scope toggle: current folder / everywhere
- Goal: search "invoice" and find it instantly

**Session 8 — Terminal pane:**
- Embedded terminal using Windows ConPTY
- Toggle with `Ctrl+``
- CWD syncs with current navigation
- Resizable pane
- Goal: run `git status` without leaving the explorer

**Session 9 — Preview pane:**
- Text file preview with basic syntax highlighting
- Image thumbnail preview
- File metadata panel for everything else
- Toggle with `Ctrl+P`
- Goal: quick-look at files without opening them

**Session 10 — Views (grid + miller):**
- Grid view with thumbnails
- Miller column view with three-pane cascade
- View switching keybindings
- Goal: three ways to browse files

**Session 11 — Polish + install:**
- Icon extraction for file types
- Context menu (right-click)
- Keyboard shortcut system
- Config file loading
- Install/uninstall scripts
- File associations (optional — register as "Open with Sovereign Explorer")
- Goal: usable as daily driver file manager

### Testing Checklist
- [ ] Navigate to any directory without crashing
- [ ] All file operations work (copy, move, delete, rename)
- [ ] Tabs maintain independent state
- [ ] Batch rename produces correct results (preview matches outcome)
- [ ] Search returns results from search-daemon
- [ ] Terminal pane opens and accepts input
- [ ] Archives can be browsed and extracted
- [ ] Miller column view works for deep navigation
- [ ] Drag and drop between explorer and external applications
- [ ] Large directories (10,000+ files) render without lag

---

## Phase 4 — Task Monitor

**Priority:** 4  
**Complexity:** Medium  
**Estimated Claude Code sessions:** 5–7  
**Target status:** Alpha  

### What It Replaces
Task Manager (`taskmgr.exe`) — elevating it to Process Explorer depth as the default.

### Architecture

```
modules/task-monitor/
├── manifest.toml
├── README.md
├── install.ps1
├── uninstall.ps1
├── src-tauri/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs            # Window, update loop
│   │   ├── processes.rs       # Process enumeration, tree building
│   │   ├── system_stats.rs    # CPU, RAM, disk, GPU, network global stats
│   │   ├── file_locks.rs      # "Who is locking this file?" via RestartManager
│   │   ├── services.rs        # Service enumeration and dependency mapping
│   │   └── process_actions.rs # Kill, suspend, resume, set priority/affinity
├── src/
│   ├── index.html
│   ├── style.css
│   ├── app.js
│   ├── components/
│   │   ├── process-tree.js    # Hierarchical process view
│   │   ├── process-table.js   # Flat sortable process list
│   │   ├── system-graphs.js   # Real-time CPU/RAM/disk/GPU/network graphs
│   │   ├── file-lock-finder.js # UI for "find handle" feature
│   │   └── service-list.js    # Windows services with dependency tree
└── config.default.toml
```

### Behavior Specification

**Views (tabs along top):**

1. **Processes (default):**
   - Tree view: processes nested by parent-child relationship
   - Flat view toggle: sortable table (like classic Task Manager but with more columns)
   - Columns: Name, PID, CPU%, Memory, Disk I/O, Network I/O, GPU%, Threads, Handles, User, Path
   - Color coding: high CPU = warm colors, suspended = gray
   - Right-click: Kill, Kill Tree, Suspend, Resume, Set Priority, Set Affinity, Open File Location, Properties

2. **Performance:**
   - Real-time graphs (last 60 seconds): CPU, RAM, Disk read/write, Network send/receive, GPU (if available)
   - Per-core CPU breakdown
   - Memory composition: in use, available, cached, paged
   - Update interval: 1 second (configurable)

3. **Services:**
   - All Windows services: name, display name, status, startup type
   - Dependency tree: click a service → see what it depends on and what depends on it
   - Start / Stop / Restart (requires elevation)

4. **File Locks (the killer feature):**
   - "Which process has this file locked?"
   - Browse or type a file path
   - Uses `RmStartSession` / `RmRegisterResources` / `RmGetList` (Restart Manager API)
   - Shows: process name, PID, handle type
   - Option to kill the locking process directly
   - This alone justifies replacing Task Manager

**System tray:**
- Minimizes to tray (optional)
- Tray icon shows miniature CPU usage indicator
- Right-click tray: Show, Quick Kill (type process name → kill), Exit

**Config:**
```toml
[general]
update_interval_ms = 1000
default_view = "processes"
show_system_processes = false
confirm_kill = true

[tray]
minimize_to_tray = true
show_cpu_indicator = true
```

### Session-by-Session Build Plan

**Session 1 — Process enumeration + table:**
- Use `sysinfo` crate to enumerate all processes
- Display in a sortable table: name, PID, CPU%, memory
- Auto-refresh every 1 second
- Goal: see what's running with accurate resource usage

**Session 2 — Process tree + actions:**
- Build parent-child process tree from PID/PPID
- Render as collapsible tree UI
- Right-click: kill, kill tree, suspend, resume
- Goal: navigate process hierarchy, kill misbehaving processes

**Session 3 — Performance graphs:**
- Real-time line graphs for CPU, RAM, disk, network
- Use Canvas or SVG for rendering (keep it lightweight)
- Per-core CPU breakdown
- Memory composition breakdown
- Goal: live system performance dashboard

**Session 4 — File lock finder:**
- Input: file path (browse button or paste)
- Use Windows Restart Manager API to find locking processes
- Display results: process name, PID, handle info
- "Kill" button per locking process
- Goal: answer "why can't I delete this file?" in 3 seconds

**Session 5 — Services view:**
- Enumerate all Windows services via WMI or `EnumServicesStatusEx`
- Display with status, startup type
- Service dependency tree
- Start/Stop/Restart actions (with UAC elevation prompt)
- Goal: manage services without opening services.msc

**Session 6 — GPU stats + polish:**
- GPU utilization (NVML for NVIDIA, D3DKMT for generic)
- System tray integration
- Per-process GPU usage (if available)
- Config loading
- Install/uninstall scripts
- Goal: feature-complete task monitor

### Testing Checklist
- [ ] All running processes appear with correct resource usage
- [ ] Process tree correctly reflects parent-child relationships
- [ ] Kill / Kill Tree actually terminates processes
- [ ] Performance graphs update smoothly at 1-second intervals
- [ ] File lock finder correctly identifies locking processes
- [ ] Service start/stop works (with elevation)
- [ ] Handles 200+ processes without UI lag

---

## Phase 5 — Audio Router

**Priority:** 5  
**Complexity:** Medium-High (deep Windows audio API work)  
**Estimated Claude Code sessions:** 5–7  
**Target status:** Alpha  

### What It Replaces
Volume Mixer — and adds per-application audio routing that Windows has never offered.

### Key Challenge
Virtual audio device creation requires a kernel-mode audio driver. For v1, we can avoid this by focusing on per-app output device assignment (which WASAPI supports natively) and volume control. Virtual devices are a v2 feature requiring a signed miniport driver or leveraging an existing virtual audio cable driver.

### Architecture

```
modules/audio-router/
├── manifest.toml
├── README.md
├── install.ps1
├── uninstall.ps1
├── src-tauri/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs
│   │   ├── devices.rs         # Enumerate audio devices (playback + capture)
│   │   ├── sessions.rs        # Enumerate audio sessions (per-app)
│   │   ├── routing.rs         # Set per-app default device
│   │   ├── volume.rs          # Per-app volume control
│   │   ├── monitor.rs         # Real-time audio level metering
│   │   └── events.rs          # Device arrival/removal, session create/destroy
├── src/
│   ├── index.html
│   ├── style.css
│   ├── app.js
│   ├── components/
│   │   ├── device-list.js     # Audio output/input devices
│   │   ├── session-mixer.js   # Per-app volume sliders with routing
│   │   └── level-meters.js    # Real-time audio level visualization
└── config.default.toml
```

### Behavior Specification

**Main view — the mixer:**
```
┌─────────────────────────────────────────────────────┐
│ Output Devices                                      │
│  🔊 Speakers (Realtek)          [====|====] 80%     │
│  🎧 Headset (HyperX)           [======|==] 100%    │
├─────────────────────────────────────────────────────┤
│ Application Routing                                 │
│                                                     │
│  🎵 Spotify         → [Speakers ▾]  [===|=====] 60%│
│     ▐▐▐▐▐▐▐░░░░░░ (live meter)                     │
│                                                     │
│  🎮 Discord         → [Headset  ▾]  [======|==] 90%│
│     ▐▐▐▐░░░░░░░░░ (live meter)                     │
│                                                     │
│  🌐 Firefox         → [Speakers ▾]  [====|====] 70%│
│     ▐▐▐▐▐▐▐▐░░░░░ (live meter)                     │
│                                                     │
│  ⚙  System Sounds   → [Speakers ▾]  [==|======] 30%│
│     ▐░░░░░░░░░░░░ (live meter)                     │
└─────────────────────────────────────────────────────┘
```

**Features:**
- Enumerate all audio output and input devices
- Enumerate all active audio sessions (one per application that has audio)
- Per-session volume slider (uses `ISimpleAudioVolume`)
- Per-session output device selector (uses `IAudioSessionControl` + Windows 10 per-app audio routing)
- Live audio level meters per session (uses `IAudioMeterInformation`)
- Device default assignment (set which device is system default for playback/capture)
- Hot-plug handling: new devices or sessions appear/disappear automatically

**System tray quick access:**
- System tray icon with click-to-open
- Scroll on tray icon to adjust master volume
- Middle-click to mute/unmute

**Config:**
```toml
[general]
update_interval_ms = 50       # For level meters
show_inactive_sessions = false

[routing_presets]
# Named presets for quick switching
[routing_presets.gaming]
"discord.exe" = "Headset (HyperX)"
"*" = "Speakers (Realtek)"

[routing_presets.music]
"*" = "Speakers (Realtek)"
```

### Session-by-Session Build Plan

**Session 1 — Device + session enumeration:**
- Use `windows` crate to access Core Audio APIs
- Enumerate playback and capture devices via `IMMDeviceEnumerator`
- Enumerate active audio sessions via `IAudioSessionEnumerator`
- Display in UI: device list + per-app session list
- Goal: see all audio devices and which apps are producing audio

**Session 2 — Volume control:**
- Per-session volume sliders via `ISimpleAudioVolume`
- Master device volume via `IAudioEndpointVolume`
- Mute/unmute toggles
- Goal: control volume per-app from one panel

**Session 3 — Audio routing:**
- Per-session output device assignment
- Uses Windows 10 `AudioRoutingManager` or registry-based per-app defaults
- Dropdown device selector per session
- Goal: route Discord to headset, Spotify to speakers

**Session 4 — Live metering:**
- Audio level meters via `IAudioMeterInformation`
- Smooth animated bars per session
- Peak hold indicators
- 50ms update interval
- Goal: see real-time audio levels per application

**Session 5 — Events + hot-plug:**
- `IMMNotificationClient` for device arrival/removal
- `IAudioSessionNotification` for new session detection
- Auto-update UI when devices are plugged/unplugged or apps start/stop audio
- Goal: plug in headphones and it appears instantly

**Session 6 — Presets + tray + polish:**
- Routing presets (save/load from config)
- System tray integration
- Config file loading
- Install/uninstall scripts
- Goal: daily-driver audio management tool

### Testing Checklist
- [ ] All playback and capture devices listed correctly
- [ ] Per-app volume control works independently
- [ ] Audio routing changes take effect immediately
- [ ] Level meters reflect actual audio output in real time
- [ ] Hot-plug of USB audio devices updates UI
- [ ] New audio sessions appear when apps start playing audio
- [ ] Presets save and restore correctly
- [ ] No audio glitches or dropouts when changing routing

---

## Phase 6 — Network Panel

**Priority:** 6  
**Complexity:** Medium-High  
**Estimated Claude Code sessions:** 6–8  
**Target status:** Alpha  

### What It Replaces
The three-headed monster of Windows networking: Settings → Network, Control Panel → Network Connections, and `ncpa.cpl`.

### Architecture

```
modules/net-panel/
├── manifest.toml
├── README.md
├── install.ps1
├── uninstall.ps1
├── src-tauri/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs
│   │   ├── adapters.rs        # Network adapter enumeration + config
│   │   ├── connections.rs     # Active connections (TCP/UDP) with process attribution
│   │   ├── dns.rs             # DNS config + resolver test
│   │   ├── firewall.rs        # Windows Firewall rule management
│   │   ├── diagnostics.rs     # Ping, traceroute, DNS lookup tools
│   │   └── monitor.rs         # Real-time bandwidth per-adapter and per-process
├── src/
│   ├── index.html
│   ├── style.css
│   ├── app.js
│   ├── components/
│   │   ├── adapter-list.js    # Network adapters with config panels
│   │   ├── connection-table.js # Active connections (netstat-style but better)
│   │   ├── firewall-rules.js  # Firewall rule viewer/editor
│   │   ├── diagnostics.js     # Ping/traceroute/DNS lookup tools
│   │   └── bandwidth-graph.js # Real-time network usage graphs
└── config.default.toml
```

### Behavior Specification

**Views (tabs):**

1. **Adapters:**
   - All network interfaces: Ethernet, Wi-Fi, VPN, Loopback
   - Per adapter: IP (v4/v6), subnet, gateway, DNS, MAC, status, speed
   - Edit mode: change IP, DNS, gateway (applies via `netsh` or WMI)
   - Enable/disable adapters

2. **Connections:**
   - All active TCP/UDP connections (like `netstat -ano` but readable)
   - Columns: Local Address, Remote Address, State, PID, Process Name, bytes sent/received
   - Filter by process, remote host, or port
   - Right-click: kill connection, copy info, whois lookup

3. **Firewall:**
   - View all Windows Firewall rules (inbound + outbound)
   - Sortable, filterable table
   - Toggle rules on/off
   - Create new rules via a simple form (not the 8-step Windows wizard)
   - Import/export rule sets

4. **Diagnostics:**
   - Ping tool: target host, count, interval, live graph of RTT
   - Traceroute: hop-by-hop with geo lookup (optional)
   - DNS lookup: query any record type against any nameserver
   - Speed test: simple download/upload bandwidth estimate
   - All results displayed inline, no popup windows

5. **Bandwidth:**
   - Real-time per-adapter throughput graphs
   - Per-process network usage (top talkers)
   - Daily/weekly cumulative bandwidth tracker

**Config:**
```toml
[general]
update_interval_ms = 1000
show_loopback = false

[dns]
preferred_servers = ["1.1.1.1", "9.9.9.9"]

[diagnostics]
default_ping_count = 10
default_traceroute_max_hops = 30
```

### Session-by-Session Build Plan

**Session 1 — Adapter enumeration + display:**
- Use IP Helper API (`GetAdaptersAddresses`) to enumerate adapters
- Display all interfaces with IP, gateway, DNS, MAC, status
- Goal: one panel showing all network interfaces

**Session 2 — Active connections:**
- Use `GetExtendedTcpTable` / `GetExtendedUdpTable` for connection listing
- Process attribution via PID lookup
- Sortable/filterable table
- Auto-refresh
- Goal: see every network connection with process names

**Session 3 — Adapter configuration:**
- Edit IP, DNS, gateway via `netsh` or WMI calls
- Enable/disable adapters
- Apply button with validation
- Goal: configure network without opening Control Panel

**Session 4 — Firewall management:**
- Enumerate firewall rules via `INetFwPolicy2` (COM) or `netsh advfirewall`
- Display in sortable table
- Toggle rules, create new rules via simple form
- Goal: manage firewall rules without the Windows wizard

**Session 5 — Diagnostics tools:**
- Built-in ping with live RTT graph
- Traceroute with hop listing
- DNS lookup (A, AAAA, MX, TXT, NS records)
- Goal: diagnose network issues without opening cmd

**Session 6 — Bandwidth monitoring:**
- Per-adapter throughput (bytes/sec) via performance counters
- Per-process network usage via ETW or periodic connection delta
- Real-time graphs
- Goal: see who's using bandwidth

**Session 7 — Polish + install:**
- Config loading
- Per-process bandwidth history
- Export connection list / firewall rules
- Install/uninstall scripts
- Goal: complete networking panel

### Testing Checklist
- [ ] All adapters listed with correct info
- [ ] Active connections match `netstat -ano` output
- [ ] IP/DNS changes apply correctly and are reversible
- [ ] Firewall rules can be toggled and created
- [ ] Ping/traceroute/DNS tools produce correct results
- [ ] Bandwidth graphs update in real time
- [ ] Works with VPN adapters
- [ ] Handles adapter connect/disconnect events

---

## Phase 7 — Notification Queue

**Priority:** 7 (lowest friction but completes the shell)  
**Complexity:** Small-Medium  
**Estimated Claude Code sessions:** 3–5  
**Target status:** Alpha  

### What It Replaces
Action Center — the bloated notification panel full of tips, ads, and quick settings nobody asked for.

### Architecture

```
modules/notify-queue/
├── manifest.toml
├── README.md
├── install.ps1
├── uninstall.ps1
├── src-tauri/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs
│   │   ├── listener.rs        # Intercept Windows notifications
│   │   ├── queue.rs           # Notification storage + filtering
│   │   ├── rules.rs           # Per-app allow/block/priority rules
│   │   ├── ipc_server.rs      # Accept notifications from other SS modules
│   │   └── history.rs         # Searchable notification history (SQLite)
├── src/
│   ├── index.html
│   ├── style.css
│   ├── app.js
│   ├── components/
│   │   ├── toast.js           # Popup toast notification renderer
│   │   ├── queue-panel.js     # Full notification list panel
│   │   ├── rules-editor.js    # Per-app notification rules
│   │   └── history-search.js  # Search past notifications
└── config.default.toml
```

### Behavior Specification

**Toast notifications:**
- Small popup in the corner (configurable: top-right, bottom-right, etc.)
- Shows: app icon, title, body, timestamp
- Auto-dismiss after configurable duration (5 seconds default)
- Click to open source app or dismiss
- Silent mode: suppress all toasts, accumulate in queue

**Queue panel:**
- `Win+N` (or configurable hotkey) opens the full notification panel
- Sorted by recency
- Grouped by application
- Per-notification actions: dismiss, dismiss all from app, block app
- Clear all button

**Filtering rules:**
```toml
[rules.default]
action = "show"               # show | silent | block
duration_seconds = 5

[rules."Microsoft.Windows.Explorer"]
action = "block"               # Block all Explorer notifications (tips, etc.)

[rules."discord.exe"]
action = "show"
duration_seconds = 8
priority = "high"              # High priority = shows even in silent mode
```

**History:**
- All notifications stored in SQLite
- Searchable by app, title, body, date
- Retention: 30 days (configurable)
- Goal: "What was that notification I dismissed 3 days ago?"

**IPC interface:**
- Named pipe: `\\.\pipe\sovereign-shell-notify`
- Other Sovereign Shell modules can send notifications:
```json
{"type": "notify", "payload": {"title": "Search Complete", "body": "Indexed 142,853 files", "source": "search-daemon", "priority": "low"}}
```

### Interception approach:
- Use `Windows.UI.Notifications.Management.UserNotificationListener` (requires notification access capability)
- Alternatively, use UI Automation to intercept toast notifications
- Fallback: accessibility API hook on the notification area
- This is the trickiest technical challenge — Windows does not offer a clean API for intercepting third-party notifications. V1 may need to focus on suppressing Windows-generated notifications and providing the IPC channel for Sovereign Shell modules, with third-party notification interception as a v2 feature.

### Session-by-Session Build Plan

**Session 1 — Toast renderer + IPC server:**
- Tauri window configured as a small popup (corner-positioned, borderless, always-on-top)
- IPC server accepts notification messages from other modules
- Render toast: icon, title, body, auto-dismiss timer
- Goal: search-daemon can send a notification that appears as a toast

**Session 2 — Queue panel:**
- Full notification panel (separate window, hotkey-triggered)
- In-memory notification queue
- Group by source, sort by time
- Dismiss individual / dismiss all / dismiss by app
- Goal: see all notifications in one panel

**Session 3 — History + search:**
- SQLite storage for all notifications
- Searchable by text content, app name, date range
- Auto-purge after retention period
- Goal: find a notification from last week

**Session 4 — Rules engine + Windows integration:**
- Per-app rules (show/silent/block)
- Rules editor UI
- Attempt Windows notification interception via `UserNotificationListener`
- Suppress Windows tips/suggestions at the notification level
- Goal: block annoying notifications, keep important ones

**Session 5 — Polish:**
- Toast animation (slide in/out)
- Silent mode toggle (hotkey)
- System tray indicator (unread count badge)
- Config loading
- Install/uninstall scripts
- Goal: clean notification experience

### Testing Checklist
- [ ] Toasts appear and auto-dismiss
- [ ] IPC notifications from search-daemon render correctly
- [ ] Queue panel shows all accumulated notifications
- [ ] Dismiss actions work (single, all, by app)
- [ ] History search finds past notifications
- [ ] Per-app rules correctly filter/block/show
- [ ] Silent mode suppresses toasts but queues them
- [ ] 30-day history retention and auto-purge works

---

## Cross-Module Integration Points

Once individual modules reach alpha, these integration paths become available:

### Launcher + Search Daemon (Phase 2 completion)
- Launcher queries search-daemon for file results alongside app results
- File results appear in a separate section below app matches
- Opens file location or default app on select

### Explorer + Search Daemon (Phase 3)
- Explorer's `Ctrl+F` queries search-daemon
- Results replace file listing in the active tab
- Click result navigates to file location

### Explorer + Task Monitor (Phase 4)
- "Which process has this file locked?" accessible from Explorer's context menu
- Calls task-monitor's file lock finder and displays the result inline

### Notify Queue + All Modules (Phase 7)
- All modules can send operational notifications via IPC
- Search daemon: "Indexing complete"
- Explorer: "Copy complete — 2.4 GB moved"
- Task monitor: "Process killed: node.exe (PID 14232)"

### Launcher + Audio Router (Phase 5)
- Type "audio" in launcher to quick-open audio router
- Or surface audio device switching as a launcher action: "Switch to Headset"

---

## Version Milestones

### v0.1.0 — Scaffold (CURRENT)
All module directories created, manifests written, debloat script complete, governance documents in place.

### v0.2.0 — Launcher Alpha
Launcher is functional: hotkey activation, app indexing, FTS5 search, launch tracking.

### v0.3.0 — Search Foundation
Search daemon is running: filesystem indexing, content extraction, IPC server, launcher integration.

### v0.4.0 — Explorer Alpha
File explorer is usable as daily driver: tabs, all views, file operations, search integration, terminal pane.

### v0.5.0 — System Tools
Task monitor and audio router reach alpha.

### v0.6.0 — Network + Notifications
Net panel and notify queue reach alpha. All seven modules functional.

### v0.7.0 — Integration Pass
Cross-module integration points wired up. Modules talk to each other.

### v0.8.0 — Polish Pass
Config files standardized, install/uninstall scripts bulletproof, error handling hardened, edge cases addressed.

### v0.9.0 — Daily Driver
All modules tested as a complete shell replacement on a real workstation for 30 days. Bug fixes from real usage.

### v1.0.0 — Sovereign Shell
Stable release. Constitutional governance docs finalized. Every module documented. Ready for public repo.

---

## Notes for Claude Code Sessions

### Context management
- Always read the module's `manifest.toml` first — it's the contract
- Read this plan's section for the specific module being worked on
- Read `CONSTITUTION.md` Article II (Module Contract) for structural requirements
- Each session should produce a working increment — never leave the module in a broken state

### Commit discipline
- One commit per session minimum
- Commit messages describe INTENT, not just diff: "Add FTS5 search with fuzzy prefix matching and launch-count-weighted ranking" not "add search"
- Update `manifest.toml` version and `last_updated` on every session

### When stuck
- Windows API issues: check the `windows` crate docs at https://microsoft.github.io/windows-docs-rs/
- Tauri issues: check https://v2.tauri.app (use Tauri v2 for all modules)
- Audio APIs: WASAPI is well-documented on MSDN; the `windows` crate exposes it directly
- If an approach isn't working after 30 minutes, document the blocker in `known_limitations` and move on

### Rust crate preferences
- Windows APIs: `windows` crate (official Microsoft bindings)
- SQLite: `rusqlite` with `bundled` feature
- File watching: `notify` crate
- System info: `sysinfo` crate
- Serialization: `serde` + `toml` for config, `serde_json` for IPC
- HTTP (if ever needed): `reqwest` — but prefer no network dependencies
- Async runtime: `tokio` only if truly needed; prefer synchronous + threads for simplicity

---

*"Build the modules in the order that reduces your daily friction fastest. Every session should leave you with a tool that's better than what Windows shipped."*
