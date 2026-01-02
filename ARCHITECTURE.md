# Architecture

## Overview

Crossword generator using SAT solving for high-density puzzle creation. Two-phase architecture separates encoding from solving for accurate progress tracking.

## Components

### Rust/WASM Core (`src/wasm/`)

**dictionary.rs**
- Parses Oxford English Dictionary (22k+ words)
- Extracts clean one-sentence clues
- Filters self-references, abbreviations, malformed definitions

**encoder.rs**
- Converts crossword rules to SAT constraints:
  - Placement boundaries (no adjacent words)
  - Character intersections
  - Connected component (all words reachable)
  - 75% minimum grid density
  - Sequence validation (no non-words)
- Returns variable/clause counts for progress estimation

**solver.rs**
- Wraps Varisat SAT solver
- Two functions: `solve_with_iterations()` (legacy) and `solve_encoded()` (two-phase)
- Estimates solve time: `0.085ms per variable`

**solution.rs**
- Extracts word placements from SAT model
- Builds grid and numbered clues
- Validates grid matches placements

**lib.rs**
- WASM interface with three functions:
  - `initialize()` - Load dictionary
  - `estimate_problem_size(size)` - Fast complexity estimate
  - `encode_problem(size)` - Encode constraints, return stats
  - `solve_problem()` - Solve encoded problem
  - `generate_crossword(size)` - Legacy single-call version

### Web Workers (`src/workers/`)

**crossword.worker.ts**
- Manages WASM lifecycle
- Three-phase generation:
  1. Call `estimate_problem_size()` → send ESTIMATE
  2. Call `encode_problem()` → send ENCODING_COMPLETE with real stats
  3. Call `solve_problem()` → send SUCCESS with puzzle

**progress.worker.ts**
- Runs on separate thread
- Animates progress bar every 100ms
- Updates estimate when ENCODING_COMPLETE arrives
- Never blocks

### React UI (`src/components/`)

**App.tsx**
- Manages two workers (crossword + progress)
- State machine: UNINITIALIZED → INITIALIZING → READY → GENERATING → COMPLETE
- Handles ESTIMATE and ENCODING_COMPLETE messages

**ControlPanel.tsx**
- Grid size selector (8×8, 10×10, 12×12)
- Show answers toggle
- Print button
- Statistics display

**CrosswordGrid.tsx**
- Renders grid with cell numbering
- Conditionally shows letters based on `showAnswers` prop
- Calculates clue numbers from word starts

**CluesList.tsx**
- Two-column: Across / Down
- Displays numbered clues

## Data Flow

1. User clicks Generate
2. Progress worker starts with initial estimate
3. WASM worker encodes (5-10s), sends ENCODING_COMPLETE
4. Progress worker updates to real estimate
5. WASM solver runs (10-30s), progress animates
6. SUCCESS → display puzzle

## Key Constraints

- **Placement**: At most one word per position
- **Grid**: Each cell has at most one character
- **Bidirectional**: Grid characters only from placements (prevents non-words)
- **Connectivity**: All filled cells form single connected component
- **Density**: Minimum 75% of cells filled
- **Quality**: Minimum word count based on average lengths

## Performance

- 8×8: ~50k vars, ~2s solve
- 10×10: ~190k vars, ~8s solve
- 12×12: ~330k vars, ~20s solve

Main bottleneck: Connected component constraint adds ~10k reachability variables per grid.

## Word Selection

Heavily biased toward short words (70% are 3-5 letters) for:
- Faster solving (fewer placement variables)
- Better density (more words fit)
- Easier puzzles

Pool sizes: 80 words (8×8), 120 words (10×10), 150 words (12×12)
