// Search bar component.

const SearchBar = (() => {
    const { invoke } = window.__TAURI__.core;
    let debounceTimer = null;
    let onResults = null;

    function init(searchResultsCallback) {
        onResults = searchResultsCallback;

        const container = document.getElementById('search-bar');
        container.innerHTML = '<input id="search-input" type="text" placeholder="Search files..." />';

        const input = document.getElementById('search-input');
        input.addEventListener('input', () => {
            clearTimeout(debounceTimer);
            const query = input.value.trim();

            if (!query) {
                if (onResults) onResults(null);
                return;
            }

            debounceTimer = setTimeout(() => search(query), 150);
        });

        input.addEventListener('keydown', (e) => {
            if (e.key === 'Escape') {
                input.value = '';
                input.blur();
                if (onResults) onResults(null);
            }
        });
    }

    async function search(query) {
        try {
            const available = await invoke('search_daemon_available');
            if (!available) {
                // Fall back to no results if daemon is not running
                if (onResults) onResults([]);
                return;
            }

            const results = await invoke('search_files', {
                query,
                maxResults: 50,
                fileTypes: null,
            });

            if (onResults) onResults(results);
        } catch (e) {
            console.error('Search error:', e);
            if (onResults) onResults([]);
        }
    }

    function focus() {
        const input = document.getElementById('search-input');
        if (input) {
            input.focus();
            input.select();
        }
    }

    function clear() {
        const input = document.getElementById('search-input');
        if (input) input.value = '';
        if (onResults) onResults(null);
    }

    return { init, focus, clear };
})();
