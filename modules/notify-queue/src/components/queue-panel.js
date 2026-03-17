// Queue Panel component — full notification list grouped by source.
const QueuePanel = (function () {
    let container = null;
    let grouped = [];
    let timer = null;

    function init(el) {
        container = el;
        render();
        refresh();
        startPolling();
    }

    function startPolling() {
        timer = setInterval(refresh, 2000);
    }

    function stop() {
        if (timer) clearInterval(timer);
        timer = null;
    }

    async function refresh() {
        try {
            grouped = await window.__TAURI__.core.invoke('get_grouped_notifications');
        } catch (e) {
            grouped = [];
        }
        render();
    }

    function sourceIcon(source) {
        const s = source.toLowerCase();
        if (s.includes('search')) return '🔍';
        if (s.includes('explorer')) return '📁';
        if (s.includes('task') || s.includes('monitor')) return '📊';
        if (s.includes('audio')) return '🔊';
        if (s.includes('net')) return '🌐';
        return '🔔';
    }

    function priorityBadge(p) {
        return `<span class="badge badge-${esc(p)}">${esc(p)}</span>`;
    }

    function timeAgo(ts) {
        const now = Date.now();
        const then = new Date(ts).getTime();
        const diff = Math.floor((now - then) / 1000);
        if (diff < 60) return 'just now';
        if (diff < 3600) return Math.floor(diff / 60) + 'm ago';
        if (diff < 86400) return Math.floor(diff / 3600) + 'h ago';
        return Math.floor(diff / 86400) + 'd ago';
    }

    function render() {
        if (!container) return;

        const totalUnread = grouped.reduce((sum, g) => sum + g.unread_count, 0);

        const toolbar = `
            <div class="toolbar">
                <span style="font-weight:500">Notifications</span>
                ${totalUnread > 0 ? `<span class="badge badge-normal">${totalUnread} unread</span>` : ''}
                <span style="margin-left:auto"></span>
                <button class="btn btn-sm" id="q-refresh">Refresh</button>
                <button class="btn btn-sm btn-danger" id="q-clear-all">Clear All</button>
            </div>`;

        if (grouped.length === 0) {
            container.innerHTML = toolbar + `
                <div class="empty-state">
                    <div class="empty-state-icon">🔔</div>
                    No notifications
                </div>`;
            bindToolbar();
            return;
        }

        let html = toolbar;

        for (const group of grouped) {
            html += `
                <div class="group-header">
                    <span class="group-name">${sourceIcon(group.source)} ${esc(group.source)}</span>
                    <div style="display:flex;gap:4px;align-items:center">
                        <span class="group-count">${group.notifications.length}</span>
                        <button class="btn-icon" data-dismiss-source="${esc(group.source)}" title="Dismiss all from ${esc(group.source)}">&times;</button>
                    </div>
                </div>`;

            for (const notif of group.notifications) {
                const unreadClass = notif.read ? '' : 'unread';
                html += `
                    <div class="notif-card ${unreadClass}" data-id="${esc(notif.id)}">
                        <div class="notif-icon">${sourceIcon(notif.source)}</div>
                        <div class="notif-content">
                            <div class="notif-title">${esc(notif.title)}</div>
                            <div class="notif-body">${esc(notif.body)}</div>
                            <div class="notif-meta">
                                ${priorityBadge(notif.priority)}
                                <span>${timeAgo(notif.timestamp)}</span>
                            </div>
                        </div>
                        <div class="notif-actions">
                            <button class="btn-icon" data-dismiss="${esc(notif.id)}" title="Dismiss">&times;</button>
                        </div>
                    </div>`;
            }
        }

        container.innerHTML = html;
        bindToolbar();
        bindActions();
    }

    function bindToolbar() {
        container.querySelector('#q-refresh')?.addEventListener('click', refresh);
        container.querySelector('#q-clear-all')?.addEventListener('click', async () => {
            try {
                await window.__TAURI__.core.invoke('dismiss_all');
            } catch (e) {}
            refresh();
        });
    }

    function bindActions() {
        container.querySelectorAll('[data-dismiss]').forEach(btn => {
            btn.addEventListener('click', async (e) => {
                e.stopPropagation();
                const id = btn.dataset.dismiss;
                try {
                    await window.__TAURI__.core.invoke('dismiss_notification', { id });
                } catch (e) {}
                refresh();
            });
        });

        container.querySelectorAll('[data-dismiss-source]').forEach(btn => {
            btn.addEventListener('click', async () => {
                const source = btn.dataset.dismissSource;
                try {
                    await window.__TAURI__.core.invoke('dismiss_by_source', { source });
                } catch (e) {}
                refresh();
            });
        });

        // Click on card to mark as read
        container.querySelectorAll('.notif-card').forEach(card => {
            card.addEventListener('click', async () => {
                const id = card.dataset.id;
                try {
                    await window.__TAURI__.core.invoke('mark_read', { id });
                } catch (e) {}
                card.classList.remove('unread');
            });
        });
    }

    function esc(s) {
        if (!s) return '';
        const d = document.createElement('div');
        d.textContent = s;
        return d.innerHTML;
    }

    return { init, refresh, stop };
})();
