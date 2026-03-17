// Sovereign Shell — Network Panel app entry point.
(function () {
    const { getCurrentWindow } = window.__TAURI__.window;

    // Titlebar controls
    document.getElementById('btn-minimize').addEventListener('click', () => getCurrentWindow().minimize());
    document.getElementById('btn-maximize').addEventListener('click', async () => {
        const win = getCurrentWindow();
        (await win.isMaximized()) ? win.unmaximize() : win.maximize();
    });
    document.getElementById('btn-close').addEventListener('click', () => getCurrentWindow().close());

    // View navigation
    const views = {
        adapters: { el: document.getElementById('view-adapters'), component: AdapterList },
        connections: { el: document.getElementById('view-connections'), component: ConnectionTable },
        firewall: { el: document.getElementById('view-firewall'), component: FirewallRules },
        diagnostics: { el: document.getElementById('view-diagnostics'), component: Diagnostics },
        bandwidth: { el: document.getElementById('view-bandwidth'), component: BandwidthGraph },
    };

    let currentView = 'adapters';
    const initialized = new Set();

    function switchView(name) {
        if (!views[name]) return;
        document.querySelectorAll('.nav-btn').forEach(b => b.classList.toggle('active', b.dataset.view === name));
        document.querySelectorAll('.view').forEach(v => v.classList.remove('active'));
        views[name].el.classList.add('active');

        if (!initialized.has(name)) {
            views[name].component.init(views[name].el);
            initialized.add(name);
        } else if (views[name].component.refresh) {
            views[name].component.refresh();
        }

        // Stop bandwidth polling when not visible
        if (currentView === 'bandwidth' && name !== 'bandwidth' && BandwidthGraph.stop) {
            BandwidthGraph.stop();
        }

        currentView = name;
    }

    document.querySelectorAll('.nav-btn').forEach(btn => {
        btn.addEventListener('click', () => switchView(btn.dataset.view));
    });

    // Initialize default view
    switchView('adapters');
})();
