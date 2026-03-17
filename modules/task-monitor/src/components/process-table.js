// Flat sortable process table.

const ProcessTable = (() => {
    let sortCol = 'cpu_percent';
    let sortAsc = false;
    let filter = '';
    let selectedPid = null;
    let treeMode = false;

    function render(processes, container, callbacks) {
        container.innerHTML = '';

        // Toolbar
        const toolbar = document.createElement('div');
        toolbar.className = 'process-toolbar';
        toolbar.innerHTML = `
            <input type="text" placeholder="Filter processes..." id="proc-filter" value="${escapeHtml(filter)}" />
            <button class="toolbar-btn ${treeMode ? '' : 'active'}" id="btn-flat">List</button>
            <button class="toolbar-btn ${treeMode ? 'active' : ''}" id="btn-tree">Tree</button>
            <button class="toolbar-btn danger" id="btn-kill" title="Kill selected process">Kill</button>
        `;
        container.appendChild(toolbar);

        document.getElementById('proc-filter').addEventListener('input', (e) => {
            filter = e.target.value.toLowerCase();
            callbacks.onRefresh();
        });

        document.getElementById('btn-flat').addEventListener('click', () => {
            treeMode = false;
            callbacks.onViewChange('flat');
        });

        document.getElementById('btn-tree').addEventListener('click', () => {
            treeMode = true;
            callbacks.onViewChange('tree');
        });

        document.getElementById('btn-kill').addEventListener('click', () => {
            if (selectedPid) callbacks.onKill(selectedPid);
        });

        if (treeMode) return; // Tree view is handled by ProcessTree

        // Filter
        let filtered = processes;
        if (filter) {
            filtered = processes.filter(p =>
                p.name.toLowerCase().includes(filter) ||
                p.pid.toString().includes(filter) ||
                p.exe.toLowerCase().includes(filter)
            );
        }

        // Sort
        filtered = sortProcesses(filtered);

        // Table
        const table = document.createElement('table');
        table.className = 'proc-table';

        const thead = document.createElement('thead');
        thead.innerHTML = `<tr>
            <th data-col="name">Name ${indicator('name')}</th>
            <th data-col="pid" class="col-pid">PID ${indicator('pid')}</th>
            <th data-col="cpu_percent" class="col-cpu">CPU ${indicator('cpu_percent')}</th>
            <th data-col="memory_bytes" class="col-mem">Memory ${indicator('memory_bytes')}</th>
            <th data-col="disk_read_bytes" class="col-disk">Disk R ${indicator('disk_read_bytes')}</th>
            <th data-col="status" class="col-status">Status ${indicator('status')}</th>
        </tr>`;
        thead.querySelectorAll('th').forEach(th => {
            th.addEventListener('click', () => {
                const col = th.dataset.col;
                if (sortCol === col) sortAsc = !sortAsc;
                else { sortCol = col; sortAsc = col === 'name'; }
                callbacks.onRefresh();
            });
        });
        table.appendChild(thead);

        const tbody = document.createElement('tbody');
        filtered.forEach(proc => {
            const tr = document.createElement('tr');
            if (proc.pid === selectedPid) tr.className = 'selected';

            const cpuClass = proc.cpu_percent > 80 ? 'cpu-high' :
                             proc.cpu_percent > 30 ? 'cpu-med' : 'cpu-low';

            tr.innerHTML = `
                <td>${escapeHtml(proc.name)}</td>
                <td class="col-pid">${proc.pid}</td>
                <td class="col-cpu ${cpuClass}">${proc.cpu_percent.toFixed(1)}%</td>
                <td class="col-mem">${formatBytes(proc.memory_bytes)}</td>
                <td class="col-disk">${formatBytes(proc.disk_read_bytes)}</td>
                <td class="col-status">${escapeHtml(proc.status)}</td>
            `;

            tr.addEventListener('click', () => {
                selectedPid = proc.pid;
                container.querySelectorAll('tr.selected').forEach(r => r.classList.remove('selected'));
                tr.classList.add('selected');
                callbacks.onSelect(proc);
            });

            tr.addEventListener('contextmenu', (e) => {
                e.preventDefault();
                selectedPid = proc.pid;
                callbacks.onContextMenu(e, proc);
            });

            tbody.appendChild(tr);
        });
        table.appendChild(tbody);
        container.appendChild(table);
    }

    function sortProcesses(procs) {
        return [...procs].sort((a, b) => {
            let cmp = 0;
            switch (sortCol) {
                case 'name': cmp = a.name.localeCompare(b.name); break;
                case 'pid': cmp = a.pid - b.pid; break;
                case 'cpu_percent': cmp = a.cpu_percent - b.cpu_percent; break;
                case 'memory_bytes': cmp = a.memory_bytes - b.memory_bytes; break;
                case 'disk_read_bytes': cmp = a.disk_read_bytes - b.disk_read_bytes; break;
                case 'status': cmp = a.status.localeCompare(b.status); break;
            }
            return sortAsc ? cmp : -cmp;
        });
    }

    function indicator(col) {
        if (sortCol !== col) return '';
        return sortAsc ? '\u25B2' : '\u25BC';
    }

    function isTreeMode() { return treeMode; }
    function getSelectedPid() { return selectedPid; }

    function formatBytes(bytes) {
        if (!bytes) return '0 B';
        const units = ['B', 'KB', 'MB', 'GB'];
        const i = Math.floor(Math.log(bytes) / Math.log(1024));
        return (bytes / Math.pow(1024, i)).toFixed(i > 0 ? 1 : 0) + ' ' + units[i];
    }

    function escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }

    return { render, isTreeMode, getSelectedPid };
})();
