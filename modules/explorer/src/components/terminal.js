// Terminal pane placeholder.
// Full PTY integration (ConPTY) will be added in a future session.
// For now, provides a simple "Open in Terminal" action.

const Terminal = (() => {
    let visible = false;

    function toggle() {
        visible = !visible;
        const pane = document.getElementById('terminal-pane');
        pane.classList.toggle('hidden', !visible);

        if (visible && pane.children.length === 0) {
            pane.innerHTML = '<div class="terminal-placeholder">Terminal pane — use "Open in Terminal" from context menu or press Ctrl+` to toggle. Full embedded terminal coming soon.</div>';
        }

        return visible;
    }

    function isVisible() { return visible; }

    return { toggle, isVisible };
})();
