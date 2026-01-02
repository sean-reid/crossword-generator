import { copyFileSync, readdirSync, existsSync } from 'fs';
import { join } from 'path';

const wasmPkgDir = 'wasm-pkg';
const outDir = 'docs';

if (existsSync(wasmPkgDir) && existsSync(outDir)) {
  const files = readdirSync(wasmPkgDir);
  let copied = 0;
  
  for (const file of files) {
    if (file.endsWith('.wasm')) {
      try {
        copyFileSync(join(wasmPkgDir, file), join(outDir, file));
        console.log(`Copied WASM: ${file}`);
        copied++;
      } catch (err) {
        console.error(`Failed to copy ${file}:`, err);
      }
    }
  }
  
  if (copied > 0) {
    console.log(`\n✓ Copied ${copied} WASM file(s) to ${outDir}/`);
  } else {
    console.log(`\n⚠ No WASM files found in ${wasmPkgDir}/`);
  }
} else {
  if (!existsSync(wasmPkgDir)) {
    console.error(`Error: ${wasmPkgDir}/ not found. Run 'npm run build:wasm' first.`);
  }
  if (!existsSync(outDir)) {
    console.error(`Error: ${outDir}/ not found. Build may have failed.`);
  }
}
