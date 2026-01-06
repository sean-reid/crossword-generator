# Crossword Generator

A crossword puzzle generator with both web (WASM) and CLI interfaces for generating LaTeX puzzle books.

## Quick Start

### Prerequisites
Install [mise](https://mise.jdx.dev/) for version management:
```bash
curl https://mise.run | sh
```

### Setup
```bash
# Install pinned versions of Rust, Node.js, and wasm-pack
mise install

# Install all dependencies
mise run install
```

### Web Application
```bash
mise run dev
```

### CLI Tool
```bash
mise run build-cli
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
│   └── wordset/            # WordSet dictionary data
├── cli/                    # Native CLI for LaTeX generation
│   └── src/
│       ├── main.rs
│       ├── book.rs
│       └── latex.rs
├── src/                    # React frontend
└── package.json
```

## Pinned Versions

This project uses [mise](https://mise.jdx.dev/) to pin tool versions for reproducible builds:
- **Rust**: 1.83.0
- **Node.js**: 22.12.0
- **wasm-pack**: 0.13.0

All versions are specified in `mise.toml`. Run `mise install` to get the correct versions.

## Features

- **Web App**: Interactive browser-based puzzle generation (uses built-in clean word filter)
- **CLI Tool**: Generate professional LaTeX books ready for publishing
- **KDP Compliant**: Proper margins, gutters, facing pages, and front matter for Amazon KDP
- **Cover Generation**: Automatically generate KDP covers with correct spine width from templates
- **Facing Pages**: Puzzle on left, clues on right - see both at once
- **Professional Front Matter**: Title page, copyright page, table of contents
- **Word Filtering**: Custom allowlist support to control vocabulary
- **Parallel Generation**: Uses all CPU cores with rayon for fast batch generation
- **Publishing Ready**: Customizable title page with author, ISBN, publisher info
- **Custom Graphics**: Embed SVG cover art and decorations
- **SAT Solving**: Uses Boolean satisfiability for optimal word placement
- **WordSet Dictionary**: Comprehensive word definitions from the open-source [WordSet project](https://github.com/wordset/wordset-dictionary)

## Building

**Prerequisites**: [mise](https://mise.jdx.dev/) for managing Rust, Node.js, and wasm-pack versions

### Initial Setup
```bash
mise install        # Install Rust, Node.js
mise run install    # Install wasm-pack and npm packages
```

### Web Application

```bash
mise run build-wasm      # Build WASM with --features wasm
mise run build-web       # Build frontend
# Or: mise run build     # Build both
mise run dev             # Development server
```

**Debug build** (includes console logging):
```bash
mise run build-wasm-debug
```

### CLI Tool

```bash
mise run build-cli
# Binary at: target/release/crossword-cli
```

### Development Tasks

```bash
mise run check      # Run cargo check
mise run clippy     # Run linter
mise run fmt        # Format Rust code
mise run test       # Run tests
mise run clean      # Clean all build artifacts
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
    --generate-cover \
    --cover-template cli/paperback-cover.svg \
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
- `--density` - Target density % of filled cells (default: 50)
- `--word-pool` - Word pool size for solver (0 = auto, default: 0)
- `-o, --output` - Output file (default: crossword_book.tex)
- `-j, --jobs` - Parallel threads (default: CPU cores)
- `--seed` - Random seed for reproducibility
- `--compile` - Auto-compile PDF with pdflatex
- `--allowlist` - Path to word allowlist file (default: uses wasm/clean_allowlist.txt)
- `--kdp-format` - paperback or ebook (default: paperback)
- `--trim-size` - Paperback size: 5x8, 5.5x8.5, 6x9, 7x10, 8x10 (default: 8x10)

**Cover Generation:**
- `--generate-cover` - Generate KDP cover from template
- `--cover-template` - Path to cover SVG template (type determined by --kdp-format)
- `--subtitle` - Cover subtitle text (replaces "PUZZLES" in template)
- `--color-interior` - Use color interior spine calculation (default: black & white)

**Publishing Options:**
- `-t, --title` - Book title
- `-a, --author` - Author name
- `-p, --publisher` - Publisher name
- `-e, --edition` - Edition info (e.g., "First Edition", "Volume 1")
- `--isbn` - ISBN number
- `--copyright` - Copyright year
- `-d, --description` - Book description for title page

## How It Works

1. **Dictionary**: Uses WordSet dictionary data with comprehensive definitions
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
