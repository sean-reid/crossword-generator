# Crossword Generator - Dual Target Edition

A crossword puzzle generator that can run both as a web application (WASM) and as a native CLI tool for generating LaTeX puzzle books.

## Project Structure

This is a Cargo workspace with two main crates:

```
crossword-generator/
├── Cargo.toml              # Workspace root
├── wasm/                   # Core library (can build as WASM or native)
│   ├── Cargo.toml
│   ├── Oxford_English_Dictionary.txt
│   └── src/
│       ├── lib.rs          # Conditional WASM bindings
│       ├── dictionary.rs   # Dictionary parsing
│       ├── encoder.rs      # SAT encoding
│       ├── solver.rs       # SAT solving
│       ├── solution.rs     # Data structures
│       └── debug.rs        # Platform-specific debugging
├── cli/                    # Native CLI for LaTeX book generation
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs         # CLI entry point
│       ├── book.rs         # Book structure management
│       └── latex.rs        # LaTeX generation
├── src/                    # Frontend React application
│   ├── App.tsx
│   ├── components/
│   └── types/
├── package.json            # Frontend build config
└── vite.config.ts          # Vite configuration
```

## Features

### Web Application (WASM)
- Interactive crossword puzzle generation in the browser
- Real-time visualization of generated puzzles
- Adjustable grid sizes and difficulty

### CLI Tool
- Generate LaTeX documents containing multiple crossword puzzles
- Configurable book generation (title, number of puzzles, grid size)
- Optional automatic PDF compilation with pdflatex
- Reproducible generation with optional random seeds
- Progress indicators during generation

## Building

### Prerequisites

**For Web Application:**
- Node.js and npm
- Rust and wasm-pack

**For CLI Tool:**
- Rust (stable toolchain)

**For PDF Generation (CLI only):**
- LaTeX distribution (texlive or mactex)

### Build Commands

#### Web Application

```bash
# Install dependencies
npm install

# Build WASM module
npm run build:wasm

# Build web application
npm run build:web

# Or build everything
npm run build

# Development server
npm run dev
```

#### CLI Tool

```bash
# Build CLI binary
cargo build --release -p crossword-cli

# Or use npm script
npm run build:cli

# Binary will be at: target/release/crossword-cli
```

## Usage

### CLI Usage

Generate a LaTeX crossword book:

```bash
# Basic usage - generate 10 puzzles
./target/release/crossword-cli -c 10 -o my_book.tex

# Custom configuration
./target/release/crossword-cli \
    --count 20 \
    --size 16 \
    --title "My Awesome Crosswords" \
    --output puzzles.tex

# Generate and compile to PDF immediately
./target/release/crossword-cli \
    -c 15 \
    -o book.tex \
    --compile

# Reproducible generation with seed
./target/release/crossword-cli \
    -c 10 \
    --seed 12345 \
    -o reproducible.tex
```

#### CLI Options

- `-c, --count <NUM>` - Number of puzzles to generate (default: 10)
- `-o, --output <PATH>` - Output LaTeX file path (default: crossword_book.tex)
- `-s, --size <SIZE>` - Grid size NxN (default: 16)
- `-t, --title <TITLE>` - Book title (default: "Crossword Puzzle Book")
- `--seed <SEED>` - Random seed for reproducibility
- `--compile` - Automatically compile PDF with pdflatex

### Manual LaTeX Compilation

If you don't use the `--compile` flag, you can compile the LaTeX manually:

```bash
pdflatex crossword_book.tex
pdflatex crossword_book.tex  # Run twice for proper references
```

## Architecture

### Conditional Compilation

The core crossword generation logic in the `wasm/` crate uses conditional compilation to work in both WASM and native contexts:

- **WASM mode** (with `--features wasm`): 
  - Includes wasm-bindgen bindings
  - Uses web_time for timing
  - Logs to browser console
  
- **Native mode** (default):
  - No WASM dependencies
  - Uses std::time for timing
  - Logs to stderr

### Code Sharing

The following modules are shared between web and CLI:
- `dictionary.rs` - Oxford English Dictionary parsing
- `encoder.rs` - SAT constraint encoding
- `solver.rs` - SAT solving with Varisat
- `solution.rs` - Data structures for crossword puzzles

The CLI adds:
- `latex.rs` - LaTeX document generation
- `book.rs` - Book structure and management

## Development

### Testing

```bash
# Test core library
cd wasm
cargo test

# Test CLI
cd cli
cargo test
```

### Debugging

Enable debug logging:

```bash
# WASM (web console)
npm run build:wasm:debug

# CLI (stderr)
cargo build --release -p crossword-cli --features debug
```

## Technical Details

### SAT Solving

The crossword generator uses SAT (Boolean Satisfiability) solving to find valid word placements:

1. **Encoding**: Convert the crossword constraints into Boolean formulas
2. **Solving**: Use the Varisat SAT solver to find satisfying assignments
3. **Extraction**: Convert the SAT solution back into word placements

### LaTeX Output

The CLI generates professional LaTeX documents with:
- TikZ-based grid rendering
- Properly numbered clues
- Two-column clue layout
- Automatic page breaks between puzzles

### Word Selection

The generator intelligently selects words from the Oxford English Dictionary:
- Filters by suitable lengths (3 to grid size)
- Balances word length distribution (70% short, 25% medium, 5% long)
- Uses random sampling for variety
- Adjusts word count based on grid size
