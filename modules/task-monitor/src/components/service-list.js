// Windows services list with start/stop/restart actions.

const ServiceList = (() => {
    const { invoke } = window.__TAURI__.core;
    let sortCol = 'display_name';
    let sortAsc = true;
    let filter = '';

    async function render(container) {
        container.innerHTML = '<div style="padding:12px;color:var(--text-muted)">Loading services...</div>';

        try {
            const services = await invoke('get_services');
            renderTable(services, container);
        } catch (e) {
            container.innerHTML = `<div style="padding:12px;color:var(--danger)">Error: ${escapeHtml(String(e))}</div>`;
        }
    }

    function renderTable(services, container) {
        container.innerHTML = '';

        // Filter bar
        const toolbar = document.createElement('div');
        toolbar.className = 'process-toolbar';
        toolbar.innerHTML = `<input type="text" placeholder="Filter services..." value="${escapeHtml(filter)}" />`;
        toolbar.querySelector('input').addEventListener('input', (e) => {
            filter = e.target.value.toLowerCase();
            renderTable(services, container);
        });
        container.appendChild(toolbar);

        // Filter
        let filtered = services;
        if (filter) {
            filtered = services.filter(s =>
                s.name.toLowerCase().includes(filter) ||
                s.display_name.toLowerCase().includes(filter)
            );
        }

        // Sort
        filtered = [...filtered].sort((a, b) => {
            let cmp = 0;
            switch (sortCol) {
                case 'name': cmp = a.name.localeCompare(b.name); break;
                case 'display_name': cmp = a.display_name.localeCompare(b.display_name); break;
                case 'status': cmp = a.status.localeCompare(b.status); break;
                case 'startup_type': cmp = a.startup_type.localeCompare(b.startup_type); break;
            }
            return sortAsc ? cmp : -cmp;
        });

        const table = document.createElement('table');
        table.className = 'svc-table';
        table.innerHTML = `
            <thead><tr>
                <th data-col="name">Name</th>
                <th data-col="display_name">Display Name</th>
                <th data-col="status">Status</th>
                <th data-col="startup_type">Startup</th>
                <th>Actions</th>
            </tr></thead>
        `;

        table.querySelectorAll('th[data-col]').forEach(th => {
            th.addEventListener('click', () => {
                const col = th.dataset.col;
                if (sortCol === col) sortAsc = !sortAsc;
                else { sortCol = col; sortAsc = true; }
                renderTable(services, container);
            });
        });

        const tbody = document.createElement('tbody');
        filtered.forEach(svc => {
            const tr = document.createElement('tr');
            const statusClass = svc.status === 'Running' ? 'svc-status-running' : 'svc-status-stopped';

            tr.innerHTML = `
                <td>${escapeHtml(svc.name)}</td>
                <td>${escapeHtml(svc.display_name)}</td>
                <td class="${statusClass}">${escapeHtml(svc.status)}</td>
                <td>${escapeHtml(svc.startup_type)}</td>
                <td class="svc-actions">
                    <button class="svc-action-btn" data-action="start" data-name="${escapeAttr(svc.name)}">Start</button>
                    <button class="svc-action-btn" data-action="stop" data-name="${escapeAttr(svc.name)}">Stop</button>
                    <button class="svc-action-btn" data-action="restart" data-name="${escapeAttr(svc.name)}">Restart</button>
                </td>
            `;

            tr.querySelectorAll('.svc-action-btn').forEach(btn => {
                btn.addEventListener('click', async () => {
                    const action = btn.dataset.action;
                    const name = btn.dataset.name;
                    try {
                        await invoke(`${action}_service`, { name });
                        // Refresh after action
                        setTimeout(() => render(container), 1000);
                    } catch (e) {
                        alert(`${action} service failed: ${e}`);
                    }
                });
            });

            tbody.appendChild(tr);
        });

        table.appendChild(tbody);
        container.appendChild(table);
    }

    function escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }

    function escapeAttr(text) {
        return text.replace(/&/g, '&amp;').replace(/"/g, '&quot;')
                   .replace(/'/g, '&#39;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
    }

    return { render };
})();
