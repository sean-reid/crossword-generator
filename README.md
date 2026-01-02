# Crossword Generator

High-performance crossword puzzle generator using SAT solving and density optimization. Built with Rust/WASM for the core engine and React for the UI.

## Features

- **SAT-based generation**: Uses constraint satisfaction to ensure valid crosswords
- **Density optimization**: Iteratively finds the densest possible grid layout
- **Real dictionary**: Includes Oxford English Dictionary with 70,000+ entries
- **Fast generation**: Typically completes in 5-10 seconds
- **Minimal UI**: Clean, print-friendly interface
- **No backend**: Fully static site, works offline after initial load

## Building

### Prerequisites

- Node.js 18+
- Rust 1.75+
- wasm-pack

### Build Steps

```bash
# Install dependencies
npm install

# Build WASM module
npm run build:wasm

# Build WASM with debug logging
npm run build:wasm:debug

# Build web application
npm run build:web

# Or build everything at once
npm run build

# Build with debug logging enabled
npm run build:debug
```

The built site will be in the `docs/` directory, ready for GitHub Pages deployment.

### Debug Mode

To enable verbose console logging from the WASM module:

```bash
npm run build:debug
```

This will log:
- Dictionary loading statistics
- Word selection details
- Constraint encoding progress
- SAT solver status
- Solution extraction details

Check your browser console for `[WASM]`, `[SOLVER]`, and `[CONSTRAINTS]` prefixed messages.

### Verifying the Build

After building, check that the `docs/` directory contains:
- `index.html`
- `assets/` folder with JS/CSS bundles
- `crossword_wasm_bg.wasm` - The WASM binary
- `crossword_wasm.js` - WASM JavaScript bindings
- `.nojekyll` file

If WASM files are missing, the app won't work.

## Development

```bash
# Start development server
npm run dev
```

Note: WASM must be built at least once before running the dev server.

## Troubleshooting

### "undefined is not an object" error
- Ensure WASM files are in the `docs/` directory after build
- Check browser console for WASM loading errors
- Verify `wasm-pkg/` directory was created by `npm run build:wasm`

### Generation fails or times out
- Try a smaller grid size (8x8 or 10x10)
- Increase the timeout value
- Check browser console for Rust panic messages

### Build fails
- Ensure Rust and wasm-pack are installed
- Run `wasm-pack --version` to verify installation
- Try `cargo clean` in `src/wasm/` directory

## Architecture

- **Rust/WASM** (`src/wasm/`): Core crossword generation engine
  - Dictionary parsing and indexing
  - SAT constraint encoding
  - Varisat solver integration
  - Solution building and validation

- **Web Worker** (`src/workers/`): Non-blocking WASM interface
  - WASM module lifecycle management
  - Progress reporting
  - Message-based communication

- **React UI** (`src/components/`): Minimal, clean interface
  - Grid visualization with proper numbering
  - Clue display (across/down)
  - Generation controls
  - Print support

## License

MIT
