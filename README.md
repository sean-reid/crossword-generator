# Crossword Generator

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

- **Web App**: Interactive browser-based puzzle generation (uses built-in clean word filter)
- **CLI Tool**: Generate professional LaTeX books ready for publishing
- **KDP Compliant**: Proper margins, gutters, facing pages, and front matter for Amazon KDP
- **Facing Pages**: Puzzle on left, clues on right - see both at once
- **Professional Front Matter**: Title page, copyright page, table of contents
- **Word Filtering**: Custom allowlist support to control vocabulary
- **Parallel Generation**: Uses all CPU cores with rayon for fast batch generation
- **Publishing Ready**: Customizable title page with author, ISBN, publisher info
- **Custom Graphics**: Embed SVG cover art and decorations
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
# Basic - generate 10 puzzles in parallel
./target/release/crossword-cli -c 10 -o book.tex

# KDP Paperback (default) - proper facing pages, margins, front matter
./target/release/crossword-cli \
    --count 100 \
    --title "Ultimate Crosswords" \
    --author "Sean Reid" \
    --publisher "Kindle Direct Publishing" \
    --isbn "979-8-218-12345-6" \
    --copyright "2024" \
    --trim-size 6x9 \
    --kdp-format paperback \
    -o kdp-book.tex

# With word filtering for family-friendly content
./target/release/crossword-cli \
    --count 50 \
    --allowlist clean-words.txt \
    --author "Sean Reid" \
    -o clean-book.tex

# KDP Ebook format (simpler margins)
./target/release/crossword-cli \
    --count 50 \
    --kdp-format ebook \
    -o ebook.tex

# Professional book ready for Amazon KDP
./target/release/crossword-cli \
    --count 100 \
    --size 15 \
    --title "The Ultimate Crossword Collection" \
    --author "Jane Smith" \
    --publisher "Smith Publishing" \
    --edition "First Edition" \
    --isbn "978-1-234567-89-0" \
    --copyright "2024" \
    --description "100 challenging crossword puzzles" \
    --cover-svg cover.svg \
    --title-svg decoration.svg \
    --output ultimate-crosswords.tex \
    --jobs 8

# Generate and compile to PDF
./target/release/crossword-cli -c 50 -o book.tex --compile

# Fast generation with specific thread count
./target/release/crossword-cli -c 200 -j 16 -o large-book.tex

# Reproducible with seed
./target/release/crossword-cli --seed 12345 -o book.tex
```

**Core Options:**
- `-c, --count` - Number of puzzles (default: 10)
- `-s, --size` - Grid size (default: 16)
- `-o, --output` - Output file (default: crossword_book.tex)
- `-j, --jobs` - Parallel threads (default: CPU cores)
- `--seed` - Random seed for reproducibility
- `--compile` - Auto-compile PDF with pdflatex
- `--allowlist` - Path to word allowlist file (one word per line, filters dictionary)
- `--kdp-format` - paperback or ebook (default: paperback)
- `--trim-size` - Paperback size: 5x8, 5.5x8.5, 6x9, 7x10, 8x10 (default: 6x9)

**Publishing Options:**
- `-t, --title` - Book title
- `-a, --author` - Author name
- `-p, --publisher` - Publisher name
- `-e, --edition` - Edition info (e.g., "First Edition", "Volume 1")
- `--isbn` - ISBN number
- `--copyright` - Copyright year
- `-d, --description` - Book description for title page

**Graphics Options:**
- `--cover-svg` - Path to cover SVG (e.g., `cover.svg`)
- `--title-svg` - Path to title decoration SVG (e.g., `decoration.svg`)
- **Note**: If no images provided, uses built-in TikZ decoration

## How It Works

1. **Dictionary**: Parses Oxford English Dictionary (100k+ words)
2. **Parallel Generation**: Uses rayon to generate multiple puzzles simultaneously across CPU cores
3. **SAT Encoding**: Converts crossword constraints to Boolean formulas
4. **SAT Solving**: Uses Varisat solver to find valid word placements
5. **Professional Output**: Generates publication-ready LaTeX with custom title page, SVG graphics

**Architecture:**
- Core library (`wasm/`) compiles to both WASM (web) and native (CLI)
- Conditional compilation via `--features wasm` flag
- Shared logic: dictionary, encoder, solver, solution types
- CLI adds: parallel generation (rayon), LaTeX generation, book management, SVG embedding

**Performance:**
- Parallel generation scales linearly with CPU cores
- 100 puzzles in ~5-10 minutes on modern hardware (vs ~30-60 minutes single-threaded)
- Progress bar shows real-time generation status

**Custom Graphics:**
- SVG images optional - built-in TikZ decoration by default
- Provide paths with `--cover-svg` and `--title-svg`
- Sample SVGs in `cli/` directory for reference
