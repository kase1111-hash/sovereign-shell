// Preview pane component.

const Preview = (() => {
    const { invoke } = window.__TAURI__.core;
    let visible = false;

    function toggle() {
        visible = !visible;
        const pane = document.getElementById('preview-pane');
        pane.classList.toggle('visible', visible);
        return visible;
    }

    function isVisible() { return visible; }

    async function show(entry) {
        if (!visible) return;
        const pane = document.getElementById('preview-pane');

        if (!entry) {
            pane.innerHTML = '<div class="empty-state">Select a file to preview</div>';
            return;
        }

        let html = `<div class="preview-title">${escapeHtml(entry.name)}</div>`;

        // Metadata
        html += `<div class="preview-meta"><span class="preview-meta-label">Size:</span> ${formatSize(entry.size)}</div>`;
        html += `<div class="preview-meta"><span class="preview-meta-label">Modified:</span> ${formatDate(entry.modified)}</div>`;
        html += `<div class="preview-meta"><span class="preview-meta-label">Type:</span> ${entry.is_dir ? 'Folder' : (entry.extension || 'File').toUpperCase()}</div>`;

        if (entry.is_dir) {
            pane.innerHTML = html;
            return;
        }

        const ext = (entry.extension || '').toLowerCase();
        const imageExts = ['png', 'jpg', 'jpeg', 'gif', 'svg', 'webp', 'bmp', 'ico'];
        const textExts = [
            'txt', 'md', 'log', 'csv', 'json', 'toml', 'yaml', 'yml', 'xml',
            'rs', 'py', 'js', 'ts', 'html', 'css', 'java', 'c', 'cpp', 'h',
            'go', 'rb', 'sh', 'ps1', 'bat', 'cfg', 'ini', 'conf', 'env',
        ];

        if (imageExts.includes(ext)) {
            html += `<img class="preview-image" src="https://asset.localhost/${entry.path}" alt="${escapeHtml(entry.name)}" />`;
        } else if (textExts.includes(ext) && entry.size < 1024 * 1024) {
            try {
                const content = await invoke('read_text_preview', { path: entry.path, maxLines: 200 });
                html += `<div class="preview-content">${escapeHtml(content)}</div>`;
            } catch (e) {
                html += `<div class="preview-meta">Cannot preview: ${escapeHtml(String(e))}</div>`;
            }
        }

        pane.innerHTML = html;
    }

    function formatSize(bytes) {
        if (!bytes) return '0 B';
        const units = ['B', 'KB', 'MB', 'GB', 'TB'];
        const i = Math.floor(Math.log(bytes) / Math.log(1024));
        return (bytes / Math.pow(1024, i)).toFixed(i > 0 ? 1 : 0) + ' ' + units[i];
    }

    function formatDate(ts) {
        if (!ts) return '';
        return new Date(ts * 1000).toLocaleString();
    }

    function escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }

    return { toggle, isVisible, show };
})();
