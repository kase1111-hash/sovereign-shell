// Hierarchical process tree view.

const ProcessTree = (() => {
    const expanded = new Set();
    let selectedPid = null;

    function render(treeNodes, container, callbacks) {
        // Reuse toolbar from ProcessTable — only render tree body
        const existing = container.querySelector('.tree-body');
        if (existing) existing.remove();

        const body = document.createElement('div');
        body.className = 'tree-body';

        renderNodes(treeNodes, body, 0, callbacks);
        container.appendChild(body);
    }

    function renderNodes(nodes, parent, depth, callbacks) {
        // Sort nodes by CPU descending at each level
        const sorted = [...nodes].sort((a, b) => b.process.cpu_percent - a.process.cpu_percent);

        sorted.forEach(node => {
            const proc = node.process;
            const hasChildren = node.children && node.children.length > 0;
            const isExpanded = expanded.has(proc.pid);

            const row = document.createElement('div');
            row.className = 'tree-row' + (proc.pid === selectedPid ? ' selected' : '');

            // Indentation
            let indent = '';
            for (let i = 0; i < depth; i++) {
                indent += '<span class="tree-indent"></span>';
            }

            const toggle = hasChildren
                ? `<span class="tree-toggle-btn">${isExpanded ? '\u25BC' : '\u25B6'}</span>`
                : '<span class="tree-indent" style="width:16px"></span>';

            const cpuClass = proc.cpu_percent > 80 ? 'cpu-high' :
                             proc.cpu_percent > 30 ? 'cpu-med' : 'cpu-low';

            row.innerHTML = `
                ${indent}
                ${toggle}
                <span class="tree-name">${escapeHtml(proc.name)}</span>
                <span class="tree-pid">${proc.pid}</span>
                <span class="tree-cpu ${cpuClass}">${proc.cpu_percent.toFixed(1)}%</span>
                <span class="tree-mem">${formatBytes(proc.memory_bytes)}</span>
            `;

            row.addEventListener('click', (e) => {
                if (e.target.classList.contains('tree-toggle-btn')) {
                    if (isExpanded) expanded.delete(proc.pid);
                    else expanded.add(proc.pid);
                    callbacks.onRefresh();
                    return;
                }
                selectedPid = proc.pid;
                callbacks.onSelect(proc);
                callbacks.onRefresh();
            });

            row.addEventListener('contextmenu', (e) => {
                e.preventDefault();
                selectedPid = proc.pid;
                callbacks.onContextMenu(e, proc);
            });

            parent.appendChild(row);

            if (hasChildren && isExpanded) {
                renderNodes(node.children, parent, depth + 1, callbacks);
            }
        });
    }

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

    return { render };
})();
