#!/bin/bash
# Build verification script for the refactored crossword generator

set -e

echo "=========================================="
echo "Crossword Generator - Build Verification"
echo "=========================================="
echo ""

# Check prerequisites
echo "Checking prerequisites..."
command -v cargo >/dev/null 2>&1 || { echo "Error: cargo not found. Install Rust." >&2; exit 1; }
command -v node >/dev/null 2>&1 || { echo "Error: node not found. Install Node.js." >&2; exit 1; }
command -v npm >/dev/null 2>&1 || { echo "Error: npm not found. Install npm." >&2; exit 1; }
echo "✓ Prerequisites found"
echo ""

# Test WASM build
echo "Testing WASM build (with wasm feature)..."
cd wasm
if command -v wasm-pack >/dev/null 2>&1; then
    wasm-pack build --target web --features wasm --out-dir ../wasm-pkg-test
    echo "✓ WASM build successful"
    rm -rf ../wasm-pkg-test
else
    echo "⚠ wasm-pack not found - skipping WASM build test"
    echo "  Install with: cargo install wasm-pack"
fi
cd ..
echo ""

# Test CLI build
echo "Testing CLI build..."
cargo build --release -p crossword-cli
echo "✓ CLI build successful"
echo "  Binary location: target/release/crossword-cli"
echo ""

# Test frontend dependencies
echo "Testing frontend build setup..."
if [ -f "package.json" ]; then
    npm install
    echo "✓ Frontend dependencies installed"
else
    echo "⚠ package.json not found - skipping frontend test"
fi
echo ""

echo "=========================================="
echo "All builds completed successfully!"
echo "=========================================="
echo ""
echo "Next steps:"
echo "  1. Run CLI: ./target/release/crossword-cli --help"
echo "  2. Build frontend: npm run build"
echo "  3. Read README.md for full documentation"
