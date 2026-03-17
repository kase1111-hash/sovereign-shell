// Per-application session mixer with volume sliders and routing.

const SessionMixer = (() => {
    const { invoke } = window.__TAURI__.core;

    function render(sessions, devices, container) {
        container.innerHTML = '';

        const header = document.createElement('div');
        header.className = 'section-header';
        header.textContent = 'Application Routing';
        container.appendChild(header);

        if (sessions.length === 0) {
            const empty = document.createElement('div');
            empty.style.cssText = 'color:var(--text-muted);padding:12px;font-size:12px;text-align:center;';
            empty.textContent = 'No active audio sessions';
            container.appendChild(empty);
            return;
        }

        const playbackDevices = devices.filter(d => d.device_type === 'playback');

        sessions.forEach(session => {
            container.appendChild(createSessionCard(session, playbackDevices));
        });
    }

    function createSessionCard(session, playbackDevices) {
        const card = document.createElement('div');
        card.className = 'session-card';
        card.dataset.pid = session.process_id;

        const icon = getAppIcon(session.process_name);

        // Build device options
        const deviceOptions = playbackDevices.map(d => {
            const selected = d.id === session.device_id ? 'selected' : '';
            return `<option value="${escapeAttr(d.id)}" ${selected}>${escapeHtml(d.name)}</option>`;
        }).join('');

        card.innerHTML = `
            <div class="session-header">
                <span class="session-icon">${icon}</span>
                <span class="session-name">${escapeHtml(session.display_name)}</span>
                <span class="session-pid">PID ${session.process_id}</span>
                <div class="volume-control">
                    <button class="mute-btn ${session.is_muted ? 'muted' : ''}"
                            data-pid="${session.process_id}"
                            data-muted="${session.is_muted}">
                        ${session.is_muted ? '\u{1F507}' : '\u{1F509}'}
                    </button>
                    <input type="range" class="volume-slider" min="0" max="100"
                           value="${Math.round(session.volume * 100)}"
                           data-pid="${session.process_id}" />
                    <span class="volume-value">${Math.round(session.volume * 100)}%</span>
                </div>
            </div>
            <div class="session-routing">
                <span class="routing-arrow">\u2192</span>
                <select class="routing-select" data-pid="${session.process_id}">
                    ${deviceOptions}
                </select>
            </div>
            <div class="level-meter-container" data-pid="${session.process_id}">
                <div class="level-meter-bar" style="width:${session.peak_level * 100}%"></div>
                <div class="level-meter-peak" style="left:${session.peak_level * 100}%"></div>
            </div>
        `;

        // Volume slider
        const slider = card.querySelector('.volume-slider');
        const valueLabel = card.querySelector('.volume-value');
        slider.addEventListener('input', async (e) => {
            const level = parseInt(e.target.value) / 100;
            valueLabel.textContent = e.target.value + '%';
            try {
                await invoke('set_session_volume', { pid: session.process_id, level });
            } catch (err) {
                console.error('Set session volume error:', err);
            }
        });

        // Mute button
        const muteBtn = card.querySelector('.mute-btn');
        muteBtn.addEventListener('click', async () => {
            const nowMuted = muteBtn.dataset.muted !== 'true';
            try {
                await invoke('set_session_mute', { pid: session.process_id, muted: nowMuted });
                muteBtn.dataset.muted = String(nowMuted);
                muteBtn.classList.toggle('muted', nowMuted);
                muteBtn.textContent = nowMuted ? '\u{1F507}' : '\u{1F509}';
            } catch (err) {
                console.error('Set session mute error:', err);
            }
        });

        return card;
    }

    function getAppIcon(processName) {
        const name = processName.toLowerCase().replace('.exe', '');
        const icons = {
            'spotify': '\u{1F3B5}',
            'discord': '\u{1F3AE}',
            'firefox': '\u{1F310}',
            'chrome': '\u{1F310}',
            'msedge': '\u{1F310}',
            'brave': '\u{1F310}',
            'vlc': '\u{1F3AC}',
            'obs': '\u{1F3A5}',
            'teams': '\u{1F4AC}',
            'zoom': '\u{1F4AC}',
            'slack': '\u{1F4AC}',
            'steam': '\u{1F3AE}',
        };
        return icons[name] || '\u{2699}';
    }

    function escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }

    function escapeAttr(text) {
        return text.replace(/"/g, '&quot;');
    }

    return { render };
})();
