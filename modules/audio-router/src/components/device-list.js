// Audio device list component.

const DeviceList = (() => {
    const { invoke } = window.__TAURI__.core;

    function render(devices, container) {
        container.innerHTML = '';

        const header = document.createElement('div');
        header.className = 'section-header';
        header.textContent = 'Output Devices';
        container.appendChild(header);

        const playback = devices.filter(d => d.device_type === 'playback');
        const capture = devices.filter(d => d.device_type === 'capture');

        playback.forEach(device => {
            container.appendChild(createDeviceCard(device));
        });

        if (capture.length > 0) {
            const capHeader = document.createElement('div');
            capHeader.className = 'section-header';
            capHeader.style.marginTop = '12px';
            capHeader.textContent = 'Input Devices';
            container.appendChild(capHeader);

            capture.forEach(device => {
                container.appendChild(createDeviceCard(device));
            });
        }
    }

    function createDeviceCard(device) {
        const card = document.createElement('div');
        card.className = 'device-card' + (device.is_default ? ' is-default' : '');

        const icon = device.device_type === 'playback' ? '\u{1F50A}' : '\u{1F3A4}';
        const defaultBadge = device.is_default
            ? '<span class="device-default-badge">DEFAULT</span>'
            : '';

        card.innerHTML = `
            <span class="device-icon">${icon}</span>
            <div class="device-info">
                <div class="device-name">${escapeHtml(device.name)}${defaultBadge}</div>
                <div class="device-type">${device.device_type}</div>
            </div>
            <div class="volume-control">
                <button class="mute-btn ${device.is_muted ? 'muted' : ''}"
                        data-device-id="${escapeAttr(device.id)}"
                        data-muted="${device.is_muted}">
                    ${device.is_muted ? '\u{1F507}' : '\u{1F50A}'}
                </button>
                <input type="range" class="volume-slider" min="0" max="100"
                       value="${Math.round(device.volume * 100)}"
                       data-device-id="${escapeAttr(device.id)}" />
                <span class="volume-value">${Math.round(device.volume * 100)}%</span>
            </div>
        `;

        // Volume slider
        const slider = card.querySelector('.volume-slider');
        const valueLabel = card.querySelector('.volume-value');
        slider.addEventListener('input', async (e) => {
            const level = parseInt(e.target.value) / 100;
            valueLabel.textContent = e.target.value + '%';
            try {
                await invoke('set_device_volume', { deviceId: device.id, level });
            } catch (err) {
                console.error('Set device volume error:', err);
            }
        });

        // Mute button
        const muteBtn = card.querySelector('.mute-btn');
        muteBtn.addEventListener('click', async () => {
            const nowMuted = muteBtn.dataset.muted !== 'true';
            try {
                await invoke('set_device_mute', { deviceId: device.id, muted: nowMuted });
                muteBtn.dataset.muted = String(nowMuted);
                muteBtn.classList.toggle('muted', nowMuted);
                muteBtn.textContent = nowMuted ? '\u{1F507}' : '\u{1F50A}';
            } catch (err) {
                console.error('Set device mute error:', err);
            }
        });

        return card;
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
