// Breadcrumb / path bar component.

const Breadcrumb = (() => {
    let isEditing = false;

    function render(path, onNavigate) {
        const container = document.getElementById('breadcrumb');

        if (isEditing) return;

        container.innerHTML = '';

        const segments = path.split(/[/\\]/).filter(Boolean);

        // On Windows, the first segment might be a drive like "C:"
        let accumulated = '';

        segments.forEach((seg, i) => {
            if (i > 0) {
                const sep = document.createElement('span');
                sep.className = 'breadcrumb-separator';
                sep.textContent = '\u203A'; // ›
                container.appendChild(sep);
            }

            // Build the accumulated path
            if (i === 0 && seg.match(/^[A-Z]:$/i)) {
                accumulated = seg + '\\';
            } else {
                accumulated += (accumulated.endsWith('/') || accumulated.endsWith('\\'))
                    ? seg
                    : '/' + seg;
            }

            const segEl = document.createElement('span');
            segEl.className = 'breadcrumb-segment';
            segEl.textContent = seg;
            const navPath = accumulated;
            segEl.addEventListener('click', () => onNavigate(navPath));
            container.appendChild(segEl);
        });

        // Click empty area to enter edit mode
        container.addEventListener('click', (e) => {
            if (e.target === container) {
                enterEditMode(container, path, onNavigate);
            }
        });
    }

    function enterEditMode(container, currentPath, onNavigate) {
        isEditing = true;
        container.innerHTML = '';
        const input = document.createElement('input');
        input.className = 'breadcrumb-edit';
        input.type = 'text';
        input.value = currentPath;
        container.appendChild(input);
        input.focus();
        input.select();

        const finish = () => {
            isEditing = false;
            const val = input.value.trim();
            if (val && val !== currentPath) {
                onNavigate(val);
            } else {
                render(currentPath, onNavigate);
            }
        };

        input.addEventListener('keydown', (e) => {
            if (e.key === 'Enter') {
                e.preventDefault();
                finish();
            } else if (e.key === 'Escape') {
                isEditing = false;
                render(currentPath, onNavigate);
            }
        });

        input.addEventListener('blur', finish);
    }

    function focusEdit(currentPath, onNavigate) {
        const container = document.getElementById('breadcrumb');
        enterEditMode(container, currentPath, onNavigate);
    }

    return { render, focusEdit };
})();
