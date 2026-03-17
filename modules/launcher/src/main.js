/**
 * Sovereign Launcher — Frontend
 *
 * Handles search input, result rendering, keyboard navigation,
 * and communication with the Rust backend via Tauri invoke.
 */

const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const { getCurrentWindow } = window.__TAURI__.window;

// ── State ───────────────────────────────────────────────────────────────

let selectedIndex = 0;
let results = [];
let calcResult = null; // { display, value } when in calculator mode
let debounceTimer = null;
const DEBOUNCE_MS = 80;
const launcher = document.getElementById('launcher');

// ── DOM Elements ────────────────────────────────────────────────────────

const searchInput = document.getElementById('search-input');
const resultsContainer = document.getElementById('results');
const indexCount = document.getElementById('index-count');

// ── Initialization ──────────────────────────────────────────────────────

async function init() {
    // Show index count
    try {
        const count = await invoke('get_index_count');
        indexCount.textContent = `${count} apps`;
    } catch (e) {
        console.error('Failed to get index count:', e);
    }

    // Load initial results (most-launched apps)
    await doSearch('');

    // Listen for window-shown events (from hotkey toggle)
    await listen('window-shown', () => {
        // Replay entrance animation
        launcher.style.animation = 'none';
        launcher.offsetHeight; // force reflow
        launcher.style.animation = '';
        searchInput.focus();
        searchInput.select();
    });

    // Focus input on load
    searchInput.focus();
}

// ── Search ──────────────────────────────────────────────────────────────

searchInput.addEventListener('input', () => {
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
        doSearch(searchInput.value);
    }, DEBOUNCE_MS);
});

async function doSearch(query) {
    // Calculator mode: input starts with '='
    if (query.trimStart().startsWith('=')) {
        const expr = query.trimStart().substring(1).trim();
        results = [];
        if (expr.length === 0) {
            calcResult = null;
        } else {
            try {
                calcResult = await invoke('evaluate_calc', { expr });
            } catch (e) {
                calcResult = { display: e, value: null, error: true };
            }
        }
        renderResults();
        resizeWindow();
        return;
    }

    calcResult = null;
    try {
        results = await invoke('search', { query });
        selectedIndex = 0;
        renderResults();
        resizeWindow();
    } catch (e) {
        console.error('Search error:', e);
        results = [];
        renderResults();
    }
}

// ── Rendering ───────────────────────────────────────────────────────────

function renderResults() {
    // Calculator mode display
    if (calcResult !== null) {
        resultsContainer.classList.remove('hidden');
        if (calcResult.error) {
            resultsContainer.innerHTML = `<div class="no-results">${escapeHtml(calcResult.display)}</div>`;
        } else {
            resultsContainer.innerHTML = `
                <div class="calc-result" id="calc-result-item">
                    <div class="calc-icon">=</div>
                    <div class="calc-value">${escapeHtml(calcResult.display)}</div>
                    <span class="calc-hint">Enter to copy</span>
                </div>
                <div id="results-footer">
                    <span class="hint"><kbd>Enter</kbd> copy to clipboard</span>
                    <span class="hint"><kbd>Esc</kbd> close</span>
                </div>
            `;
            document.getElementById('calc-result-item')?.addEventListener('click', copyCalcResult);
        }
        return;
    }

    if (results.length === 0 && searchInput.value.trim() !== '') {
        resultsContainer.innerHTML = '<div class="no-results">No results found</div>';
        resultsContainer.classList.remove('hidden');
        return;
    }

    if (results.length === 0) {
        resultsContainer.classList.add('hidden');
        return;
    }

    resultsContainer.classList.remove('hidden');

    let html = '';
    results.forEach((result, i) => {
        const selected = i === selectedIndex ? ' selected' : '';
        const initial = result.name.charAt(0).toUpperCase();
        const shortPath = truncatePath(result.path, 60);
        const badge = result.launch_count > 0
            ? `<span class="result-badge">${result.launch_count}×</span>`
            : '';

        // Use icon_path for the icon element, fallback to initial letter
        const iconId = `icon-${i}`;
        const iconHtml = `<div class="result-icon" id="${iconId}">${initial}</div>`;

        html += `
            <div class="result-item${selected}"
                 data-index="${i}"
                 data-id="${result.id}"
                 data-path="${escapeHtml(result.path)}"
                 data-icon-path="${escapeHtml(result.icon_path || '')}">
                ${iconHtml}
                <div class="result-info">
                    <div class="result-name">${escapeHtml(result.name)}</div>
                    <div class="result-path">${escapeHtml(shortPath)}</div>
                </div>
                ${badge}
            </div>
        `;
    });

    // Keyboard hint footer
    html += `
        <div id="results-footer">
            <span class="hint"><kbd>↑↓</kbd> navigate</span>
            <span class="hint"><kbd>Enter</kbd> launch</span>
            <span class="hint"><kbd>Ctrl+Enter</kbd> open folder</span>
            <span class="hint"><kbd>Esc</kbd> close</span>
        </div>
    `;

    resultsContainer.innerHTML = html;

    // Click handlers
    resultsContainer.querySelectorAll('.result-item').forEach(el => {
        el.addEventListener('click', () => {
            const idx = parseInt(el.dataset.index);
            selectedIndex = idx;
            launchSelected();
        });
    });

    // Load icons asynchronously
    loadIcons();
}

async function loadIcons() {
    for (let i = 0; i < results.length; i++) {
        const result = results[i];
        const iconTarget = result.icon_path || result.path;
        if (!iconTarget) continue;

        try {
            const pngPath = await invoke('get_icon', { exePath: iconTarget });
            if (pngPath) {
                const el = document.getElementById(`icon-${i}`);
                if (el) {
                    el.innerHTML = `<img src="https://asset.localhost/${pngPath}" width="20" height="20" style="image-rendering: pixelated;" />`;
                    el.style.background = 'transparent';
                }
            }
        } catch (_) {
            // Silently fall back to initial letter
        }
    }
}

// ── Window Resizing ─────────────────────────────────────────────────────

async function resizeWindow() {
    const appWindow = getCurrentWindow();

    // Base height: search bar (~52px) + border (2px)
    const baseHeight = 54;

    let resultHeight;
    if (calcResult !== null) {
        // Calculator mode: result row (~60px) + footer (~28px)
        resultHeight = 88;
    } else if (results.length > 0) {
        // Each result item is ~47px, footer is ~28px
        resultHeight = (Math.min(results.length, 8) * 47) + 28;
    } else {
        resultHeight = searchInput.value.trim() !== '' ? 50 : 0;
    }

    const totalHeight = baseHeight + resultHeight;

    try {
        await appWindow.setSize({
            type: 'Logical',
            width: 600,
            height: Math.max(54, Math.min(totalHeight, 470))
        });
    } catch (e) {
        console.error('Resize error:', e);
    }
}

// ── Keyboard Navigation ─────────────────────────────────────────────────

searchInput.addEventListener('keydown', async (e) => {
    switch (e.key) {
        case 'ArrowDown':
            e.preventDefault();
            if (results.length > 0) {
                selectedIndex = Math.min(selectedIndex + 1, results.length - 1);
                renderResults();
                scrollToSelected();
            }
            break;

        case 'ArrowUp':
            e.preventDefault();
            if (results.length > 0) {
                selectedIndex = Math.max(selectedIndex - 1, 0);
                renderResults();
                scrollToSelected();
            }
            break;

        case 'Enter':
            e.preventDefault();
            if (calcResult && !calcResult.error) {
                copyCalcResult();
            } else if (e.ctrlKey) {
                openSelectedFolder();
            } else {
                launchSelected();
            }
            break;

        case 'Escape':
            e.preventDefault();
            searchInput.value = '';
            results = [];
            resultsContainer.classList.add('hidden');
            animateHide();
            break;

        case 'Tab':
            e.preventDefault();
            // Tab cycles through results like arrow down
            if (results.length > 0) {
                selectedIndex = (selectedIndex + 1) % results.length;
                renderResults();
                scrollToSelected();
            }
            break;
    }
});

function scrollToSelected() {
    const selected = resultsContainer.querySelector('.result-item.selected');
    if (selected) {
        selected.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
    }
}

// ── Launch Actions ──────────────────────────────────────────────────────

async function launchSelected() {
    if (results.length === 0 || selectedIndex >= results.length) return;

    const result = results[selectedIndex];

    try {
        // Record the launch for ranking
        await invoke('record_launch', { id: result.id });

        // Launch the application
        await invoke('launch_app', { path: result.path });

        // Reset and hide
        searchInput.value = '';
        results = [];
        resultsContainer.classList.add('hidden');
        animateHide();
    } catch (e) {
        console.error('Launch error:', e);
    }
}

async function openSelectedFolder() {
    if (results.length === 0 || selectedIndex >= results.length) return;

    const result = results[selectedIndex];

    try {
        await invoke('open_containing_folder', { path: result.path });
        searchInput.value = '';
        results = [];
        resultsContainer.classList.add('hidden');
        animateHide();
    } catch (e) {
        console.error('Open folder error:', e);
    }
}

// ── Calculator ──────────────────────────────────────────────────────

async function copyCalcResult() {
    if (!calcResult || calcResult.error) return;
    try {
        await navigator.clipboard.writeText(calcResult.display);
        // Brief visual feedback
        const el = document.getElementById('calc-result-item');
        if (el) {
            const hint = el.querySelector('.calc-hint');
            if (hint) hint.textContent = 'Copied!';
            setTimeout(() => {
                searchInput.value = '';
                calcResult = null;
                resultsContainer.classList.add('hidden');
                animateHide();
            }, 400);
        }
    } catch (e) {
        console.error('Copy error:', e);
    }
}

// ── Show/Hide Animation ─────────────────────────────────────────────

function animateHide() {
    launcher.classList.add('hiding');
    launcher.addEventListener('animationend', async () => {
        launcher.classList.remove('hiding');
        try {
            await invoke('hide_window');
        } catch (err) {
            console.error('Hide error:', err);
        }
    }, { once: true });
}

// ── Utilities ───────────────────────────────────────────────────────────

function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

function truncatePath(path, maxLen) {
    if (path.length <= maxLen) return path;
    // Show start and end of path
    const start = path.substring(0, 20);
    const end = path.substring(path.length - (maxLen - 23));
    return `${start}...${end}`;
}

// ── Start ───────────────────────────────────────────────────────────────

init();
