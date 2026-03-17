// Diagnostics component — ping, traceroute, DNS lookup.
const Diagnostics = (function () {
    let container = null;
    let activeTab = 'ping';
    let output = '';
    let running = false;

    function init(el) {
        container = el;
        render();
    }

    function render() {
        if (!container) return;

        container.innerHTML = `
            <div class="toolbar">
                <button class="btn ${activeTab === 'ping' ? 'btn-accent' : ''}" data-tab="ping">Ping</button>
                <button class="btn ${activeTab === 'traceroute' ? 'btn-accent' : ''}" data-tab="traceroute">Traceroute</button>
                <button class="btn ${activeTab === 'dns' ? 'btn-accent' : ''}" data-tab="dns">DNS Lookup</button>
            </div>
            <div id="diag-form" style="margin-bottom:10px">
                ${renderForm()}
            </div>
            <div class="output-box" id="diag-output">${output || 'Results will appear here...'}</div>`;

        bindTabs();
        bindForm();
    }

    function renderForm() {
        if (activeTab === 'ping') {
            return `
                <div class="toolbar">
                    <input type="text" id="diag-host" placeholder="Host (e.g. 8.8.8.8)" style="width:220px">
                    <input type="number" id="diag-count" value="5" min="1" max="100" style="width:70px" title="Count">
                    <button class="btn btn-accent" id="diag-run" ${running ? 'disabled' : ''}>
                        ${running ? 'Running...' : 'Ping'}
                    </button>
                </div>`;
        }
        if (activeTab === 'traceroute') {
            return `
                <div class="toolbar">
                    <input type="text" id="diag-host" placeholder="Host (e.g. 8.8.8.8)" style="width:220px">
                    <input type="number" id="diag-hops" value="30" min="1" max="64" style="width:70px" title="Max hops">
                    <button class="btn btn-accent" id="diag-run" ${running ? 'disabled' : ''}>
                        ${running ? 'Running...' : 'Trace'}
                    </button>
                </div>`;
        }
        // dns
        return `
            <div class="toolbar">
                <input type="text" id="diag-host" placeholder="Domain (e.g. example.com)" style="width:200px">
                <select id="diag-rtype">
                    <option value="A">A</option>
                    <option value="AAAA">AAAA</option>
                    <option value="MX">MX</option>
                    <option value="NS">NS</option>
                    <option value="TXT">TXT</option>
                    <option value="CNAME">CNAME</option>
                </select>
                <input type="text" id="diag-server" placeholder="DNS server (optional)" style="width:140px">
                <button class="btn btn-accent" id="diag-run" ${running ? 'disabled' : ''}>
                    ${running ? 'Running...' : 'Lookup'}
                </button>
            </div>`;
    }

    function bindTabs() {
        container.querySelectorAll('[data-tab]').forEach(btn => {
            btn.addEventListener('click', () => {
                activeTab = btn.dataset.tab;
                output = '';
                render();
            });
        });
    }

    function bindForm() {
        const runBtn = container.querySelector('#diag-run');
        if (!runBtn) return;
        runBtn.addEventListener('click', () => {
            const host = container.querySelector('#diag-host')?.value?.trim();
            if (!host) return;

            if (activeTab === 'ping') runPing(host);
            else if (activeTab === 'traceroute') runTraceroute(host);
            else runDns(host);
        });
    }

    async function runPing(host) {
        const count = parseInt(container.querySelector('#diag-count')?.value) || 5;
        running = true;
        output = `Pinging ${esc(host)} with ${count} packets...\n`;
        render();

        try {
            const results = await window.__TAURI__.core.invoke('run_ping', { host, count });
            let text = '';
            let success = 0;
            results.forEach(r => {
                if (r.success) {
                    text += `Reply from ${esc(r.host)}: seq=${r.seq} time=${r.rtt_ms.toFixed(1)}ms ttl=${r.ttl}\n`;
                    success++;
                } else {
                    text += `Request timed out (seq=${r.seq})\n`;
                }
            });
            const loss = results.length > 0 ? ((results.length - success) / results.length * 100).toFixed(0) : 0;
            text += `\n--- ${esc(host)} ping statistics ---\n`;
            text += `${results.length} packets transmitted, ${success} received, ${loss}% loss\n`;
            if (success > 0) {
                const rtts = results.filter(r => r.success).map(r => r.rtt_ms);
                text += `rtt min/avg/max = ${Math.min(...rtts).toFixed(1)}/${(rtts.reduce((a,b)=>a+b,0)/rtts.length).toFixed(1)}/${Math.max(...rtts).toFixed(1)} ms\n`;
            }
            output = text;
        } catch (e) {
            output = `Error: ${esc(String(e))}\n`;
        }
        running = false;
        render();
    }

    async function runTraceroute(host) {
        const maxHops = parseInt(container.querySelector('#diag-hops')?.value) || 30;
        running = true;
        output = `Traceroute to ${esc(host)} (max ${maxHops} hops)...\n`;
        render();

        try {
            const hops = await window.__TAURI__.core.invoke('run_traceroute', { host, maxHops });
            let text = '';
            hops.forEach(h => {
                if (h.timed_out) {
                    text += `${String(h.hop).padStart(2)}  *  *  *\n`;
                } else {
                    const rtts = h.rtt_ms.map(r => r.toFixed(1) + ' ms').join('  ');
                    text += `${String(h.hop).padStart(2)}  ${esc(h.address)}  ${rtts}\n`;
                }
            });
            output = text || 'No hops returned.\n';
        } catch (e) {
            output = `Error: ${esc(String(e))}\n`;
        }
        running = false;
        render();
    }

    async function runDns(host) {
        const recordType = container.querySelector('#diag-rtype')?.value || 'A';
        const server = container.querySelector('#diag-server')?.value?.trim() || null;
        running = true;
        output = `Looking up ${recordType} records for ${esc(host)}...\n`;
        render();

        try {
            const result = await window.__TAURI__.core.invoke('dns_lookup', {
                host, recordType, server,
            });
            let text = `; Server: ${esc(result.server)}\n; Query time: ${result.elapsed_ms}ms\n\n`;
            if (result.answers.length === 0) {
                text += 'No records found.\n';
            } else {
                result.answers.forEach(a => {
                    text += `${esc(a.name)}\t${a.ttl}\t${esc(a.record_type)}\t${esc(a.value)}\n`;
                });
            }
            output = text;
        } catch (e) {
            output = `Error: ${esc(String(e))}\n`;
        }
        running = false;
        render();
    }

    function esc(s) {
        if (!s) return '';
        const d = document.createElement('div');
        d.textContent = s;
        return d.innerHTML;
    }

    return { init };
})();
