// Tab bar component.

const TabBar = (() => {
    function render(tabs, activeTabId, callbacks) {
        const container = document.getElementById('tab-bar');
        container.innerHTML = '';

        tabs.forEach(tab => {
            const el = document.createElement('div');
            el.className = 'tab' + (tab.id === activeTabId ? ' active' : '');
            el.dataset.tabId = tab.id;

            const name = tab.path.split(/[/\\]/).filter(Boolean).pop() || tab.path;
            el.innerHTML = `
                <span class="tab-label">${escapeHtml(name)}</span>
                <span class="tab-close" data-tab-id="${tab.id}">&times;</span>
            `;

            el.addEventListener('click', (e) => {
                if (e.target.classList.contains('tab-close')) {
                    callbacks.onClose(tab.id);
                } else {
                    callbacks.onSelect(tab.id);
                }
            });

            // Middle-click to close
            el.addEventListener('auxclick', (e) => {
                if (e.button === 1) {
                    e.preventDefault();
                    callbacks.onClose(tab.id);
                }
            });

            container.appendChild(el);
        });

        // New tab button
        const newBtn = document.createElement('button');
        newBtn.className = 'tab-new';
        newBtn.textContent = '+';
        newBtn.title = 'New Tab (Ctrl+T)';
        newBtn.addEventListener('click', callbacks.onNew);
        container.appendChild(newBtn);
    }

    function escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }

    return { render };
})();
