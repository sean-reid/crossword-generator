# Crossword Generator

SAT-based crossword puzzle generator with high-density optimization. Built with Rust/WASM and React.

## Features

- **SAT solving**: Uses constraint satisfaction for valid, connected crosswords
- **75% density**: Generates densely-packed grids
- **Clean clues**: Automated extraction from Oxford English Dictionary
- **Two-phase generation**: Separate encoding and solving with accurate progress tracking
- **Print support**: Two-page portrait layout (grid + clues)
- **Show/hide answers**: Toggle for solving mode

## Quick Start

```bash
npm install
npm run build
```

Output in `docs/` directory, ready for GitHub Pages.

## Development

```bash
npm run dev           # Development server
npm run build:debug   # Build with console logging
```

## Build Commands

- `npm run build:wasm` - Compile Rust to WASM
- `npm run build:wasm:debug` - WASM with debug logging
- `npm run build:web` - Build React app
- `npm run build` - Full build
- `npm run clean` - Clean build artifacts

## Grid Sizes

- 8×8: ~3 seconds
- 10×10: ~8 seconds  
- 12×12: ~15-30 seconds

## How It Works

1. **Dictionary**: 22,000+ words from Oxford English Dictionary with clean clue extraction
2. **Encoding**: Converts crossword rules to SAT constraints (placement, connectivity, density)
3. **Solving**: Varisat SAT solver finds valid grid layouts
4. **Two-phase progress**: Encoding stats update progress bar mid-generation

## Architecture

- **Rust/WASM** (`src/wasm/`): Core engine with dictionary, encoder, solver
- **Web Workers** (`src/workers/`): Non-blocking computation, progress updates
- **React UI** (`src/components/`): Grid, clues, controls

See [ARCHITECTURE.md](ARCHITECTURE.md) for details.

## Troubleshooting

**Generation fails**: Try smaller grid (8×8), check browser console for errors

**Slow solving**: 75% density creates complex SAT problems; 20-40s solve time is normal for 12×12

**Print issues**: Use browser print dialog, select portrait orientation

## License

MIT
