interface LoadingOverlayProps {
  stage: string;
  percent: number;
  onCancel: () => void;
}

export function LoadingOverlay({ stage, percent, onCancel }: LoadingOverlayProps) {
  return (
    <div className="fixed inset-0 bg-white bg-opacity-90 flex items-center justify-center z-50 p-4">
      <div className="max-w-md w-full">
        <div className="bg-white border border-gray-200 rounded-lg p-6 md:p-8 shadow-sm">
          <div className="flex flex-col items-center">
            <div className="w-10 h-10 md:w-12 md:h-12 mb-4">
              <div className="w-full h-full border-4 border-gray-200 border-t-gray-900 rounded-full animate-spin"></div>
            </div>
            
            <h3 className="text-base md:text-lg font-medium mb-2">Generating Crossword</h3>
            <p className="text-xs md:text-sm text-gray-600 mb-4 text-center">{stage}</p>
            
            <div className="w-full bg-gray-100 rounded-full h-2 mb-4 md:mb-6">
              <div 
                className="bg-gray-900 h-2 rounded-full transition-all duration-300"
                style={{ width: `${percent}%` }}
              ></div>
            </div>
            
            <button
              onClick={onCancel}
              className="px-4 py-2 text-xs md:text-sm text-gray-600 hover:text-gray-900 transition-colors"
            >
              Cancel
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
