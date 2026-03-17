// File lock finder — "Who is locking this file?"

const FileLockFinder = (() => {
    const { invoke } = window.__TAURI__.core;

    function render(container) {
        container.innerHTML = '';

        const finder = document.createElement('div');
        finder.className = 'lock-finder';

        finder.innerHTML = `
            <h3 style="margin-bottom:8px;font-size:15px;">Find File Locks</h3>
            <p style="margin-bottom:12px;color:var(--text-secondary);font-size:12px;">
                Enter a file path to discover which processes have it locked.
            </p>
            <div class="lock-finder-input">
                <input type="text" id="lock-path-input" placeholder="C:\\path\\to\\locked\\file.txt" />
                <button class="lock-finder-btn" id="lock-find-btn">Find Locks</button>
            </div>
            <div id="lock-results"></div>
        `;

        container.appendChild(finder);

        document.getElementById('lock-find-btn').addEventListener('click', doSearch);
        document.getElementById('lock-path-input').addEventListener('keydown', (e) => {
            if (e.key === 'Enter') doSearch();
        });
    }

    async function doSearch() {
        const input = document.getElementById('lock-path-input');
        const results = document.getElementById('lock-results');
        const path = input.value.trim();

        if (!path) return;

        results.innerHTML = '<div style="padding:8px;color:var(--text-muted)">Searching...</div>';

        try {
            const locks = await invoke('find_file_locks', { filePath: path });

            if (locks.length === 0) {
                results.innerHTML = `
                    <div style="padding:12px;color:var(--success);font-size:13px;">
                        No processes are locking this file.
                    </div>
                `;
                return;
            }

            results.innerHTML = `
                <div style="margin-bottom:8px;color:var(--text-secondary);font-size:12px;">
                    ${locks.length} process${locks.length > 1 ? 'es' : ''} locking this file:
                </div>
            `;

            locks.forEach(lock => {
                const el = document.createElement('div');
                el.className = 'lock-result';
                el.innerHTML = `
                    <div class="lock-result-info">
                        <span class="lock-result-name">${escapeHtml(lock.name)}</span>
                        <span class="lock-result-pid">PID: ${lock.pid}</span>
                        <span class="lock-result-desc">${escapeHtml(lock.description)}</span>
                    </div>
                    <button class="lock-kill-btn" data-pid="${lock.pid}" data-name="${escapeHtml(lock.name)}">
                        Kill Process
                    </button>
                `;

                el.querySelector('.lock-kill-btn').addEventListener('click', async (e) => {
                    const pid = parseInt(e.target.dataset.pid);
                    const name = e.target.dataset.name;
                    if (confirm(`Kill process "${name}" (PID ${pid})?`)) {
                        try {
                            await invoke('kill_process', { pid });
                            // Re-search after kill
                            setTimeout(doSearch, 500);
                        } catch (err) {
                            alert(`Kill failed: ${err}`);
                        }
                    }
                });

                results.appendChild(el);
            });
        } catch (e) {
            results.innerHTML = `
                <div style="padding:12px;color:var(--danger);font-size:13px;">
                    Error: ${e}
                </div>
            `;
        }
    }

    function escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }

    return { render };
})();
