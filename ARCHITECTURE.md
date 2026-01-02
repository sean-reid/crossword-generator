# Crossword Puzzle Generator - Architecture Document

## System Overview

A high-performance crossword puzzle generator that uses constraint satisfaction and SAT solving to create dense, valid crossword grids from dictionary words. The system prioritizes grid density and leverages Rust/WASM for computational performance, with React providing a clean, minimal UI.

## Architecture Layers

### 1. Core Engine Layer (Rust → WASM)

**Location**: `/src/wasm/`

**Responsibilities**:
- Dictionary parsing and word indexing
- SAT-based constraint satisfaction for crossword generation
- Grid density optimization
- Solution validation and scoring

**Key Modules**:

**1.1 Dictionary Module** (`dictionary.rs`)
- Parses the Oxford English Dictionary format (word + definition pairs)
- Builds efficient lookup structures (hash maps, tries for prefix matching)
- Extracts one-sentence clues from definitions
- Compiles dictionary data directly into WASM binary for instant availability

**1.2 Grid Constraints Module** (`constraints.rs`)
- Encodes crossword rules as SAT constraints:
  - Word placement boundaries (no adjacent parallel words)
  - Character intersection requirements
  - Single connected component requirement
  - Empty cell constraints
- Implements density scoring function (characters placed / total grid cells)
- Configurable grid size and minimum quality thresholds

**1.3 SAT Solver Integration** (`solver.rs`)
- Integrates a Rust SAT solver library (e.g., `varisat` or `splr`)
- Converts crossword constraints to CNF (Conjunctive Normal Form)
- Implements iterative solving with increasing density requirements:
  - Start with relaxed constraints to find any valid solution
  - Progressively tighten density requirements
  - Return the densest solution found within time budget
- Timeout mechanism to ensure UI responsiveness (target: <5 seconds)

**1.4 Solution Builder Module** (`solution.rs`)
- Interprets SAT model into concrete word placements
- Constructs final grid representation
- Generates clue list with proper formatting (across/down numbering)
- Calculates solution statistics (word count, density, coverage)

**1.5 WASM Interface** (`lib.rs`)
- Exposes public API via `wasm-bindgen`:
  - `initialize()` - One-time setup, validates dictionary
  - `generate_crossword(size: u32, timeout_ms: u32) → Result<CrosswordPuzzle, String>`
  - `get_dictionary_stats() → DictionaryStats`
  - `validate_solution(grid: Grid, words: Vec<Word>) → bool`

**Data Structures**:
```
CrosswordPuzzle {
  grid: Vec<Vec<Option<char>>>,  // 2D grid with Some(char) or None
  across_clues: Vec<Clue>,        // {number, word, clue_text, x, y}
  down_clues: Vec<Clue>,
  metadata: {
    density: f32,
    word_count: u32,
    total_letters: u32,
    generation_time_ms: u32
  }
}
```

### 2. Web Worker Layer (TypeScript)

**Location**: `/src/workers/`

**Responsibilities**:
- Initialize and manage WASM module lifecycle
- Handle message-based communication with main thread
- Provide non-blocking computation interface
- Manage generation timeouts and cancellation

**Key Components**:

**2.1 Crossword Worker** (`crossword.worker.ts`)
- Loads WASM module on worker initialization
- Message handlers:
  - `INIT` - Load WASM, verify dictionary, respond with stats
  - `GENERATE` - Call WASM generation with parameters, stream progress updates
  - `CANCEL` - Abort current generation
  - `VALIDATE` - Check if user solution is correct
- Progress reporting (via periodic messages during generation)
- Error handling and graceful degradation

**Message Protocol**:
```
// From Main → Worker
{ type: 'INIT' }
{ type: 'GENERATE', payload: { size: number, timeoutMs: number } }
{ type: 'CANCEL' }

// From Worker → Main
{ type: 'READY', payload: { dictionaryStats } }
{ type: 'PROGRESS', payload: { stage: string, percent: number } }
{ type: 'SUCCESS', payload: { puzzle: CrosswordPuzzle } }
{ type: 'ERROR', payload: { message: string } }
```

### 3. UI Layer (React + TypeScript)

**Location**: `/src/components/`, `/src/App.tsx`

**Responsibilities**:
- Display crossword grid and clues
- Manage user interactions
- Coordinate with Web Worker
- Provide generation controls

**Component Hierarchy**:

**3.1 App** (`App.tsx`)
- Root component, manages global state
- Handles worker initialization and communication
- Routes messages between worker and child components

**3.2 ControlPanel** (`ControlPanel.tsx`)
- Grid size selector (dropdown: 8x8, 10x10, 12x12, 15x15, 20x20)
- Generate button with loading state
- Generation timeout control (slider: 1-30 seconds)
- Statistics display (density, word count, generation time)
- Export options (print, save as PDF, share)

**3.3 CrosswordGrid** (`CrosswordGrid.tsx`)
- Renders the crossword grid with proper cell numbering
- Displays black cells vs. letter cells
- Interactive mode: click cells to reveal answers (for solving)
- Print-friendly styling
- Responsive sizing (maintains aspect ratio)

**3.4 CluesList** (`CluesList.tsx`)
- Two-column layout: Across and Down
- Each clue shows: number, clue text
- Click-to-highlight corresponding grid cells
- Scrollable with sticky section headers
- Search/filter functionality for long clue lists

**3.5 LoadingOverlay** (`LoadingOverlay.tsx`)
- Progress indicator during generation
- Cancel button
- Status messages ("Encoding constraints", "Searching for solution", etc.)
- Minimalist spinner animation

**3.6 ErrorBoundary** (`ErrorBoundary.tsx`)
- Catches React errors
- Displays user-friendly error messages
- Retry/reset functionality

**State Management**:
- React hooks (useState, useEffect, useReducer) for local state
- No external state management library needed for this scope
- Worker communication state machine:
  - `UNINITIALIZED` → `INITIALIZING` → `READY` → `GENERATING` → `COMPLETE`

### 4. Build & Deployment Layer

**Location**: `/`, build configuration files

**Build Pipeline**:

**4.1 WASM Build** (`wasm-pack`)
- Compiles Rust to WASM with optimizations (`wasm-opt -O3`)
- Bundles dictionary data directly into WASM
- Generates TypeScript bindings
- Outputs to `/src/wasm-pkg/`

**4.2 Frontend Build** (`Vite`)
- Bundles React app with WASM integration
- Optimizes for production (code splitting, tree shaking)
- Copies WASM files to output directory
- Generates `/docs/` for GitHub Pages hosting
- Configures base path for GH Pages subdirectory

**4.3 NPM Scripts** (`package.json`)
```
build:wasm - Compile Rust to WASM
build:web - Build React app
build - Full build (wasm + web)
dev - Development server with hot reload
preview - Preview production build locally
deploy - Build and commit to gh-pages
```

## Data Flow

### Generation Flow:
1. User clicks "Generate" in ControlPanel
2. App sends `GENERATE` message to worker with parameters
3. Worker invokes WASM `generate_crossword()`
4. WASM engine:
   - Encodes all dictionary words as potential placements
   - Builds SAT constraints for validity and connectivity
   - Iteratively solves with increasing density requirements
   - Returns densest solution found within timeout
5. Worker sends `SUCCESS` message with puzzle data
6. App updates state, triggers re-render of Grid and CluesList

### Density Optimization Strategy:
1. First pass: Find any valid solution (baseline)
2. Second pass: Add constraint requiring density > baseline
3. Repeat until timeout or UNSAT
4. Return best solution found

## Performance Considerations

**WASM Optimization**:
- Compile dictionary into binary format (no runtime parsing)
- Use bit-packing for grid representation
- Minimize heap allocations during solving
- Profile hot paths with `cargo flamegraph`

**Worker Thread**:
- Prevent main thread blocking during generation
- Enable cancellation for long-running operations
- Reuse worker instance (no reload between generations)

**UI Responsiveness**:
- Virtual scrolling for large clue lists (if needed)
- Debounce user inputs
- Lazy load non-critical components
- Optimize grid rendering with CSS Grid

## Technology Stack Summary

- **Core Engine**: Rust 1.75+, wasm-pack, wasm-bindgen
- **SAT Solver**: varisat or splr (pure Rust)
- **Frontend**: React 18, TypeScript 5, Vite 5
- **Styling**: TailwindCSS 3 (minimal configuration)
- **Build**: npm, wasm-pack, gh-pages
- **Hosting**: GitHub Pages

## File Structure

```
/
├── src/
│   ├── wasm/                 # Rust source
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs        # WASM interface
│   │   │   ├── dictionary.rs
│   │   │   ├── constraints.rs
│   │   │   ├── solver.rs
│   │   │   └── solution.rs
│   │   └── Oxford_English_Dictionary.txt
│   ├── workers/
│   │   └── crossword.worker.ts
│   ├── components/
│   │   ├── App.tsx
│   │   ├── ControlPanel.tsx
│   │   ├── CrosswordGrid.tsx
│   │   ├── CluesList.tsx
│   │   ├── LoadingOverlay.tsx
│   │   └── ErrorBoundary.tsx
│   ├── types/
│   │   └── crossword.ts      # TypeScript interfaces
│   ├── main.tsx
│   └── index.css
├── public/
├── docs/                     # Build output (GH Pages)
├── package.json
├── vite.config.ts
├── tailwind.config.js
├── tsconfig.json
└── README.md
```

## Design Principles

**Minimalism**: Clean, typography-focused interface with generous whitespace
**Performance**: Sub-5-second generation for reasonable grid sizes
**Density**: Prioritize packed grids over sparse ones
**Reliability**: Graceful error handling, always return best-effort result
**Portability**: Static site, no backend required, works offline after initial load
