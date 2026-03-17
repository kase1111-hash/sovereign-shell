// Firewall Rules component — lists and manages firewall rules.
const FirewallRules = (function () {
    let container = null;
    let rules = [];
    let filter = '';
    let dirFilter = 'all';

    function init(el) {
        container = el;
        render();
        refresh();
    }

    async function refresh() {
        try {
            rules = await window.__TAURI__.core.invoke('get_firewall_rules');
        } catch (e) {
            rules = [];
            console.error('Failed to load firewall rules:', e);
        }
        render();
    }

    function filtered() {
        let list = rules;
        if (dirFilter !== 'all') {
            list = list.filter(r => r.direction.toLowerCase() === dirFilter);
        }
        if (filter) {
            const q = filter.toLowerCase();
            list = list.filter(r =>
                r.name.toLowerCase().includes(q) ||
                r.protocol.toLowerCase().includes(q) ||
                r.program.toLowerCase().includes(q)
            );
        }
        return list;
    }

    function render() {
        if (!container) return;

        const toolbar = `
            <div class="toolbar">
                <input type="text" placeholder="Filter rules..." id="fw-filter" value="${esc(filter)}" style="width:220px">
                <select id="fw-dir">
                    <option value="all" ${dirFilter === 'all' ? 'selected' : ''}>All Directions</option>
                    <option value="in" ${dirFilter === 'in' ? 'selected' : ''}>Inbound</option>
                    <option value="out" ${dirFilter === 'out' ? 'selected' : ''}>Outbound</option>
                </select>
                <button class="btn btn-accent" id="fw-refresh">Refresh</button>
                <button class="btn" id="fw-add">+ New Rule</button>
                <span style="margin-left:auto;color:var(--text-muted);font-size:11px">${filtered().length} rules</span>
            </div>`;

        const rows = filtered().map(r => `
            <tr>
                <td>${esc(r.name)}</td>
                <td>${esc(r.direction)}</td>
                <td><span class="badge ${r.action === 'Allow' ? 'badge-allow' : 'badge-block'}">${esc(r.action)}</span></td>
                <td>${esc(r.protocol)}</td>
                <td style="font-family:var(--mono)">${esc(r.local_port)}</td>
                <td>${esc(r.profile)}</td>
                <td>
                    <label class="toggle">
                        <input type="checkbox" ${r.enabled ? 'checked' : ''} data-name="${esc(r.name)}">
                        <span class="toggle-slider"></span>
                    </label>
                </td>
            </tr>`).join('');

        container.innerHTML = toolbar + `
            <div class="table-wrap">
                <table>
                    <thead><tr>
                        <th>Name</th>
                        <th>Direction</th>
                        <th>Action</th>
                        <th>Protocol</th>
                        <th>Port</th>
                        <th>Profile</th>
                        <th>Enabled</th>
                    </tr></thead>
                    <tbody>${rows || '<tr><td colspan="7" style="text-align:center;color:var(--text-muted)">No rules found</td></tr>'}</tbody>
                </table>
            </div>`;

        bind();
    }

    function bind() {
        const fInput = container.querySelector('#fw-filter');
        if (fInput) fInput.addEventListener('input', e => { filter = e.target.value; render(); });
        const dSelect = container.querySelector('#fw-dir');
        if (dSelect) dSelect.addEventListener('change', e => { dirFilter = e.target.value; render(); });
        container.querySelector('#fw-refresh')?.addEventListener('click', refresh);
        container.querySelector('#fw-add')?.addEventListener('click', showAddDialog);

        container.querySelectorAll('.toggle input').forEach(el => {
            el.addEventListener('change', async (e) => {
                const name = e.target.dataset.name;
                try {
                    await window.__TAURI__.core.invoke('toggle_firewall_rule', {
                        name, enabled: e.target.checked,
                    });
                } catch (err) {
                    console.error('Toggle rule failed:', err);
                }
                setTimeout(refresh, 500);
            });
        });
    }

    function showAddDialog() {
        const name = prompt('Rule name:');
        if (!name) return;
        const direction = prompt('Direction (In/Out):', 'In');
        if (!direction) return;
        const action = prompt('Action (Allow/Block):', 'Block');
        if (!action) return;
        const protocol = prompt('Protocol (TCP/UDP/Any):', 'TCP');
        const port = prompt('Local port (or Any):', 'Any');

        createRule({
            name,
            direction,
            action,
            protocol: protocol || 'Any',
            local_port: port || 'Any',
            remote_port: 'Any',
            program: '',
            enabled: true,
            profile: 'Any',
        });
    }

    async function createRule(rule) {
        try {
            await window.__TAURI__.core.invoke('create_firewall_rule', { rule });
        } catch (err) {
            console.error('Create rule failed:', err);
        }
        setTimeout(refresh, 500);
    }

    function esc(s) {
        if (!s) return '';
        const d = document.createElement('div');
        d.textContent = s;
        return d.innerHTML;
    }

    return { init, refresh };
})();
