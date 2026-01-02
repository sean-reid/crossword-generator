import { WorkerMessage } from '../types/crossword';

let wasmModule: any = null;

self.onmessage = async (event: MessageEvent<WorkerMessage>) => {
  const { type, payload } = event.data;

  try {
    switch (type) {
      case 'INIT':
        await initializeWasm();
        break;

      case 'GENERATE':
        await generateCrossword(payload.size);
        break;

      case 'CANCEL':
        self.postMessage({ type: 'ERROR', payload: { message: 'Generation cancelled' } });
        break;

      default:
        self.postMessage({ 
          type: 'ERROR', 
          payload: { message: `Unknown message type: ${type}` } 
        });
    }
  } catch (error) {
    self.postMessage({ 
      type: 'ERROR', 
      payload: { message: error instanceof Error ? error.message : String(error) } 
    });
  }
};

async function initializeWasm() {
  try {
    self.postMessage({ type: 'PROGRESS', payload: { stage: 'Loading WASM module', percent: 0 } });
    
    // Dynamic import of WASM module
    let wasm;
    try {
      // Try importing the generated WASM bindings
      wasm = await import('../../wasm-pkg/crossword_wasm.js');
    } catch (e) {
      // Fallback to try without .js extension
      wasm = await import('../../wasm-pkg/crossword_wasm');
    }
    
    self.postMessage({ type: 'PROGRESS', payload: { stage: 'Initializing WASM runtime', percent: 25 } });
    
    // Initialize the WASM module (this loads the .wasm file)
    // The default export is the init function
    if (typeof wasm.default === 'function') {
      await wasm.default();
    }
    
    wasmModule = wasm;
    
    self.postMessage({ type: 'PROGRESS', payload: { stage: 'Loading dictionary', percent: 50 } });
    
    // Call the initialize function to set up the dictionary
    const stats = wasm.initialize();
    
    self.postMessage({ type: 'READY', payload: { dictionaryStats: stats } });
  } catch (error) {
    console.error('WASM initialization error:', error);
    throw new Error(`Failed to initialize WASM: ${error instanceof Error ? error.message : String(error)}`);
  }
}

async function generateCrossword(size: number) {
  if (!wasmModule) {
    throw new Error('WASM module not initialized');
  }

  // Phase 1: Initial estimate
  self.postMessage({ type: 'PROGRESS', payload: { stage: 'Analyzing...', percent: 5 } });
  await new Promise(resolve => setTimeout(resolve, 50));

  const estimate = wasmModule.estimate_problem_size(size);
  self.postMessage({ 
    type: 'ESTIMATE', 
    payload: { 
      encoding_ms: estimate.encoding_ms,
      solving_ms: estimate.solving_ms
    } 
  });
  
  await new Promise(resolve => setTimeout(resolve, 50));

  // Phase 2: Encode (blocks but returns stats)
  self.postMessage({ type: 'PROGRESS', payload: { stage: 'Encoding...', percent: 10 } });
  
  try {
    const encodingResult = wasmModule.encode_problem(size);
    
    // Send updated estimate based on REAL encoding stats
    self.postMessage({
      type: 'ENCODING_COMPLETE',
      payload: {
        encoding_time_ms: encodingResult.encoding_time_ms,
        estimated_solve_ms: encodingResult.estimated_solve_ms
      }
    });
    
    await new Promise(resolve => setTimeout(resolve, 100));
    
    // Phase 3: Solve (blocks)
    const puzzle = wasmModule.solve_problem();
    
    self.postMessage({ type: 'PROGRESS', payload: { stage: 'Complete!', percent: 100 } });
    self.postMessage({ type: 'SUCCESS', payload: { puzzle } });
  } catch (error) {
    console.error('Generation error:', error);
    throw new Error(`Failed to generate crossword: ${error instanceof Error ? error.message : String(error)}`);
  }
}
