// History Search component — search past notifications.
const HistorySearch = (function () {
    let container = null;
    let entries = [];
    let query = '';
    let searchTimer = null;

    function init(el) {
        container = el;
        render();
        loadRecent();
    }

    async function loadRecent() {
        try {
            entries = await window.__TAURI__.core.invoke('get_recent_history', { limit: 100 });
        } catch (e) {
            entries = [];
        }
        render();
    }

    async function search(q) {
        if (!q.trim()) {
            return loadRecent();
        }
        try {
            entries = await window.__TAURI__.core.invoke('search_history', { query: q, limit: 50 });
        } catch (e) {
            entries = [];
        }
        render();
    }

    function priorityBadge(p) {
        return `<span class="badge badge-${esc(p)}">${esc(p)}</span>`;
    }

    function formatDate(ts) {
        try {
            const d = new Date(ts);
            return d.toLocaleDateString(undefined, {
                month: 'short',
                day: 'numeric',
                hour: '2-digit',
                minute: '2-digit',
            });
        } catch {
            return ts;
        }
    }

    function render() {
        if (!container) return;

        const toolbar = `
            <div class="toolbar">
                <input type="text" id="history-search" placeholder="Search notifications..." value="${esc(query)}" style="flex:1">
                <button class="btn" id="history-refresh">Refresh</button>
                <span style="color:var(--text-muted);font-size:11px">${entries.length} results</span>
            </div>`;

        if (entries.length === 0) {
            container.innerHTML = toolbar + `
                <div class="empty-state">
                    <div class="empty-state-icon">📋</div>
                    ${query ? 'No results found' : 'No notification history'}
                </div>`;
            bindToolbar();
            return;
        }

        const items = entries.map(e => `
            <div class="history-entry">
                <div style="display:flex;align-items:center;gap:6px">
                    <span class="history-title">${esc(e.title)}</span>
                    ${priorityBadge(e.priority)}
                </div>
                ${e.body ? `<div class="history-body">${esc(e.body)}</div>` : ''}
                <div class="history-meta">
                    ${esc(e.source)} &middot; ${formatDate(e.timestamp)}
                </div>
            </div>
        `).join('');

        container.innerHTML = toolbar + `<div style="overflow:auto;flex:1">${items}</div>`;
        bindToolbar();
    }

    function bindToolbar() {
        const input = container.querySelector('#history-search');
        if (input) {
            input.addEventListener('input', (e) => {
                query = e.target.value;
                clearTimeout(searchTimer);
                searchTimer = setTimeout(() => search(query), 300);
            });
            // Focus the input if it was already focused
            if (document.activeElement === input) {
                input.setSelectionRange(input.value.length, input.value.length);
            }
        }
        container.querySelector('#history-refresh')?.addEventListener('click', () => {
            query ? search(query) : loadRecent();
        });
    }

    function refresh() {
        query ? search(query) : loadRecent();
    }

    function esc(s) {
        if (!s) return '';
        const d = document.createElement('div');
        d.textContent = s;
        return d.innerHTML;
    }

    return { init, refresh };
})();
