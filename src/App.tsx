import { useEffect, useState, useRef } from 'react';
import { ControlPanel } from './components/ControlPanel';
import { CrosswordGrid } from './components/CrosswordGrid';
import { CluesList } from './components/CluesList';
import { LoadingOverlay } from './components/LoadingOverlay';
import { ErrorBoundary } from './components/ErrorBoundary';
import { 
  CrosswordPuzzle, 
  GenerationState, 
  WorkerMessage 
} from './types/crossword';

function App() {
  const [state, setState] = useState<GenerationState>('UNINITIALIZED');
  const [puzzle, setPuzzle] = useState<CrosswordPuzzle | null>(null);
  const [progress, setProgress] = useState({ stage: '', percent: 0 });
  const [error, setError] = useState<string | null>(null);
  const [showAnswers, setShowAnswers] = useState(false);
  
  const workerRef = useRef<Worker | null>(null);
  const progressWorkerRef = useRef<Worker | null>(null);

  useEffect(() => {
    // Initialize WASM worker
    setState('INITIALIZING');
    const worker = new Worker(
      new URL('./workers/crossword.worker.ts', import.meta.url),
      { type: 'module' }
    );
    
    // Initialize progress worker
    const progressWorker = new Worker(
      new URL('./workers/progress.worker.ts', import.meta.url),
      { type: 'module' }
    );
    
    progressWorker.onmessage = (event) => {
      if (event.data.type === 'UPDATE') {
        setProgress({ 
          stage: event.data.stage, 
          percent: event.data.percent 
        });
      }
    };

    worker.onmessage = (event: MessageEvent<WorkerMessage>) => {
      const { type, payload } = event.data;

      switch (type) {
        case 'READY':
          setState('READY');
          break;
        
        case 'ESTIMATE':
          // Send estimate to progress worker
          const { solving_ms } = payload;
          const encodingEndTime = Date.now();
          const actualEncodingTime = encodingEndTime - ((window as any).progressStartTime || Date.now());
          const totalEstimate = actualEncodingTime + solving_ms;
          
          
          if (progressWorkerRef.current) {
            progressWorkerRef.current.postMessage({
              type: 'UPDATE_ESTIMATE',
              payload: { estimatedTime: totalEstimate }
            });
          }
          break;
          break;

        case 'ENCODING_COMPLETE':
          const { encoding_time_ms, estimated_solve_ms } = payload;
          const newTotal = encoding_time_ms + estimated_solve_ms;
          
          if (progressWorkerRef.current) {
            progressWorkerRef.current.postMessage({
              type: 'UPDATE_ESTIMATE',
              payload: { 
                estimatedTime: newTotal,
                actualEncodingTime: encoding_time_ms
              }
            });
          }
          break;

        case 'PROGRESS':
          setProgress(payload);
          break;

        case 'SUCCESS':
          if (progressWorkerRef.current) {
            progressWorkerRef.current.postMessage({ type: 'STOP' });
          }
          setPuzzle(payload.puzzle);
          setState('COMPLETE');
          break;

        case 'ERROR':
          if (progressWorkerRef.current) {
            progressWorkerRef.current.postMessage({ type: 'STOP' });
          }
          setError(payload.message);
          setState('ERROR');
          setTimeout(() => {
            setState('READY');
            setError(null);
          }, 3000);
          break;
      }
    };

    worker.onerror = (error) => {
      console.error('Worker error:', error);
      setError('Worker initialization failed');
      setState('ERROR');
    };

    workerRef.current = worker;
    progressWorkerRef.current = progressWorker;
    
    worker.postMessage({ type: 'INIT' });

    return () => {
      worker.terminate();
      progressWorker.terminate();
    };
  }, []);

  const handleGenerate = (size: number) => {
    if (!workerRef.current || state === 'GENERATING') return;

    setState('GENERATING');
    setProgress({ stage: 'Starting...', percent: 0 });
    
    (window as any).progressStartTime = Date.now();
    
    // Start progress worker with initial estimate
    if (progressWorkerRef.current) {
      progressWorkerRef.current.postMessage({
        type: 'START',
        payload: { estimatedTime: 5000 } // Initial guess, will be updated
      });
    }
    
    workerRef.current.postMessage({
      type: 'GENERATE',
      payload: { size },
    });
  };

  const handleCancel = () => {
    if (workerRef.current && state === 'GENERATING') {
      // Stop progress worker
      if (progressWorkerRef.current) {
        progressWorkerRef.current.postMessage({ type: 'STOP' });
      }
      
      // Terminate WASM worker
      workerRef.current.terminate();
      
      // Recreate WASM worker
      setState('INITIALIZING');
      const worker = new Worker(
        new URL('./workers/crossword.worker.ts', import.meta.url),
        { type: 'module' }
      );
      
      worker.onmessage = workerRef.current.onmessage;
      worker.onerror = workerRef.current.onerror;
      
      workerRef.current = worker;
      worker.postMessage({ type: 'INIT' });
    }
  };

  return (
    <ErrorBoundary>
      <div className="min-h-screen bg-gray-50 py-4 md:py-8 px-2 md:px-4">
        <div className="max-w-7xl mx-auto">
          <header className="text-center mb-6 md:mb-8 px-2">
            <h1 className="text-3xl md:text-4xl font-bold text-gray-900 mb-2">
              Crossword Generator
            </h1>
            <p className="text-sm md:text-base text-gray-600">
              SAT-based puzzle generation with high-density optimization
            </p>
          </header>

          {state === 'INITIALIZING' && (
            <div className="text-center py-12">
              <div className="inline-block w-8 h-8 border-4 border-gray-200 border-t-gray-900 rounded-full animate-spin mb-4"></div>
              <p className="text-gray-600">Initializing...</p>
            </div>
          )}

          {error && (
            <div className="mb-4 p-4 bg-red-50 border border-red-200 rounded-lg text-red-800 text-sm no-print">
              {error}
            </div>
          )}

          {(state === 'READY' || state === 'COMPLETE' || state === 'GENERATING') && (
            <div className="grid grid-cols-1 lg:grid-cols-4 gap-6">
              <div className="lg:col-span-1 order-first lg:order-first">
                <div className="sticky top-4">
                  <ControlPanel
                    onGenerate={handleGenerate}
                    isGenerating={state === 'GENERATING'}
                    metadata={puzzle?.metadata || null}
                    showAnswers={showAnswers}
                    onToggleAnswers={setShowAnswers}
                    onPrint={() => window.print()}
                  />
                </div>
              </div>

              <div className="lg:col-span-3 space-y-4 lg:space-y-8 order-last lg:order-last">
                <div className="print-layout">
                {puzzle && (
                  <>
                    <div className="bg-white crossword-container print:break-inside-avoid border border-gray-200 rounded-lg p-2 md:p-6">
                      <CrosswordGrid 
                        grid={puzzle.grid} 
                        size={puzzle.grid.length}
                        showAnswers={showAnswers}
                      />
                    </div>

                    <div className="bg-white border border-gray-200 rounded-lg p-6">
                      <h2 className="text-2xl font-semibold mb-6">Clues</h2>
                      <CluesList
                        acrossClues={puzzle.across_clues}
                        downClues={puzzle.down_clues}
                      />
                    </div>
                  </>
                )}
                </div>

                {!puzzle && state === 'READY' && (
                  <div className="bg-white border border-gray-200 rounded-lg p-12 text-center">
                    <p className="text-gray-500">
                      Click "Generate" to create a crossword puzzle
                    </p>
                  </div>
                )}
              </div>
            </div>
          )}

          {state === 'GENERATING' && (
            <LoadingOverlay
              stage={progress.stage}
              percent={progress.percent}
              onCancel={handleCancel}
            />
          )}
        </div>
      </div>
    </ErrorBoundary>
  );
}

export default App;
