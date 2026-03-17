// Real-time system performance graphs using Canvas.

const SystemGraphs = (() => {
    const HISTORY_LENGTH = 60; // 60 seconds of data
    const history = {
        cpu: [],
        memory: [],
        diskRead: [],
        diskWrite: [],
        netRx: [],
        netTx: [],
    };

    let canvases = {};

    function render(container) {
        container.innerHTML = '';
        const grid = document.createElement('div');
        grid.className = 'perf-grid';

        // CPU card
        grid.appendChild(createCard('cpu', 'CPU Usage', '--cpu-color'));
        // Memory card
        grid.appendChild(createCard('memory', 'Memory Usage', '--mem-color'));
        // Disk card
        grid.appendChild(createCard('disk', 'Disk I/O', '--disk-color'));
        // Network card
        grid.appendChild(createCard('network', 'Network', '--net-color'));

        container.appendChild(grid);
    }

    function createCard(id, title, colorVar) {
        const card = document.createElement('div');
        card.className = 'perf-card';
        card.innerHTML = `
            <div class="perf-card-title">${title}</div>
            <div class="perf-card-value" id="perf-${id}-value">--</div>
            <canvas id="perf-${id}-canvas" height="80"></canvas>
            <div class="perf-card-detail" id="perf-${id}-detail"></div>
        `;
        return card;
    }

    function update(stats) {
        // Push to history
        history.cpu.push(stats.cpu.total_percent);
        history.memory.push(stats.memory.percent);
        history.diskRead.push(0); // Delta not available from single snapshot
        history.diskWrite.push(0);
        history.netRx.push(stats.network.rx_bytes);
        history.netTx.push(stats.network.tx_bytes);

        // Trim history
        Object.values(history).forEach(arr => {
            while (arr.length > HISTORY_LENGTH) arr.shift();
        });

        // Update CPU
        updateValue('cpu', stats.cpu.total_percent.toFixed(1) + '%');
        updateDetail('cpu',
            `${stats.cpu.brand} | ${stats.cpu.logical_cores} cores | ${stats.cpu.frequency_mhz} MHz`
        );
        drawGraph('cpu', history.cpu, 100, '#00d4ff');

        // Update Memory
        updateValue('memory', stats.memory.percent.toFixed(1) + '%');
        updateDetail('memory',
            `${formatBytes(stats.memory.used)} / ${formatBytes(stats.memory.total)} | Swap: ${formatBytes(stats.memory.swap_used)}`
        );
        drawGraph('memory', history.memory, 100, '#cc66ff');

        // Update Disk
        const diskInfo = stats.disks.map(d =>
            `${d.mount_point}: ${formatBytes(d.available_bytes)} free`
        ).join(' | ');
        updateValue('disk', stats.disks.length + ' volumes');
        updateDetail('disk', diskInfo);
        drawGraph('disk', history.diskRead, Math.max(...history.diskRead, 1), '#44ff88');

        // Update Network
        updateValue('network', `\u2193 ${formatBytes(stats.network.rx_bytes)} \u2191 ${formatBytes(stats.network.tx_bytes)}`);
        updateDetail('network',
            `${stats.network.interfaces.length} interfaces`
        );
        drawGraph('network', history.netRx, Math.max(...history.netRx, 1), '#ffaa33');
    }

    function updateValue(id, text) {
        const el = document.getElementById(`perf-${id}-value`);
        if (el) el.textContent = text;
    }

    function updateDetail(id, text) {
        const el = document.getElementById(`perf-${id}-detail`);
        if (el) el.textContent = text;
    }

    function drawGraph(id, data, maxVal, color) {
        const canvas = document.getElementById(`perf-${id}-canvas`);
        if (!canvas) return;

        const ctx = canvas.getContext('2d');
        const w = canvas.offsetWidth;
        const h = canvas.height;
        canvas.width = w;

        ctx.clearRect(0, 0, w, h);

        if (data.length < 2) return;

        // Fill
        ctx.beginPath();
        ctx.moveTo(0, h);

        const step = w / (HISTORY_LENGTH - 1);
        const offset = HISTORY_LENGTH - data.length;

        data.forEach((val, i) => {
            const x = (offset + i) * step;
            const y = h - (val / maxVal) * h;
            ctx.lineTo(x, y);
        });

        ctx.lineTo((offset + data.length - 1) * step, h);
        ctx.closePath();
        ctx.fillStyle = color + '20'; // Low opacity fill
        ctx.fill();

        // Line
        ctx.beginPath();
        data.forEach((val, i) => {
            const x = (offset + i) * step;
            const y = h - (val / maxVal) * h;
            if (i === 0) ctx.moveTo(x, y);
            else ctx.lineTo(x, y);
        });
        ctx.strokeStyle = color;
        ctx.lineWidth = 1.5;
        ctx.stroke();
    }

    function formatBytes(bytes) {
        if (!bytes) return '0 B';
        const units = ['B', 'KB', 'MB', 'GB', 'TB'];
        const i = Math.floor(Math.log(bytes) / Math.log(1024));
        return (bytes / Math.pow(1024, i)).toFixed(i > 0 ? 1 : 0) + ' ' + units[i];
    }

    return { render, update };
})();
