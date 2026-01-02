# # Crossword Generator

A crossword puzzle generator with both web (WASM) and CLI interfaces for generating LaTeX puzzle books.

> **Note:** Package renamed from `crossword-wasm` to `crossword-core`. Build with `--features wasm` for web use.

## Quick Start

### Web Application
```bash
npm install
npm run build:wasm    # Builds with --features wasm
npm run dev
```

### CLI Tool
```bash
cargo build --release -p crossword-cli
./target/release/crossword-cli -c 10 -o book.tex
```

## Project Structure

```
crossword-generator/
├── Cargo.toml              # Workspace root
├── wasm/                   # Core library (WASM or native)
│   ├── src/
│   │   ├── lib.rs          # Conditional WASM bindings
│   │   ├── dictionary.rs
│   │   ├── encoder.rs
│   │   ├── solver.rs
│   │   └── solution.rs
│   └── Oxford_English_Dictionary.txt
├── cli/                    # Native CLI for LaTeX generation
│   └── src/
│       ├── main.rs
│       ├── book.rs
│       └── latex.rs
├── src/                    # React frontend
└── package.json
```

## Features

- **Web App**: Interactive browser-based puzzle generation
- **CLI Tool**: Generate LaTeX books with multiple puzzles
- **SAT Solving**: Uses Boolean satisfiability for optimal word placement
- **Oxford Dictionary**: 100k+ words with definitions

## Building

**Prerequisites**: Rust, Node.js, wasm-pack, optionally pdflatex for PDF generation

### Web Application

```bash
npm install
npm run build:wasm    # Important: builds with --features wasm
npm run build:web
# Or: npm run build
npm run dev          # Development server
```

### CLI Tool

```bash
cargo build --release -p crossword-cli
# Binary at: target/release/crossword-cli
```

## CLI Usage

```bash
# Basic - generate 10 puzzles
./target/release/crossword-cli -c 10 -o book.tex

# Custom configuration
./target/release/crossword-cli \
    --count 20 \
    --size 16 \
    --title "My Puzzles" \
    --output book.tex

# Generate and compile to PDF
./target/release/crossword-cli -c 15 -o book.tex --compile

# Reproducible with seed
./target/release/crossword-cli --seed 12345 -o book.tex
```

**Options:**
- `-c, --count` - Number of puzzles (default: 10)
- `-s, --size` - Grid size (default: 16)
- `-o, --output` - Output file (default: crossword_book.tex)
- `-t, --title` - Book title
- `--seed` - Random seed for reproducibility
- `--compile` - Auto-compile PDF with pdflatex

## How It Works

1. **Dictionary**: Parses Oxford English Dictionary (100k+ words)
2. **SAT Encoding**: Converts crossword constraints to Boolean formulas
3. **SAT Solving**: Uses Varisat solver to find valid word placements
4. **Output**: Generates interactive web UI or LaTeX documents

**Architecture:**
- Core library (`wasm/`) compiles to both WASM (web) and native (CLI)
- Conditional compilation via `--features wasm` flag
- Shared logic: dictionary, encoder, solver, solution types
- CLI adds: LaTeX generation, book managementdjusts word count based on grid size
