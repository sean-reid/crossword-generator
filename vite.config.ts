import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import wasm from 'vite-plugin-wasm';
import { copyFileSync, readdirSync, existsSync } from 'fs';
import { join } from 'path';

// Plugin to copy WASM files to output
function copyWasmFiles() {
  return {
    name: 'copy-wasm-files',
    closeBundle() {
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
        }
      }
    }
  };
}

export default defineConfig({
  plugins: [
    react(), 
    wasm(),
    copyWasmFiles()
  ],
  base: '/crossword-generator/',
  build: {
    outDir: 'docs',
    target: 'esnext',
    assetsInlineLimit: 0,
  },
  worker: {
    format: 'es',
    plugins: () => [wasm()],
  },
  optimizeDeps: {
    exclude: ['wasm-pkg'],
  },
  server: {
    fs: {
      allow: ['..']
    }
  }
});
