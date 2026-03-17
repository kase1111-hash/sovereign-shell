// Adapter List component — shows network adapters with status and controls.
const AdapterList = (function () {
    let container = null;
    let adapters = [];

    function init(el) {
        container = el;
        render();
        refresh();
    }

    async function refresh() {
        try {
            adapters = await window.__TAURI__.core.invoke('get_adapters');
        } catch (e) {
            adapters = [];
            console.error('Failed to load adapters:', e);
        }
        render();
    }

    function adapterIcon(type) {
        if (type === 'WiFi') return '📶';
        if (type === 'Ethernet') return '🔌';
        if (type === 'Virtual') return '🖧';
        return '🌐';
    }

    function render() {
        if (!container) return;

        const toolbar = `
            <div class="toolbar">
                <button class="btn btn-accent" id="adapter-refresh">Refresh</button>
            </div>`;

        if (adapters.length === 0) {
            container.innerHTML = toolbar + '<div class="empty-state">No adapters found</div>';
            bindToolbar();
            return;
        }

        const cards = adapters.map((a, i) => `
            <div class="card adapter-card">
                <div class="adapter-icon">${adapterIcon(a.adapter_type)}</div>
                <div class="adapter-info">
                    <div class="adapter-name">${esc(a.name)}</div>
                    <div class="adapter-detail">
                        ${esc(a.adapter_type)} &middot;
                        ${a.ip_address ? esc(a.ip_address) : 'No IP'} &middot;
                        MAC: ${esc(a.mac_address || 'N/A')}
                        ${a.gateway ? ' &middot; GW: ' + esc(a.gateway) : ''}
                        ${a.dns_servers && a.dns_servers.length ? ' &middot; DNS: ' + a.dns_servers.map(esc).join(', ') : ''}
                    </div>
                </div>
                <span class="badge ${a.status === 'Up' ? 'badge-up' : 'badge-down'}">${esc(a.status)}</span>
                <div class="adapter-actions">
                    <label class="toggle">
                        <input type="checkbox" ${a.status === 'Up' ? 'checked' : ''} data-idx="${i}">
                        <span class="toggle-slider"></span>
                    </label>
                    <button class="btn" data-dns-idx="${i}">DNS</button>
                </div>
            </div>
        `).join('');

        container.innerHTML = toolbar + cards;
        bindToolbar();
        bindActions();
    }

    function bindToolbar() {
        const btn = container.querySelector('#adapter-refresh');
        if (btn) btn.addEventListener('click', refresh);
    }

    function bindActions() {
        container.querySelectorAll('.toggle input').forEach(el => {
            el.addEventListener('change', async (e) => {
                const idx = parseInt(e.target.dataset.idx);
                const adapter = adapters[idx];
                if (!adapter) return;
                try {
                    await window.__TAURI__.core.invoke('set_adapter_state', {
                        name: adapter.name,
                        enabled: e.target.checked,
                    });
                } catch (err) {
                    console.error('Toggle adapter failed:', err);
                }
                setTimeout(refresh, 1000);
            });
        });

        container.querySelectorAll('[data-dns-idx]').forEach(el => {
            el.addEventListener('click', (e) => {
                const idx = parseInt(e.target.dataset.dnsIdx);
                const adapter = adapters[idx];
                if (!adapter) return;
                promptDns(adapter);
            });
        });
    }

    async function promptDns(adapter) {
        const current = (adapter.dns_servers || []).join(', ');
        const input = prompt(`DNS servers for ${adapter.name} (comma-separated):`, current);
        if (input === null) return;
        const servers = input.split(',').map(s => s.trim()).filter(Boolean);
        try {
            await window.__TAURI__.core.invoke('set_dns_servers', {
                adapterName: adapter.name,
                servers,
            });
        } catch (err) {
            console.error('Set DNS failed:', err);
        }
        setTimeout(refresh, 1000);
    }

    function esc(s) {
        if (!s) return '';
        const d = document.createElement('div');
        d.textContent = s;
        return d.innerHTML;
    }

    return { init, refresh };
})();
