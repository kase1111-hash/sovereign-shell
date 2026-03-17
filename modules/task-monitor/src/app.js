// Sovereign Task Monitor — Main Application Logic.

const App = (() => {
    const { invoke } = window.__TAURI__.core;
    const { getCurrentWindow } = window.__TAURI__.window;

    let currentView = 'processes';
    let updateInterval = null;
    let config = {};

    // ── View Switching ──────────────────────────────────────────────

    function switchView(view) {
        currentView = view;

        document.querySelectorAll('.view-tab').forEach(t => {
            t.classList.toggle('active', t.dataset.view === view);
        });
        document.querySelectorAll('.view').forEach(v => {
            v.classList.toggle('active', v.id === 'view-' + view);
        });

        // Initialize view content on first switch
        switch (view) {
            case 'processes': refreshProcesses(); break;
            case 'performance': initPerformance(); break;
            case 'services': ServiceList.render(document.getElementById('view-services')); break;
            case 'file-locks': FileLockFinder.render(document.getElementById('view-file-locks')); break;
        }
    }

    // ── Process View ────────────────────────────────────────────────

    async function refreshProcesses() {
        const container = document.getElementById('view-processes');

        const callbacks = {
            onRefresh: refreshProcesses,
            onSelect: (proc) => {},
            onKill: async (pid) => {
                if (config.confirm_kill !== false) {
                    if (!confirm(`Kill process ${pid}?`)) return;
                }
                try {
                    await invoke('kill_process', { pid });
                    setTimeout(refreshProcesses, 300);
                } catch (e) {
                    alert('Kill failed: ' + e);
                }
            },
            onViewChange: (mode) => refreshProcesses(),
            onContextMenu: showProcessContextMenu,
        };

        if (ProcessTable.isTreeMode()) {
            try {
                const processes = await invoke('get_processes');
                ProcessTable.render(processes, container, callbacks);
                const tree = await invoke('get_process_tree');
                ProcessTree.render(tree, container, callbacks);
                updateProcessStatus(processes);
            } catch (e) {
                console.error('Process tree error:', e);
            }
        } else {
            try {
                const processes = await invoke('get_processes');
                ProcessTable.render(processes, container, callbacks);
                updateProcessStatus(processes);
            } catch (e) {
                console.error('Process list error:', e);
            }
        }
    }

    function updateProcessStatus(processes) {
        document.getElementById('status-processes').textContent = `${processes.length} processes`;
    }

    function showProcessContextMenu(e, proc) {
        const menu = document.createElement('div');
        menu.className = 'ctx-menu';
        menu.style.left = e.clientX + 'px';
        menu.style.top = e.clientY + 'px';

        const items = [
            { label: 'Kill Process', action: () => killProcess(proc.pid), danger: true },
            { label: 'Kill Process Tree', action: () => killProcessTree(proc.pid), danger: true },
            { separator: true },
            { label: 'Suspend', action: () => suspendProcess(proc.pid) },
            { label: 'Resume', action: () => resumeProcess(proc.pid) },
            { separator: true },
            { label: 'Set Priority: High', action: () => setPriority(proc.pid, 'high') },
            { label: 'Set Priority: Normal', action: () => setPriority(proc.pid, 'normal') },
            { label: 'Set Priority: Low', action: () => setPriority(proc.pid, 'idle') },
            { separator: true },
            { label: 'Copy PID', action: () => navigator.clipboard.writeText(String(proc.pid)) },
            { label: 'Copy Path', action: () => navigator.clipboard.writeText(proc.exe) },
        ];

        items.forEach(item => {
            if (item.separator) {
                menu.innerHTML += '<div class="ctx-separator"></div>';
                return;
            }
            const el = document.createElement('div');
            el.className = 'ctx-item' + (item.danger ? ' danger' : '');
            el.textContent = item.label;
            el.addEventListener('click', () => {
                item.action();
                menu.remove();
            });
            menu.appendChild(el);
        });

        document.body.appendChild(menu);

        const dismiss = () => { menu.remove(); document.removeEventListener('click', dismiss); };
        setTimeout(() => document.addEventListener('click', dismiss), 0);
    }

    async function killProcess(pid) {
        try {
            await invoke('kill_process', { pid });
            setTimeout(refreshProcesses, 300);
        } catch (e) { alert('Kill failed: ' + e); }
    }

    async function killProcessTree(pid) {
        try {
            const count = await invoke('kill_process_tree', { pid });
            setTimeout(refreshProcesses, 300);
        } catch (e) { alert('Kill tree failed: ' + e); }
    }

    async function suspendProcess(pid) {
        try {
            await invoke('suspend_process', { pid });
        } catch (e) { alert('Suspend failed: ' + e); }
    }

    async function resumeProcess(pid) {
        try {
            await invoke('resume_process', { pid });
        } catch (e) { alert('Resume failed: ' + e); }
    }

    async function setPriority(pid, priority) {
        try {
            await invoke('set_process_priority', { pid, priority });
        } catch (e) { alert('Set priority failed: ' + e); }
    }

    // ── Performance View ────────────────────────────────────────────

    let perfInitialized = false;

    function initPerformance() {
        if (!perfInitialized) {
            SystemGraphs.render(document.getElementById('view-performance'));
            perfInitialized = true;
        }
    }

    async function refreshStats() {
        try {
            const stats = await invoke('get_system_stats');
            document.getElementById('status-cpu').textContent = `CPU: ${stats.cpu.total_percent.toFixed(1)}%`;
            document.getElementById('status-memory').textContent = `Memory: ${stats.memory.percent.toFixed(1)}%`;

            if (currentView === 'performance') {
                SystemGraphs.update(stats);
            }
        } catch (e) {
            console.error('Stats error:', e);
        }
    }

    // ── Update Loop ─────────────────────────────────────────────────

    function startUpdateLoop(intervalMs) {
        if (updateInterval) clearInterval(updateInterval);

        updateInterval = setInterval(() => {
            refreshStats();
            if (currentView === 'processes') {
                refreshProcesses();
            }
        }, intervalMs);
    }

    // ── Init ────────────────────────────────────────────────────────

    async function init() {
        // Load config
        try {
            config = await invoke('get_config');
        } catch (e) {
            config = { general: { update_interval_ms: 1000 } };
        }

        // Window controls
        const win = getCurrentWindow();
        document.getElementById('btn-minimize').addEventListener('click', () => win.minimize());
        document.getElementById('btn-maximize').addEventListener('click', async () => {
            if (await win.isMaximized()) win.unmaximize();
            else win.maximize();
        });
        document.getElementById('btn-close').addEventListener('click', () => win.close());

        // View tab switching
        document.querySelectorAll('.view-tab').forEach(tab => {
            tab.addEventListener('click', () => switchView(tab.dataset.view));
        });

        // Initial render
        switchView(config.general?.default_view || 'processes');

        // Start update loop
        const interval = config.general?.update_interval_ms || 1000;
        startUpdateLoop(interval);

        // Initial stats
        refreshStats();
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }

    return { switchView };
})();
