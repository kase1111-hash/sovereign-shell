// Keyboard shortcut handler for the Explorer.

const Keybindings = (() => {
    const bindings = [];

    function register(combo, handler, description) {
        bindings.push({ combo: combo.toLowerCase(), handler, description });
    }

    function init() {
        document.addEventListener('keydown', (e) => {
            // Don't handle shortcuts when typing in inputs
            if (e.target.tagName === 'INPUT' || e.target.tagName === 'TEXTAREA') {
                // Allow Escape to blur inputs
                if (e.key === 'Escape') {
                    e.target.blur();
                    e.preventDefault();
                }
                return;
            }

            const parts = [];
            if (e.ctrlKey) parts.push('ctrl');
            if (e.shiftKey) parts.push('shift');
            if (e.altKey) parts.push('alt');
            parts.push(e.key.toLowerCase());
            const combo = parts.join('+');

            for (const binding of bindings) {
                if (binding.combo === combo) {
                    e.preventDefault();
                    e.stopPropagation();
                    binding.handler(e);
                    return;
                }
            }
        });
    }

    return { register, init };
})();
