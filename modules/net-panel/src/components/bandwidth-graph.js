// Bandwidth Graph component — real-time bandwidth monitoring with canvas graph.
const BandwidthGraph = (function () {
    let container = null;
    let canvas = null;
    let ctx = null;
    let timer = null;
    let history = [];
    const MAX_POINTS = 60;

    function init(el) {
        container = el;
        render();
        startPolling();
    }

    function render() {
        if (!container) return;

        container.innerHTML = `
            <div class="toolbar">
                <span style="font-weight:500">Real-time Bandwidth</span>
                <span id="bw-rx" style="margin-left:auto;color:var(--success);font-family:var(--mono);font-size:12px">RX: 0 B/s</span>
                <span id="bw-tx" style="margin-left:12px;color:var(--accent);font-family:var(--mono);font-size:12px">TX: 0 B/s</span>
            </div>
            <div class="graph-container">
                <canvas id="bw-canvas" height="250"></canvas>
            </div>
            <div id="bw-interfaces" style="margin-top:10px"></div>`;

        canvas = container.querySelector('#bw-canvas');
        ctx = canvas.getContext('2d');
        resizeCanvas();
        window.addEventListener('resize', resizeCanvas);
    }

    function resizeCanvas() {
        if (!canvas) return;
        canvas.width = canvas.parentElement.clientWidth;
        drawGraph();
    }

    function startPolling() {
        poll();
        timer = setInterval(poll, 1000);
    }

    function stop() {
        if (timer) clearInterval(timer);
        timer = null;
    }

    async function poll() {
        try {
            const snap = await window.__TAURI__.core.invoke('get_bandwidth');
            history.push(snap);
            if (history.length > MAX_POINTS) history.shift();
            updateLabels(snap);
            drawGraph();
            renderInterfaces(snap);
        } catch (e) {
            console.error('Bandwidth poll failed:', e);
        }
    }

    function updateLabels(snap) {
        const rxEl = container.querySelector('#bw-rx');
        const txEl = container.querySelector('#bw-tx');
        if (rxEl) rxEl.textContent = 'RX: ' + formatBytes(snap.total_rx_bytes_sec) + '/s';
        if (txEl) txEl.textContent = 'TX: ' + formatBytes(snap.total_tx_bytes_sec) + '/s';
    }

    function drawGraph() {
        if (!ctx || !canvas) return;
        const w = canvas.width;
        const h = canvas.height;
        const pad = { top: 20, right: 10, bottom: 25, left: 60 };

        ctx.clearRect(0, 0, w, h);

        // Find max value for scale
        let maxVal = 1024; // minimum 1KB scale
        history.forEach(s => {
            maxVal = Math.max(maxVal, s.total_rx_bytes_sec, s.total_tx_bytes_sec);
        });
        maxVal *= 1.2;

        const graphW = w - pad.left - pad.right;
        const graphH = h - pad.top - pad.bottom;

        // Grid lines
        ctx.strokeStyle = '#2a2a3e';
        ctx.lineWidth = 1;
        for (let i = 0; i <= 4; i++) {
            const y = pad.top + (graphH / 4) * i;
            ctx.beginPath();
            ctx.moveTo(pad.left, y);
            ctx.lineTo(w - pad.right, y);
            ctx.stroke();

            // Label
            const val = maxVal - (maxVal / 4) * i;
            ctx.fillStyle = '#606070';
            ctx.font = '10px monospace';
            ctx.textAlign = 'right';
            ctx.fillText(formatBytes(val), pad.left - 6, y + 4);
        }

        if (history.length < 2) return;

        const stepX = graphW / (MAX_POINTS - 1);

        // RX line (green)
        drawLine(history.map(s => s.total_rx_bytes_sec), maxVal, stepX, pad, graphH, '#00e676', 'rgba(0,230,118,0.1)');

        // TX line (cyan)
        drawLine(history.map(s => s.total_tx_bytes_sec), maxVal, stepX, pad, graphH, '#00d4ff', 'rgba(0,212,255,0.1)');

        // Time labels
        ctx.fillStyle = '#606070';
        ctx.font = '10px monospace';
        ctx.textAlign = 'center';
        const startIdx = MAX_POINTS - history.length;
        ctx.fillText(`-${MAX_POINTS}s`, pad.left, h - 5);
        ctx.fillText('now', w - pad.right, h - 5);
    }

    function drawLine(values, maxVal, stepX, pad, graphH, color, fillColor) {
        if (values.length < 2) return;
        const startIdx = MAX_POINTS - values.length;

        ctx.beginPath();
        ctx.strokeStyle = color;
        ctx.lineWidth = 1.5;

        values.forEach((v, i) => {
            const x = pad.left + (startIdx + i) * stepX;
            const y = pad.top + graphH - (v / maxVal) * graphH;
            if (i === 0) ctx.moveTo(x, y);
            else ctx.lineTo(x, y);
        });
        ctx.stroke();

        // Fill
        ctx.lineTo(pad.left + (startIdx + values.length - 1) * stepX, pad.top + graphH);
        ctx.lineTo(pad.left + startIdx * stepX, pad.top + graphH);
        ctx.closePath();
        ctx.fillStyle = fillColor;
        ctx.fill();
    }

    function renderInterfaces(snap) {
        const el = container.querySelector('#bw-interfaces');
        if (!el) return;
        if (!snap.interfaces || snap.interfaces.length === 0) {
            el.innerHTML = '';
            return;
        }

        const rows = snap.interfaces.map(iface => `
            <tr>
                <td>${esc(iface.name)}</td>
                <td style="font-family:var(--mono);color:var(--success)">${formatBytes(iface.rx_bytes_sec)}/s</td>
                <td style="font-family:var(--mono);color:var(--accent)">${formatBytes(iface.tx_bytes_sec)}/s</td>
                <td style="font-family:var(--mono);color:var(--text-muted)">${formatBytes(iface.rx_bytes)}</td>
                <td style="font-family:var(--mono);color:var(--text-muted)">${formatBytes(iface.tx_bytes)}</td>
            </tr>`).join('');

        el.innerHTML = `
            <table>
                <thead><tr>
                    <th>Interface</th>
                    <th>RX Rate</th>
                    <th>TX Rate</th>
                    <th>RX Total</th>
                    <th>TX Total</th>
                </tr></thead>
                <tbody>${rows}</tbody>
            </table>`;
    }

    function formatBytes(bytes) {
        if (bytes < 1024) return bytes + ' B';
        if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB';
        if (bytes < 1024 * 1024 * 1024) return (bytes / (1024 * 1024)).toFixed(1) + ' MB';
        return (bytes / (1024 * 1024 * 1024)).toFixed(2) + ' GB';
    }

    function esc(s) {
        if (!s) return '';
        const d = document.createElement('div');
        d.textContent = s;
        return d.innerHTML;
    }

    return { init, stop };
})();
