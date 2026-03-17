// Connection Table component — shows active network connections.
const ConnectionTable = (function () {
    let container = null;
    let connections = [];
    let filter = '';
    let protoFilter = 'all';

    function init(el) {
        container = el;
        render();
        refresh();
    }

    async function refresh() {
        try {
            connections = await window.__TAURI__.core.invoke('get_connections');
        } catch (e) {
            connections = [];
            console.error('Failed to load connections:', e);
        }
        render();
    }

    function filtered() {
        let list = connections;
        if (protoFilter !== 'all') {
            list = list.filter(c => c.protocol.toLowerCase() === protoFilter);
        }
        if (filter) {
            const q = filter.toLowerCase();
            list = list.filter(c =>
                c.local_address.toLowerCase().includes(q) ||
                c.remote_address.toLowerCase().includes(q) ||
                c.process_name.toLowerCase().includes(q) ||
                String(c.pid).includes(q)
            );
        }
        return list;
    }

    function stateBadge(state) {
        const s = state.toUpperCase();
        if (s === 'LISTEN' || s === 'LISTENING') return 'badge-listen';
        if (s === 'ESTABLISHED') return 'badge-established';
        if (s.includes('WAIT')) return 'badge-time-wait';
        return '';
    }

    function render() {
        if (!container) return;

        const toolbar = `
            <div class="toolbar">
                <input type="text" placeholder="Filter connections..." id="conn-filter" value="${esc(filter)}" style="width:220px">
                <select id="conn-proto">
                    <option value="all" ${protoFilter === 'all' ? 'selected' : ''}>All Protocols</option>
                    <option value="tcp" ${protoFilter === 'tcp' ? 'selected' : ''}>TCP</option>
                    <option value="udp" ${protoFilter === 'udp' ? 'selected' : ''}>UDP</option>
                </select>
                <button class="btn btn-accent" id="conn-refresh">Refresh</button>
                <span style="margin-left:auto;color:var(--text-muted);font-size:11px">${filtered().length} connections</span>
            </div>`;

        const rows = filtered().map(c => `
            <tr>
                <td>${esc(c.protocol)}</td>
                <td style="font-family:var(--mono)">${esc(c.local_address)}</td>
                <td style="font-family:var(--mono)">${esc(c.remote_address)}</td>
                <td><span class="badge ${stateBadge(c.state)}">${esc(c.state)}</span></td>
                <td>${c.pid || ''}</td>
                <td>${esc(c.process_name)}</td>
            </tr>`).join('');

        container.innerHTML = toolbar + `
            <div class="table-wrap">
                <table>
                    <thead><tr>
                        <th>Proto</th>
                        <th>Local Address</th>
                        <th>Remote Address</th>
                        <th>State</th>
                        <th>PID</th>
                        <th>Process</th>
                    </tr></thead>
                    <tbody>${rows || '<tr><td colspan="6" style="text-align:center;color:var(--text-muted)">No connections</td></tr>'}</tbody>
                </table>
            </div>`;

        bind();
    }

    function bind() {
        const fInput = container.querySelector('#conn-filter');
        if (fInput) fInput.addEventListener('input', e => { filter = e.target.value; render(); });
        const pSelect = container.querySelector('#conn-proto');
        if (pSelect) pSelect.addEventListener('change', e => { protoFilter = e.target.value; render(); });
        const btn = container.querySelector('#conn-refresh');
        if (btn) btn.addEventListener('click', refresh);
    }

    function esc(s) {
        if (!s) return '';
        const d = document.createElement('div');
        d.textContent = s;
        return d.innerHTML;
    }

    return { init, refresh };
})();
