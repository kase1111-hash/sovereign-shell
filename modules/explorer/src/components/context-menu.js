// Context menu component.

const ContextMenu = (() => {
    let menuEl = null;

    function init() {
        menuEl = document.getElementById('context-menu');

        // Hide on any click
        document.addEventListener('click', () => hide());
        document.addEventListener('keydown', (e) => {
            if (e.key === 'Escape') hide();
        });
    }

    function show(x, y, items) {
        menuEl.innerHTML = '';

        items.forEach(item => {
            if (item.separator) {
                const sep = document.createElement('div');
                sep.className = 'ctx-separator';
                menuEl.appendChild(sep);
                return;
            }

            const el = document.createElement('div');
            el.className = 'ctx-item' + (item.danger ? ' danger' : '');
            el.innerHTML = `
                <span>${item.label}</span>
                ${item.shortcut ? `<span class="ctx-shortcut">${item.shortcut}</span>` : ''}
            `;
            el.addEventListener('click', (e) => {
                e.stopPropagation();
                hide();
                item.action();
            });
            menuEl.appendChild(el);
        });

        // Position within viewport
        menuEl.style.left = x + 'px';
        menuEl.style.top = y + 'px';
        menuEl.classList.remove('hidden');

        // Adjust if overflowing
        requestAnimationFrame(() => {
            const rect = menuEl.getBoundingClientRect();
            if (rect.right > window.innerWidth) {
                menuEl.style.left = (window.innerWidth - rect.width - 4) + 'px';
            }
            if (rect.bottom > window.innerHeight) {
                menuEl.style.top = (window.innerHeight - rect.height - 4) + 'px';
            }
        });
    }

    function hide() {
        if (menuEl) menuEl.classList.add('hidden');
    }

    return { init, show, hide };
})();
