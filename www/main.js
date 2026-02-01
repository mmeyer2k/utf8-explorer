import init, { UnicodeExplorer, get_char_info, search_characters } from './pkg/unicode_explorer.js';

let explorer = null;
let animationId = null;

async function main() {
    // Initialize WASM
    await init();
    
    // Get DOM elements
    const canvas = document.getElementById('canvas');
    const searchInput = document.getElementById('search');
    const planeSelect = document.getElementById('plane-select');
    const zoomIn = document.getElementById('zoom-in');
    const zoomOut = document.getElementById('zoom-out');
    const zoomLevel = document.getElementById('zoom-level');
    const copyBtn = document.getElementById('copy-char');
    
    // Info panel elements
    const charDisplay = document.getElementById('char-display');
    const infoCodepoint = document.getElementById('info-codepoint');
    const infoName = document.getElementById('info-name');
    const infoCategory = document.getElementById('info-category');
    const infoBlock = document.getElementById('info-block');
    
    // Initialize explorer
    explorer = new UnicodeExplorer(canvas);
    
    // Handle resize
    function resize() {
        const rect = canvas.parentElement.getBoundingClientRect();
        const infoPanel = document.getElementById('info-panel');
        const width = rect.width - infoPanel.offsetWidth;
        const height = rect.height;
        explorer.resize(width, height);
        render();
    }
    
    window.addEventListener('resize', resize);
    resize();
    
    // Render loop
    function render() {
        explorer.render();
        zoomLevel.textContent = Math.round(explorer.get_zoom() * 100) + '%';
    }
    
    // Mouse events
    canvas.addEventListener('mousedown', (e) => {
        explorer.start_drag(e.offsetX, e.offsetY);
    });
    
    canvas.addEventListener('mousemove', (e) => {
        explorer.drag(e.offsetX, e.offsetY);
        render();
    });
    
    canvas.addEventListener('mouseup', () => {
        explorer.end_drag();
    });
    
    canvas.addEventListener('mouseleave', () => {
        explorer.end_drag();
    });
    
    canvas.addEventListener('click', (e) => {
        const codepoint = explorer.click(e.offsetX, e.offsetY);
        if (codepoint !== undefined && codepoint !== null) {
            updateInfo(codepoint);
        }
        render();
    });
    
    // Wheel zoom
    canvas.addEventListener('wheel', (e) => {
        e.preventDefault();
        explorer.zoom_at(e.offsetX, e.offsetY, e.deltaY);
        render();
    }, { passive: false });
    
    // Touch events for mobile
    let touchStartDist = 0;
    let touchStartZoom = 1;
    
    canvas.addEventListener('touchstart', (e) => {
        if (e.touches.length === 1) {
            const touch = e.touches[0];
            const rect = canvas.getBoundingClientRect();
            explorer.start_drag(touch.clientX - rect.left, touch.clientY - rect.top);
        } else if (e.touches.length === 2) {
            touchStartDist = Math.hypot(
                e.touches[0].clientX - e.touches[1].clientX,
                e.touches[0].clientY - e.touches[1].clientY
            );
            touchStartZoom = explorer.get_zoom();
        }
    });
    
    canvas.addEventListener('touchmove', (e) => {
        e.preventDefault();
        if (e.touches.length === 1) {
            const touch = e.touches[0];
            const rect = canvas.getBoundingClientRect();
            explorer.drag(touch.clientX - rect.left, touch.clientY - rect.top);
            render();
        } else if (e.touches.length === 2) {
            const dist = Math.hypot(
                e.touches[0].clientX - e.touches[1].clientX,
                e.touches[0].clientY - e.touches[1].clientY
            );
            const scale = dist / touchStartDist;
            explorer.set_zoom(touchStartZoom * scale);
            render();
        }
    }, { passive: false });
    
    canvas.addEventListener('touchend', () => {
        explorer.end_drag();
    });
    
    // Zoom buttons
    zoomIn.addEventListener('click', () => {
        explorer.set_zoom(explorer.get_zoom() * 1.2);
        render();
    });
    
    zoomOut.addEventListener('click', () => {
        explorer.set_zoom(explorer.get_zoom() / 1.2);
        render();
    });
    
    // Plane select
    planeSelect.addEventListener('change', () => {
        explorer.set_plane(parseInt(planeSelect.value));
        render();
    });
    
    // Search
    let searchTimeout = null;
    searchInput.addEventListener('input', () => {
        clearTimeout(searchTimeout);
        searchTimeout = setTimeout(() => {
            const query = searchInput.value.trim();
            if (query.length >= 2) {
                const results = search_characters(query, 20);
                // For now, jump to first result
                if (results.length > 0) {
                    const cp = results[0];
                    const plane = Math.floor(cp / 0x10000);
                    planeSelect.value = plane;
                    explorer.set_plane(plane);
                    explorer.center_on(cp);
                    explorer.click(explorer.width / 2, explorer.height / 2);
                    updateInfo(cp);
                    render();
                }
            }
        }, 300);
    });
    
    // Copy button
    copyBtn.addEventListener('click', async () => {
        const selected = explorer.get_selected();
        if (selected !== undefined && selected !== null) {
            const char = String.fromCodePoint(selected);
            try {
                await navigator.clipboard.writeText(char);
                copyBtn.textContent = 'Copied!';
                setTimeout(() => {
                    copyBtn.textContent = 'Copy Character';
                }, 1500);
            } catch (err) {
                console.error('Failed to copy:', err);
            }
        }
    });
    
    // Update info panel
    function updateInfo(codepoint) {
        try {
            const info = JSON.parse(get_char_info(codepoint));
            charDisplay.textContent = info.char ? String.fromCodePoint(codepoint) : '—';
            infoCodepoint.textContent = info.codepoint;
            infoName.textContent = info.name;
            infoCategory.textContent = info.category;
            infoBlock.textContent = info.block;
            copyBtn.disabled = !info.char;
        } catch (err) {
            console.error('Error getting char info:', err);
        }
    }
    
    // Keyboard shortcuts
    document.addEventListener('keydown', (e) => {
        if (e.target === searchInput) return;
        
        switch (e.key) {
            case '+':
            case '=':
                explorer.set_zoom(explorer.get_zoom() * 1.2);
                render();
                break;
            case '-':
                explorer.set_zoom(explorer.get_zoom() / 1.2);
                render();
                break;
            case '0':
                explorer.set_zoom(1);
                render();
                break;
            case '/':
                e.preventDefault();
                searchInput.focus();
                break;
        }
    });
    
    // Initial render
    render();
    
    console.log('Unicode Explorer initialized!');
}

main().catch(console.error);
