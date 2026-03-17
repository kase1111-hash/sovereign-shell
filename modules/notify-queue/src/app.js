// Sovereign Shell — Notification Queue app entry point.
(function () {
    const { getCurrentWindow } = window.__TAURI__.window;

    // Titlebar controls
    document.getElementById('btn-minimize').addEventListener('click', () => getCurrentWindow().minimize());
    document.getElementById('btn-maximize').addEventListener('click', async () => {
        const win = getCurrentWindow();
        (await win.isMaximized()) ? win.unmaximize() : win.maximize();
    });
    document.getElementById('btn-close').addEventListener('click', () => getCurrentWindow().close());

    // Silent mode toggle
    const silentBtn = document.getElementById('btn-silent');
    let silentMode = false;

    async function loadSilentMode() {
        try {
            silentMode = await window.__TAURI__.core.invoke('get_silent_mode');
            updateSilentBtn();
        } catch (e) {}
    }

    function updateSilentBtn() {
        silentBtn.textContent = silentMode ? '🔕' : '🔔';
        silentBtn.classList.toggle('silent-active', silentMode);
        silentBtn.title = silentMode ? 'Silent Mode ON — click to unmute' : 'Silent Mode OFF — click to mute';
    }

    silentBtn.addEventListener('click', async () => {
        silentMode = !silentMode;
        try {
            await window.__TAURI__.core.invoke('set_silent_mode', { enabled: silentMode });
        } catch (e) {}
        updateSilentBtn();
    });

    loadSilentMode();

    // Toast system
    Toast.init();

    // View navigation
    const views = {
        queue: { el: document.getElementById('view-queue'), component: QueuePanel },
        history: { el: document.getElementById('view-history'), component: HistorySearch },
        rules: { el: document.getElementById('view-rules'), component: RulesEditor },
    };

    let currentView = 'queue';
    const initialized = new Set();

    function switchView(name) {
        if (!views[name]) return;
        document.querySelectorAll('.tab-btn').forEach(b => b.classList.toggle('active', b.dataset.view === name));
        document.querySelectorAll('.view').forEach(v => v.classList.remove('active'));
        views[name].el.classList.add('active');

        if (!initialized.has(name)) {
            views[name].component.init(views[name].el);
            initialized.add(name);
        } else if (views[name].component.refresh) {
            views[name].component.refresh();
        }

        currentView = name;
    }

    document.querySelectorAll('.tab-btn').forEach(btn => {
        btn.addEventListener('click', () => switchView(btn.dataset.view));
    });

    // Initialize default view
    switchView('queue');

    // Poll for new notifications and show toasts
    let lastUnreadCount = 0;

    async function checkForNewNotifications() {
        if (silentMode) return;
        try {
            const count = await window.__TAURI__.core.invoke('get_unread_count');
            if (count > lastUnreadCount) {
                // Fetch the newest notifications to toast
                const all = await window.__TAURI__.core.invoke('get_notifications');
                const newOnes = all.filter(n => !n.read).slice(0, count - lastUnreadCount);
                for (const n of newOnes) {
                    Toast.show(n, 5000);
                }
            }
            lastUnreadCount = count;
        } catch (e) {}
    }

    setInterval(checkForNewNotifications, 2000);
})();
