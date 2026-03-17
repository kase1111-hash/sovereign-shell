// Rules Editor component — manage per-app notification filtering rules.
const RulesEditor = (function () {
    let container = null;
    let rules = [];

    function init(el) {
        container = el;
        render();
        refresh();
    }

    async function refresh() {
        try {
            rules = await window.__TAURI__.core.invoke('get_rules');
        } catch (e) {
            rules = [];
        }
        render();
    }

    function actionBadge(action) {
        return `<span class="badge badge-${action}">${action}</span>`;
    }

    function render() {
        if (!container) return;

        const toolbar = `
            <div class="toolbar">
                <span style="font-weight:500">Notification Rules</span>
                <span style="margin-left:auto"></span>
                <button class="btn btn-accent" id="rules-add">+ Add Rule</button>
                <button class="btn" id="rules-refresh">Refresh</button>
            </div>
            <div style="font-size:11px;color:var(--text-muted);margin-bottom:10px">
                Rules control how notifications from each source are handled.
                Default: show all notifications for 5 seconds.
            </div>`;

        if (rules.length === 0) {
            container.innerHTML = toolbar + `
                <div class="empty-state">
                    <div class="empty-state-icon">📝</div>
                    No custom rules — all notifications shown by default
                </div>`;
            bindToolbar();
            return;
        }

        const rows = rules.map(r => `
            <tr>
                <td style="font-weight:500">${esc(r.source)}</td>
                <td>${actionBadge(r.action)}</td>
                <td>${r.duration_seconds}s</td>
                <td>${r.priority ? `<span class="badge badge-${r.priority}">${r.priority}</span>` : 'default'}</td>
                <td>
                    <div style="display:flex;gap:4px">
                        <button class="btn btn-sm" data-edit="${esc(r.source)}">Edit</button>
                        <button class="btn btn-sm btn-danger" data-remove="${esc(r.source)}">Remove</button>
                    </div>
                </td>
            </tr>
        `).join('');

        container.innerHTML = toolbar + `
            <div style="overflow:auto;flex:1">
                <table class="rules-table">
                    <thead><tr>
                        <th>Source</th>
                        <th>Action</th>
                        <th>Duration</th>
                        <th>Priority</th>
                        <th>Actions</th>
                    </tr></thead>
                    <tbody>${rows}</tbody>
                </table>
            </div>`;

        bindToolbar();
        bindActions();
    }

    function bindToolbar() {
        container.querySelector('#rules-add')?.addEventListener('click', addRule);
        container.querySelector('#rules-refresh')?.addEventListener('click', refresh);
    }

    function bindActions() {
        container.querySelectorAll('[data-edit]').forEach(btn => {
            btn.addEventListener('click', () => editRule(btn.dataset.edit));
        });
        container.querySelectorAll('[data-remove]').forEach(btn => {
            btn.addEventListener('click', async () => {
                const source = btn.dataset.remove;
                try {
                    await window.__TAURI__.core.invoke('remove_rule', { source });
                } catch (e) {
                    console.error('Remove rule failed:', e);
                }
                refresh();
            });
        });
    }

    async function addRule() {
        const source = prompt('Application/source name:');
        if (!source) return;

        const action = prompt('Action (show / silent / block):', 'show');
        if (!action) return;

        const duration = parseInt(prompt('Toast duration (seconds):', '5')) || 5;

        const priorityInput = prompt('Priority override (low/normal/high/critical, or leave empty):', '');
        const priority = priorityInput && ['low', 'normal', 'high', 'critical'].includes(priorityInput.toLowerCase())
            ? priorityInput.toLowerCase()
            : null;

        try {
            await window.__TAURI__.core.invoke('set_rule', {
                rule: { source, action, duration_seconds: duration, priority },
            });
        } catch (e) {
            console.error('Add rule failed:', e);
        }
        refresh();
    }

    async function editRule(source) {
        const existing = rules.find(r => r.source === source);
        if (!existing) return;

        const action = prompt(`Action for ${source} (show / silent / block):`, existing.action);
        if (!action) return;

        const duration = parseInt(prompt('Toast duration (seconds):', String(existing.duration_seconds))) || 5;

        const priorityInput = prompt('Priority override (low/normal/high/critical, or empty):', existing.priority || '');
        const priority = priorityInput && ['low', 'normal', 'high', 'critical'].includes(priorityInput.toLowerCase())
            ? priorityInput.toLowerCase()
            : null;

        try {
            await window.__TAURI__.core.invoke('set_rule', {
                rule: { source, action, duration_seconds: duration, priority },
            });
        } catch (e) {
            console.error('Edit rule failed:', e);
        }
        refresh();
    }

    function esc(s) {
        if (!s) return '';
        const d = document.createElement('div');
        d.textContent = s;
        return d.innerHTML;
    }

    return { init, refresh };
})();
