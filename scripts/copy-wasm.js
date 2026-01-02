import { copyFileSync, existsSync, mkdirSync, readdirSync } from 'fs';
import { join } from 'path';

const wasmPkgDir = 'wasm-pkg';
const docsDir = 'docs';

if (!existsSync(wasmPkgDir)) {
  console.error('Error: wasm-pkg directory not found. Run npm run build:wasm first.');
  process.exit(1);
}

// Create docs directory if it doesn't exist
if (!existsSync(docsDir)) {
  mkdirSync(docsDir, { recursive: true });
}

// Copy all WASM-related files
const files = readdirSync(wasmPkgDir);
let copiedCount = 0;

for (const file of files) {
  // Copy .wasm, .js, and .d.ts files
  if (file.endsWith('.wasm') || file.endsWith('.js') || file.endsWith('.d.ts')) {
    const src = join(wasmPkgDir, file);
    const dest = join(docsDir, file);
    
    try {
      copyFileSync(src, dest);
      console.log(`Copied: ${file}`);
      copiedCount++;
    } catch (err) {
      console.error(`Failed to copy ${file}:`, err.message);
    }
  }
}

console.log(`\nSuccessfully copied ${copiedCount} WASM files to ${docsDir}/`);
