// Toast component — renders popup toast notifications with auto-dismiss.
const Toast = (function () {
    let container = null;
    let maxVisible = 3;
    let activeToasts = [];

    function init() {
        container = document.getElementById('toast-container');
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

    /**
     * Show a toast notification.
     * @param {object} notif - { id, source, title, body, priority }
     * @param {number} duration - auto-dismiss in milliseconds
     */
    function show(notif, duration) {
        if (!container) init();

        // Limit visible toasts
        while (activeToasts.length >= maxVisible) {
            dismissOldest();
        }

        const el = document.createElement('div');
        el.className = 'toast';
        el.dataset.id = notif.id;

        el.innerHTML = `
            <div class="toast-icon">${sourceIcon(notif.source)}</div>
            <div class="toast-content">
                <div class="toast-title">${esc(notif.title)}</div>
                <div class="toast-body">${esc(notif.body)}</div>
            </div>
            <button class="toast-close" data-id="${esc(notif.id)}">&times;</button>
        `;

        el.querySelector('.toast-close').addEventListener('click', () => dismiss(notif.id));
        el.addEventListener('click', (e) => {
            if (e.target.classList.contains('toast-close')) return;
            dismiss(notif.id);
            // Mark as read in queue
            if (window.__TAURI__) {
                window.__TAURI__.core.invoke('mark_read', { id: notif.id }).catch(() => {});
            }
        });

        container.appendChild(el);

        const timer = setTimeout(() => dismiss(notif.id), duration || 5000);
        activeToasts.push({ id: notif.id, el, timer });
    }

    function dismiss(id) {
        const idx = activeToasts.findIndex(t => t.id === id);
        if (idx === -1) return;

        const toast = activeToasts[idx];
        clearTimeout(toast.timer);
        toast.el.classList.add('dismissing');

        setTimeout(() => {
            toast.el.remove();
            activeToasts.splice(activeToasts.indexOf(toast), 1);
        }, 300);
    }

    function dismissOldest() {
        if (activeToasts.length === 0) return;
        dismiss(activeToasts[0].id);
    }

    function dismissAll() {
        [...activeToasts].forEach(t => dismiss(t.id));
    }

    function setMaxVisible(n) {
        maxVisible = n;
    }

    function esc(s) {
        if (!s) return '';
        const d = document.createElement('div');
        d.textContent = s;
        return d.innerHTML;
    }

    return { init, show, dismiss, dismissAll, setMaxVisible };
})();
