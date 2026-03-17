// Sovereign Explorer — Main Application Logic.
// Manages tabs, navigation history, and orchestrates all components.

const App = (() => {
    const { invoke } = window.__TAURI__.core;
    const { getCurrentWindow } = window.__TAURI__.window;

    let tabs = [];
    let activeTabId = null;
    let nextTabId = 1;
    let viewMode = 'list'; // 'list' | 'grid'

    // ── Tab State ────────────────────────────────────────────────────

    function createTab(path) {
        const tab = {
            id: nextTabId++,
            path: path,
            history: [path],
            historyIndex: 0,
            selection: [],
        };
        tabs.push(tab);
        return tab;
    }

    function getActiveTab() {
        return tabs.find(t => t.id === activeTabId);
    }

    // ── Navigation ──────────────────────────────────────────────────

    async function navigateTo(path) {
        const tab = getActiveTab();
        if (!tab) return;

        try {
            const listing = await invoke('list_directory', { path });
            tab.path = listing.path;

            // Update history
            if (tab.history[tab.historyIndex] !== listing.path) {
                tab.history = tab.history.slice(0, tab.historyIndex + 1);
                tab.history.push(listing.path);
                tab.historyIndex = tab.history.length - 1;
            }

            renderAll(listing);
        } catch (e) {
            console.error('Navigate error:', e);
        }
    }

    function goBack() {
        const tab = getActiveTab();
        if (!tab || tab.historyIndex <= 0) return;
        tab.historyIndex--;
        navigateToHistoryEntry(tab);
    }

    function goForward() {
        const tab = getActiveTab();
        if (!tab || tab.historyIndex >= tab.history.length - 1) return;
        tab.historyIndex++;
        navigateToHistoryEntry(tab);
    }

    function goUp() {
        const tab = getActiveTab();
        if (!tab) return;
        const parts = tab.path.replace(/\\/g, '/').split('/').filter(Boolean);
        if (parts.length <= 1) return;
        parts.pop();
        let parent = parts.join('/');
        // Windows drive handling
        if (parent.match(/^[A-Za-z]:$/)) parent += '\\';
        else if (!parent.startsWith('/')) parent = '/' + parent;
        navigateTo(parent);
    }

    async function navigateToHistoryEntry(tab) {
        const path = tab.history[tab.historyIndex];
        try {
            const listing = await invoke('list_directory', { path });
            tab.path = listing.path;
            renderAll(listing);
        } catch (e) {
            console.error('History navigate error:', e);
        }
    }

    // ── Rendering ───────────────────────────────────────────────────

    function renderAll(listing) {
        const tab = getActiveTab();

        TabBar.render(tabs, activeTabId, {
            onSelect: switchTab,
            onClose: closeTab,
            onNew: newTab,
        });

        Breadcrumb.render(tab.path, navigateTo);

        FileList.render(listing.entries, viewMode, {
            onNavigate: navigateTo,
            onOpen: openFile,
            onSelectionChange: onSelectionChange,
            onContextMenu: showContextMenu,
        });

        Sidebar.render(tab.path, navigateTo);

        updateNavButtons(tab);
        updateStatusBar(listing);
    }

    function updateNavButtons(tab) {
        document.getElementById('btn-back').disabled = tab.historyIndex <= 0;
        document.getElementById('btn-forward').disabled = tab.historyIndex >= tab.history.length - 1;
    }

    function updateStatusBar(listing) {
        document.getElementById('status-items').textContent = `${listing.total_items} items`;
        document.getElementById('status-size').textContent = formatSize(listing.total_size);
    }

    // ── Tab Operations ──────────────────────────────────────────────

    async function newTab() {
        const home = await invoke('get_home_dir');
        const tab = createTab(home);
        activeTabId = tab.id;
        navigateTo(home);
    }

    function switchTab(tabId) {
        activeTabId = tabId;
        const tab = getActiveTab();
        if (tab) navigateTo(tab.path);
    }

    function closeTab(tabId) {
        const index = tabs.findIndex(t => t.id === tabId);
        if (index < 0 || tabs.length <= 1) return;

        tabs.splice(index, 1);
        if (activeTabId === tabId) {
            activeTabId = tabs[Math.min(index, tabs.length - 1)].id;
            const tab = getActiveTab();
            navigateTo(tab.path);
        } else {
            TabBar.render(tabs, activeTabId, {
                onSelect: switchTab,
                onClose: closeTab,
                onNew: newTab,
            });
        }
    }

    // ── File Operations ─────────────────────────────────────────────

    async function openFile(path) {
        try {
            await invoke('open_file', { path });
        } catch (e) {
            console.error('Open error:', e);
        }
    }

    function onSelectionChange(entries) {
        const tab = getActiveTab();
        if (tab) tab.selection = entries;

        // Update status bar
        if (entries.length > 0) {
            const totalSize = entries.reduce((s, e) => s + e.size, 0);
            document.getElementById('status-selected').textContent =
                `${entries.length} selected`;
            document.getElementById('status-size').textContent = formatSize(totalSize);
        } else {
            document.getElementById('status-selected').textContent = '';
        }

        // Update preview
        if (entries.length === 1) {
            Preview.show(entries[0]);
        } else {
            Preview.show(null);
        }
    }

    function showContextMenu(e, entries) {
        const tab = getActiveTab();
        const currentPath = tab ? tab.path : '';
        const hasSelection = entries.length > 0;
        const singleEntry = entries.length === 1 ? entries[0] : null;

        const items = [];

        if (singleEntry) {
            items.push({
                label: singleEntry.is_dir ? 'Open' : 'Open File',
                shortcut: 'Enter',
                action: () => {
                    if (singleEntry.is_dir) navigateTo(singleEntry.path);
                    else openFile(singleEntry.path);
                }
            });

            if (singleEntry.is_dir) {
                items.push({
                    label: 'Open in New Tab',
                    action: () => {
                        const newT = createTab(singleEntry.path);
                        activeTabId = newT.id;
                        navigateTo(singleEntry.path);
                    }
                });
            }

            items.push({ separator: true });
        }

        if (hasSelection) {
            items.push({
                label: 'Copy',
                shortcut: 'Ctrl+C',
                action: () => invoke('clipboard_copy', { paths: entries.map(e => e.path) }),
            });
            items.push({
                label: 'Cut',
                shortcut: 'Ctrl+X',
                action: () => invoke('clipboard_cut', { paths: entries.map(e => e.path) }),
            });
        }

        items.push({
            label: 'Paste',
            shortcut: 'Ctrl+V',
            action: () => paste(),
        });

        items.push({ separator: true });

        if (singleEntry) {
            items.push({
                label: 'Rename',
                shortcut: 'F2',
                action: () => startRename(singleEntry),
            });
        }

        if (hasSelection) {
            items.push({
                label: 'Delete',
                shortcut: 'Del',
                danger: true,
                action: () => deleteSelected(false),
            });
            items.push({
                label: 'Delete Permanently',
                shortcut: 'Shift+Del',
                danger: true,
                action: () => deleteSelected(true),
            });
        }

        items.push({ separator: true });

        items.push({
            label: 'New Folder',
            shortcut: 'Ctrl+Shift+N',
            action: () => createNewFolder(),
        });
        items.push({
            label: 'New File',
            shortcut: 'Ctrl+N',
            action: () => createNewFile(),
        });

        if (singleEntry) {
            items.push({ separator: true });
            items.push({
                label: 'Copy Path',
                action: () => navigator.clipboard.writeText(singleEntry.path),
            });
            items.push({
                label: 'Open in Terminal',
                action: () => invoke('open_in_terminal', { path: singleEntry.path }),
            });
        }

        if (hasSelection) {
            const archivable = entries.some(e => {
                const ext = (e.extension || '').toLowerCase();
                return ['zip', 'tar', 'gz', '7z', 'rar'].includes(ext);
            });

            if (archivable && entries.length === 1) {
                items.push({ separator: true });
                items.push({
                    label: 'Extract Here',
                    action: () => extractArchive(entries[0].path, currentPath),
                });
            }

            items.push({ separator: true });
            items.push({
                label: 'Create Zip',
                action: () => createZipFromSelection(entries, currentPath),
            });
        }

        ContextMenu.show(e.clientX, e.clientY, items);
    }

    // ── File Actions ────────────────────────────────────────────────

    async function paste() {
        const tab = getActiveTab();
        if (!tab) return;
        try {
            await invoke('clipboard_paste', { destDir: tab.path });
            navigateTo(tab.path);
        } catch (e) {
            console.error('Paste error:', e);
        }
    }

    async function deleteSelected(permanent) {
        const paths = FileList.getSelectedPaths();
        if (paths.length === 0) return;

        try {
            if (permanent) {
                await invoke('delete_permanent', { paths });
            } else {
                await invoke('delete_to_trash', { paths });
            }
            const tab = getActiveTab();
            if (tab) navigateTo(tab.path);
        } catch (e) {
            console.error('Delete error:', e);
        }
    }

    async function startRename(entry) {
        const newName = prompt('Rename to:', entry.name);
        if (newName && newName !== entry.name) {
            try {
                await invoke('rename_item', { path: entry.path, newName });
                const tab = getActiveTab();
                if (tab) navigateTo(tab.path);
            } catch (e) {
                console.error('Rename error:', e);
            }
        }
    }

    async function createNewFolder() {
        const tab = getActiveTab();
        if (!tab) return;
        const name = prompt('New folder name:', 'New Folder');
        if (name) {
            try {
                await invoke('create_directory', { parent: tab.path, name });
                navigateTo(tab.path);
            } catch (e) {
                console.error('Create folder error:', e);
            }
        }
    }

    async function createNewFile() {
        const tab = getActiveTab();
        if (!tab) return;
        const name = prompt('New file name:', 'untitled.txt');
        if (name) {
            try {
                await invoke('create_file', { parent: tab.path, name });
                navigateTo(tab.path);
            } catch (e) {
                console.error('Create file error:', e);
            }
        }
    }

    async function extractArchive(archivePath, destDir) {
        try {
            const count = await invoke('extract_archive', { archivePath, destDir });
            console.log(`Extracted ${count} files`);
            navigateTo(destDir);
        } catch (e) {
            console.error('Extract error:', e);
        }
    }

    async function createZipFromSelection(entries, currentPath) {
        const sources = entries.map(e => e.path);
        const name = entries.length === 1
            ? entries[0].name.replace(/\.[^.]+$/, '') + '.zip'
            : 'archive.zip';
        const archivePath = currentPath.replace(/\\/g, '/') + '/' + name;
        try {
            await invoke('create_archive', { sources, archivePath });
            navigateTo(currentPath);
        } catch (e) {
            console.error('Create archive error:', e);
        }
    }

    // ── Keyboard Shortcuts ──────────────────────────────────────────

    function registerShortcuts() {
        // Navigation
        Keybindings.register('alt+arrowleft', goBack);
        Keybindings.register('alt+arrowright', goForward);
        Keybindings.register('backspace', goUp);

        // Tabs
        Keybindings.register('ctrl+t', newTab);
        Keybindings.register('ctrl+w', () => closeTab(activeTabId));
        Keybindings.register('ctrl+tab', () => {
            const idx = tabs.findIndex(t => t.id === activeTabId);
            const next = tabs[(idx + 1) % tabs.length];
            switchTab(next.id);
        });

        // File operations
        Keybindings.register('ctrl+c', () => {
            const paths = FileList.getSelectedPaths();
            if (paths.length) invoke('clipboard_copy', { paths });
        });
        Keybindings.register('ctrl+x', () => {
            const paths = FileList.getSelectedPaths();
            if (paths.length) invoke('clipboard_cut', { paths });
        });
        Keybindings.register('ctrl+v', paste);
        Keybindings.register('delete', () => deleteSelected(false));
        Keybindings.register('shift+delete', () => deleteSelected(true));
        Keybindings.register('f2', () => {
            const entry = FileList.getSelectedEntry();
            if (entry) startRename(entry);
        });

        // Selection
        Keybindings.register('ctrl+a', () => {
            const entries = FileList.selectAll();
            onSelectionChange(entries);
        });
        Keybindings.register('ctrl+shift+a', () => {
            const entries = FileList.invertSelection();
            onSelectionChange(entries);
        });

        // Arrow key navigation
        Keybindings.register('arrowup', () => {
            const entry = FileList.moveSelection(-1);
            if (entry) onSelectionChange([entry]);
        });
        Keybindings.register('arrowdown', () => {
            const entry = FileList.moveSelection(1);
            if (entry) onSelectionChange([entry]);
        });
        Keybindings.register('enter', () => {
            const entry = FileList.getSelectedEntry();
            if (entry) {
                if (entry.is_dir) navigateTo(entry.path);
                else openFile(entry.path);
            }
        });
        Keybindings.register(' ', () => {
            // Space toggles selection (like vim)
        });

        // New items
        Keybindings.register('ctrl+shift+n', createNewFolder);
        Keybindings.register('ctrl+n', createNewFile);

        // Views
        Keybindings.register('ctrl+1', () => { viewMode = 'list'; refresh(); });
        Keybindings.register('ctrl+2', () => { viewMode = 'grid'; refresh(); });

        // Panels
        Keybindings.register('ctrl+p', () => Preview.toggle());
        Keybindings.register('ctrl+`', () => Terminal.toggle());

        // Search
        Keybindings.register('ctrl+f', () => SearchBar.focus());
        Keybindings.register('/', () => SearchBar.focus());

        // Address bar
        Keybindings.register('ctrl+l', () => {
            const tab = getActiveTab();
            if (tab) Breadcrumb.focusEdit(tab.path, navigateTo);
        });

        // Toggle hidden files
        Keybindings.register('ctrl+h', async () => {
            await invoke('toggle_hidden');
            refresh();
        });
    }

    async function refresh() {
        const tab = getActiveTab();
        if (tab) navigateTo(tab.path);
    }

    // ── Helpers ─────────────────────────────────────────────────────

    function formatSize(bytes) {
        if (!bytes) return '';
        const units = ['B', 'KB', 'MB', 'GB', 'TB'];
        const i = Math.floor(Math.log(bytes) / Math.log(1024));
        return (bytes / Math.pow(1024, i)).toFixed(i > 0 ? 1 : 0) + ' ' + units[i];
    }

    // ── Initialization ─────────────────────────────────────────────

    async function init() {
        // Initialize sub-components
        DragDrop.init();
        Keybindings.init();
        ContextMenu.init();
        SearchBar.init((results) => {
            if (results === null) {
                // Cleared search — refresh current directory
                refresh();
            } else {
                // Show search results in file list
                const entries = results.map(r => ({
                    name: r.name,
                    path: r.path,
                    is_dir: false,
                    is_hidden: false,
                    is_symlink: false,
                    extension: r.name.split('.').pop(),
                    size: r.size,
                    modified: r.modified,
                    created: 0,
                    readonly: false,
                }));
                FileList.render(entries, viewMode, {
                    onNavigate: navigateTo,
                    onOpen: openFile,
                    onSelectionChange: onSelectionChange,
                    onContextMenu: showContextMenu,
                });
            }
        });

        registerShortcuts();

        // Window controls
        const win = getCurrentWindow();
        document.getElementById('btn-minimize').addEventListener('click', () => win.minimize());
        document.getElementById('btn-maximize').addEventListener('click', async () => {
            if (await win.isMaximized()) win.unmaximize();
            else win.maximize();
        });
        document.getElementById('btn-close').addEventListener('click', () => win.close());

        // Navigation buttons
        document.getElementById('btn-back').addEventListener('click', goBack);
        document.getElementById('btn-forward').addEventListener('click', goForward);
        document.getElementById('btn-up').addEventListener('click', goUp);

        // Create initial tab
        const home = await invoke('get_home_dir');
        const tab = createTab(home);
        activeTabId = tab.id;
        await navigateTo(home);
    }

    // Boot
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }

    return { navigateTo, refresh };
})();
