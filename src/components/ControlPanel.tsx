import { useState } from 'react';
import { CrosswordMetadata } from '../types/crossword';

interface ControlPanelProps {
  onGenerate: (size: number) => void;
  isGenerating: boolean;
  metadata: CrosswordMetadata | null;
  showAnswers: boolean;
  onToggleAnswers: (show: boolean) => void;
  onPrint: () => void;
}

export function ControlPanel({ 
  onGenerate, 
  isGenerating, 
  metadata,
  showAnswers,
  onToggleAnswers,
  onPrint
}: ControlPanelProps) {
  const [size, setSize] = useState(12);

  const handleGenerate = () => {
    onGenerate(size);
  };

  return (
    <div className="bg-white border border-gray-200 rounded-lg p-6 no-print">
      <h2 className="text-xl font-semibold mb-4">Generate Crossword</h2>
      
      <div className="space-y-4 mb-6">
        <div>
          <label htmlFor="size" className="block text-sm font-medium text-gray-700 mb-1">
            Grid Size
          </label>
          <select
            id="size"
            value={size}
            onChange={(e) => setSize(Number(e.target.value))}
            disabled={isGenerating}
            className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-1 focus:ring-gray-900 disabled:bg-gray-50 disabled:text-gray-500"
          >
            <option value={8}>8 × 8</option>
            <option value={10}>10 × 10</option>
            <option value={12}>12 × 12</option>
          </select>
        </div>
      </div>

      <button
        onClick={handleGenerate}
        disabled={isGenerating}
        className="w-full px-4 py-3 bg-gray-900 text-white rounded-md hover:bg-gray-800 transition-colors disabled:bg-gray-400 disabled:cursor-not-allowed font-medium"
      >
        {isGenerating ? 'Generating...' : 'Generate'}
      </button>

      {metadata && (
        <div className="mt-4 pt-4 border-t border-gray-200">
          <h3 className="text-sm font-medium text-gray-700 mb-2">Statistics</h3>
          <div className="text-sm text-gray-600 space-y-1">
            <div>Words: {metadata.word_count}</div>
            <div>Letters: {metadata.total_letters}</div>
            <div>Density: {(metadata.density * 100).toFixed(1)}%</div>
            <div>Time: {(metadata.generation_time_ms / 1000).toFixed(2)}s</div>
          </div>
        </div>
      )}

      {metadata && (
        <div className="mt-6 pt-6 border-t border-gray-200 space-y-3">
          <h3 className="text-sm font-medium text-gray-700 mb-3">Options</h3>
          
          <div className="flex items-center justify-between">
            <span className="text-sm text-gray-700">Show answers</span>
            <button
              onClick={() => onToggleAnswers(!showAnswers)}
              className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors focus:outline-none focus:ring-2 focus:ring-gray-900 focus:ring-offset-2 ${
                showAnswers ? 'bg-gray-900' : 'bg-gray-300'
              }`}
              role="switch"
              aria-checked={showAnswers}
            >
              <span
                className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                  showAnswers ? 'translate-x-6' : 'translate-x-1'
                }`}
              />
            </button>
          </div>
          
          <button
            onClick={onPrint}
            className="w-full px-4 py-2 border border-gray-300 text-gray-700 rounded-md hover:bg-gray-50 transition-colors text-sm"
          >
            Print Puzzle
          </button>
        </div>
      )}
    </div>
  );
}
