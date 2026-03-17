// Main file listing component (list and grid views).

const FileList = (() => {
    const { invoke } = window.__TAURI__.core;

    let currentEntries = [];
    let selectedPaths = new Set();
    let lastClickedIndex = -1;
    let sortColumn = 'name';
    let sortAscending = true;
    let currentView = 'list'; // 'list' | 'grid'
    let renameTarget = null;

    function render(entries, viewMode, callbacks) {
        currentEntries = sortEntries(entries);
        currentView = viewMode || 'list';
        const container = document.getElementById('file-list');
        container.innerHTML = '';

        if (currentEntries.length === 0) {
            container.innerHTML = '<div class="empty-state">This folder is empty</div>';
            return;
        }

        if (currentView === 'grid') {
            renderGrid(container, callbacks);
        } else {
            renderList(container, callbacks);
        }
    }

    function renderList(container, callbacks) {
        // Header row
        const header = document.createElement('div');
        header.className = 'file-list-header';
        header.innerHTML = `
            <div class="file-header-col col-name" data-col="name">Name ${sortIndicator('name')}</div>
            <div class="file-header-col col-size" data-col="size">Size ${sortIndicator('size')}</div>
            <div class="file-header-col col-modified" data-col="modified">Modified ${sortIndicator('modified')}</div>
            <div class="file-header-col col-type" data-col="type">Type ${sortIndicator('type')}</div>
        `;
        header.querySelectorAll('.file-header-col').forEach(col => {
            col.addEventListener('click', () => {
                const newCol = col.dataset.col;
                if (sortColumn === newCol) {
                    sortAscending = !sortAscending;
                } else {
                    sortColumn = newCol;
                    sortAscending = true;
                }
                render(currentEntries, currentView, callbacks);
            });
        });
        container.appendChild(header);

        // File rows
        currentEntries.forEach((entry, index) => {
            const row = document.createElement('div');
            row.className = 'file-row' + (selectedPaths.has(entry.path) ? ' selected' : '');
            row.draggable = true;
            row.dataset.path = entry.path;
            row.dataset.index = index;

            const icon = getFileIcon(entry);
            row.innerHTML = `
                <span class="file-icon">${icon}</span>
                <span class="file-name">${escapeHtml(entry.name)}</span>
                <span class="file-size">${entry.is_dir ? '' : formatSize(entry.size)}</span>
                <span class="file-modified">${formatDate(entry.modified)}</span>
                <span class="file-type">${entry.is_dir ? 'Folder' : (entry.extension || '').toUpperCase()}</span>
            `;

            // Click to select
            row.addEventListener('click', (e) => handleRowClick(e, entry, index, callbacks));

            // Double-click to open
            row.addEventListener('dblclick', () => {
                if (entry.is_dir) {
                    callbacks.onNavigate(entry.path);
                } else {
                    callbacks.onOpen(entry.path);
                }
            });

            // Context menu
            row.addEventListener('contextmenu', (e) => {
                e.preventDefault();
                if (!selectedPaths.has(entry.path)) {
                    selectedPaths.clear();
                    selectedPaths.add(entry.path);
                    highlightSelected();
                }
                callbacks.onContextMenu(e, getSelectedEntries());
            });

            // Drag
            row.addEventListener('dragstart', (e) => {
                if (!selectedPaths.has(entry.path)) {
                    selectedPaths.clear();
                    selectedPaths.add(entry.path);
                    highlightSelected();
                }
                DragDrop.setDragData([...selectedPaths]);
                e.dataTransfer.effectAllowed = 'move';
            });

            container.appendChild(row);
        });

        // Click empty area in file list
        container.addEventListener('click', (e) => {
            if (e.target === container || e.target.id === 'file-list') {
                selectedPaths.clear();
                highlightSelected();
                callbacks.onSelectionChange([]);
            }
        });

        // Context menu on empty area
        container.addEventListener('contextmenu', (e) => {
            if (e.target === container || e.target.className === 'empty-state') {
                e.preventDefault();
                selectedPaths.clear();
                highlightSelected();
                callbacks.onContextMenu(e, []);
            }
        });
    }

    function renderGrid(container, callbacks) {
        const grid = document.createElement('div');
        grid.className = 'file-grid';

        currentEntries.forEach((entry, index) => {
            const item = document.createElement('div');
            item.className = 'file-grid-item' + (selectedPaths.has(entry.path) ? ' selected' : '');
            item.dataset.path = entry.path;

            const icon = getFileIcon(entry);
            item.innerHTML = `
                <span class="file-grid-icon">${icon}</span>
                <span class="file-grid-name">${escapeHtml(entry.name)}</span>
            `;

            item.addEventListener('click', (e) => handleRowClick(e, entry, index, callbacks));
            item.addEventListener('dblclick', () => {
                if (entry.is_dir) callbacks.onNavigate(entry.path);
                else callbacks.onOpen(entry.path);
            });
            item.addEventListener('contextmenu', (e) => {
                e.preventDefault();
                if (!selectedPaths.has(entry.path)) {
                    selectedPaths.clear();
                    selectedPaths.add(entry.path);
                    highlightSelected();
                }
                callbacks.onContextMenu(e, getSelectedEntries());
            });

            grid.appendChild(item);
        });

        container.appendChild(grid);
    }

    function handleRowClick(e, entry, index, callbacks) {
        if (e.ctrlKey) {
            // Toggle selection
            if (selectedPaths.has(entry.path)) {
                selectedPaths.delete(entry.path);
            } else {
                selectedPaths.add(entry.path);
            }
        } else if (e.shiftKey && lastClickedIndex >= 0) {
            // Range select
            const start = Math.min(lastClickedIndex, index);
            const end = Math.max(lastClickedIndex, index);
            for (let i = start; i <= end; i++) {
                selectedPaths.add(currentEntries[i].path);
            }
        } else {
            selectedPaths.clear();
            selectedPaths.add(entry.path);
        }

        lastClickedIndex = index;
        highlightSelected();
        callbacks.onSelectionChange(getSelectedEntries());
    }

    function highlightSelected() {
        document.querySelectorAll('.file-row, .file-grid-item').forEach(el => {
            el.classList.toggle('selected', selectedPaths.has(el.dataset.path));
        });
    }

    function selectAll() {
        selectedPaths.clear();
        currentEntries.forEach(e => selectedPaths.add(e.path));
        highlightSelected();
        return getSelectedEntries();
    }

    function invertSelection() {
        currentEntries.forEach(e => {
            if (selectedPaths.has(e.path)) selectedPaths.delete(e.path);
            else selectedPaths.add(e.path);
        });
        highlightSelected();
        return getSelectedEntries();
    }

    function clearSelection() {
        selectedPaths.clear();
        highlightSelected();
    }

    function getSelectedPaths() {
        return [...selectedPaths];
    }

    function getSelectedEntries() {
        return currentEntries.filter(e => selectedPaths.has(e.path));
    }

    function moveSelection(direction) {
        if (currentEntries.length === 0) return null;

        let newIndex;
        if (lastClickedIndex < 0) {
            newIndex = 0;
        } else {
            newIndex = lastClickedIndex + direction;
            if (newIndex < 0) newIndex = 0;
            if (newIndex >= currentEntries.length) newIndex = currentEntries.length - 1;
        }

        lastClickedIndex = newIndex;
        selectedPaths.clear();
        selectedPaths.add(currentEntries[newIndex].path);
        highlightSelected();

        // Scroll into view
        const row = document.querySelector(`[data-index="${newIndex}"]`);
        if (row) row.scrollIntoView({ block: 'nearest' });

        return currentEntries[newIndex];
    }

    function getSelectedEntry() {
        if (lastClickedIndex >= 0 && lastClickedIndex < currentEntries.length) {
            return currentEntries[lastClickedIndex];
        }
        return null;
    }

    function setView(view) {
        currentView = view;
    }

    // ── Helpers ──

    function sortEntries(entries) {
        const sorted = [...entries];
        // Directories always first
        sorted.sort((a, b) => {
            if (a.is_dir !== b.is_dir) return a.is_dir ? -1 : 1;

            let cmp = 0;
            switch (sortColumn) {
                case 'name':
                    cmp = a.name.localeCompare(b.name, undefined, { sensitivity: 'base' });
                    break;
                case 'size':
                    cmp = a.size - b.size;
                    break;
                case 'modified':
                    cmp = a.modified - b.modified;
                    break;
                case 'type':
                    cmp = (a.extension || '').localeCompare(b.extension || '');
                    break;
            }
            return sortAscending ? cmp : -cmp;
        });
        return sorted;
    }

    function sortIndicator(col) {
        if (sortColumn !== col) return '';
        return sortAscending ? '\u25B2' : '\u25BC';
    }

    function getFileIcon(entry) {
        if (entry.is_dir) return '\u{1F4C1}';
        const ext = (entry.extension || '').toLowerCase();
        const icons = {
            'png': '\u{1F5BC}', 'jpg': '\u{1F5BC}', 'jpeg': '\u{1F5BC}', 'gif': '\u{1F5BC}',
            'svg': '\u{1F5BC}', 'webp': '\u{1F5BC}', 'bmp': '\u{1F5BC}', 'ico': '\u{1F5BC}',
            'mp3': '\u{1F3B5}', 'wav': '\u{1F3B5}', 'flac': '\u{1F3B5}', 'ogg': '\u{1F3B5}',
            'mp4': '\u{1F3AC}', 'mkv': '\u{1F3AC}', 'avi': '\u{1F3AC}', 'mov': '\u{1F3AC}',
            'pdf': '\u{1F4C4}', 'doc': '\u{1F4C4}', 'docx': '\u{1F4C4}',
            'xls': '\u{1F4CA}', 'xlsx': '\u{1F4CA}', 'csv': '\u{1F4CA}',
            'zip': '\u{1F4E6}', 'tar': '\u{1F4E6}', 'gz': '\u{1F4E6}', '7z': '\u{1F4E6}', 'rar': '\u{1F4E6}',
            'exe': '\u{2699}', 'msi': '\u{2699}',
            'rs': '\u{1F4DD}', 'js': '\u{1F4DD}', 'ts': '\u{1F4DD}', 'py': '\u{1F4DD}',
            'html': '\u{1F310}', 'css': '\u{1F310}',
            'json': '\u{1F4CB}', 'toml': '\u{1F4CB}', 'yaml': '\u{1F4CB}', 'yml': '\u{1F4CB}',
            'md': '\u{1F4D6}', 'txt': '\u{1F4DD}', 'log': '\u{1F4DD}',
        };
        return icons[ext] || '\u{1F4C4}';
    }

    function formatSize(bytes) {
        if (bytes === 0) return '';
        const units = ['B', 'KB', 'MB', 'GB', 'TB'];
        const i = Math.floor(Math.log(bytes) / Math.log(1024));
        const val = bytes / Math.pow(1024, i);
        return val.toFixed(i > 0 ? 1 : 0) + ' ' + units[i];
    }

    function formatDate(timestamp) {
        if (!timestamp) return '';
        const d = new Date(timestamp * 1000);
        return d.toLocaleDateString(undefined, {
            year: 'numeric', month: 'short', day: 'numeric',
            hour: '2-digit', minute: '2-digit',
        });
    }

    function escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }

    return {
        render, selectAll, invertSelection, clearSelection,
        getSelectedPaths, getSelectedEntries, getSelectedEntry,
        moveSelection, setView,
    };
})();
