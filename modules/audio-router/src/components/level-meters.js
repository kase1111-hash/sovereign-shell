// Real-time audio level meter visualization.

const LevelMeters = (() => {
    const { invoke } = window.__TAURI__.core;
    let peakHold = {}; // PID -> peak value with decay
    let updateTimer = null;

    function startUpdating(intervalMs) {
        if (updateTimer) clearInterval(updateTimer);

        updateTimer = setInterval(async () => {
            try {
                const snapshot = await invoke('get_peak_levels');
                updateMeters(snapshot.levels);
            } catch (e) {
                // Ignore polling errors silently
            }
        }, intervalMs || 50);
    }

    function stopUpdating() {
        if (updateTimer) {
            clearInterval(updateTimer);
            updateTimer = null;
        }
    }

    function updateMeters(levels) {
        const containers = document.querySelectorAll('.level-meter-container');

        containers.forEach(container => {
            const pid = parseInt(container.dataset.pid);
            const levelData = levels.find(l => l.process_id === pid);
            const peak = levelData ? levelData.peak : 0;

            // Update bar
            const bar = container.querySelector('.level-meter-bar');
            if (bar) {
                bar.style.width = (peak * 100) + '%';
            }

            // Update peak hold (decay slowly)
            if (!peakHold[pid] || peak > peakHold[pid]) {
                peakHold[pid] = peak;
            } else {
                peakHold[pid] = Math.max(0, peakHold[pid] - 0.02); // Decay
            }

            const peakIndicator = container.querySelector('.level-meter-peak');
            if (peakIndicator) {
                peakIndicator.style.left = (peakHold[pid] * 100) + '%';
            }
        });
    }

    return { startUpdating, stopUpdating };
})();
