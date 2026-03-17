// Sovereign Audio Router — Main Application Logic.

const App = (() => {
    const { invoke } = window.__TAURI__.core;
    const { getCurrentWindow } = window.__TAURI__.window;

    let refreshTimer = null;
    let config = {};

    // ── Data Refresh ────────────────────────────────────────────────

    async function refresh() {
        try {
            const [devices, sessions] = await Promise.all([
                invoke('get_devices'),
                invoke('get_sessions'),
            ]);

            DeviceList.render(devices, document.getElementById('devices-section'));
            SessionMixer.render(sessions, devices, document.getElementById('sessions-section'));

            // Update status bar
            document.getElementById('status-devices').textContent =
                `${devices.length} device${devices.length !== 1 ? 's' : ''}`;
            document.getElementById('status-sessions').textContent =
                `${sessions.length} session${sessions.length !== 1 ? 's' : ''}`;
        } catch (e) {
            console.error('Refresh error:', e);
        }
    }

    function startRefreshLoop(intervalMs) {
        if (refreshTimer) clearInterval(refreshTimer);
        // Refresh sessions and devices every second
        refreshTimer = setInterval(refresh, intervalMs || 1000);
    }

    // ── Init ────────────────────────────────────────────────────────

    async function init() {
        // Load config
        try {
            config = await invoke('get_config');
        } catch (e) {
            config = { general: { update_interval_ms: 50 } };
        }

        // Window controls
        const win = getCurrentWindow();
        document.getElementById('btn-minimize').addEventListener('click', () => win.minimize());
        document.getElementById('btn-maximize').addEventListener('click', async () => {
            if (await win.isMaximized()) win.unmaximize();
            else win.maximize();
        });
        document.getElementById('btn-close').addEventListener('click', () => win.close());

        // Initial render
        await refresh();

        // Start update loops
        startRefreshLoop(1000);
        LevelMeters.startUpdating(config.general?.update_interval_ms || 50);
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }

    return { refresh };
})();
