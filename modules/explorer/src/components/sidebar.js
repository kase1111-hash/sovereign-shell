// Sidebar component: drives, bookmarks, directory tree.

const Sidebar = (() => {
    const { invoke } = window.__TAURI__.core;

    async function render(currentPath, onNavigate) {
        const container = document.getElementById('sidebar');
        container.innerHTML = '';

        // Bookmarks section
        try {
            const bookmarks = await invoke('get_bookmarks');
            if (bookmarks.length > 0) {
                const section = createSection('Quick Access');
                bookmarks.forEach(bm => {
                    const item = createItem(bm.name, '\u{1F4C1}', bm.path, currentPath, onNavigate);
                    section.appendChild(item);
                });
                container.appendChild(section);
            }
        } catch (e) {
            console.error('Bookmarks error:', e);
        }

        // Drives section
        try {
            const drives = await invoke('list_drives');
            if (drives.length > 0) {
                const section = createSection('Drives');
                drives.forEach(drive => {
                    const label = drive.label || drive.path;
                    const item = createItem(label, '\u{1F4BF}', drive.path, currentPath, onNavigate);
                    section.appendChild(item);
                });
                container.appendChild(section);
            }
        } catch (e) {
            console.error('Drives error:', e);
        }
    }

    function createSection(title) {
        const section = document.createElement('div');
        section.className = 'sidebar-section';

        const header = document.createElement('div');
        header.className = 'sidebar-header';
        header.textContent = title;
        section.appendChild(header);

        return section;
    }

    function createItem(name, icon, path, currentPath, onNavigate) {
        const item = document.createElement('div');
        item.className = 'sidebar-item' + (normalizePath(path) === normalizePath(currentPath) ? ' active' : '');

        item.innerHTML = `
            <span class="sidebar-icon">${icon}</span>
            <span>${escapeHtml(name)}</span>
        `;

        item.addEventListener('click', () => onNavigate(path));

        // Drop target for moving files
        DragDrop.makeDropTarget(item, (paths) => {
            if (paths.length > 0) {
                invoke('move_items', { sources: paths, destDir: path })
                    .then(() => onNavigate(currentPath))
                    .catch(e => console.error('Move error:', e));
            }
        });

        return item;
    }

    function normalizePath(p) {
        return p.replace(/\\/g, '/').replace(/\/+$/, '').toLowerCase();
    }

    function escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }

    return { render };
})();
