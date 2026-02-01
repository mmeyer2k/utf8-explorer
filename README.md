# 🔍 Unicode Explorer

A zoomable, interactive explorer for the entire Unicode character set. Built with Rust + WebAssembly for performance.

## Features

- **Zoomable grid** — Navigate 150k+ Unicode characters like a map
- **All 17 planes** — From BMP to supplementary planes
- **Color-coded categories** — Letters, numbers, symbols, punctuation, etc.
- **Search** — Find characters by name or codepoint (U+XXXX)
- **Character details** — Name, category, block, and more
- **Copy to clipboard** — One-click character copying
- **Mobile-friendly** — Touch support with pinch-to-zoom

## Live Demo

[https://yourusername.github.io/unicode-explorer](https://yourusername.github.io/unicode-explorer)

## Development

### Prerequisites

- [Rust](https://rustup.rs/)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)

### Build

```bash
# Build WASM
wasm-pack build --target web --out-dir www/pkg

# Serve locally
cd www && python3 -m http.server 8000
```

Open http://localhost:8000

### Project Structure

```
unicode-explorer/
├── Cargo.toml          # Rust dependencies
├── src/
│   └── lib.rs          # Rust/WASM core
├── www/
│   ├── index.html      # Main page
│   ├── style.css       # Styles
│   ├── main.js         # JS glue code
│   └── pkg/            # WASM output (generated)
└── .github/
    └── workflows/
        └── deploy.yml  # GitHub Pages deployment
```

## Controls

- **Drag** — Pan around the grid
- **Scroll** — Zoom in/out
- **Click** — Select a character
- **+/-** keys — Zoom in/out
- **0** key — Reset zoom
- **/** key — Focus search

## License

MIT
