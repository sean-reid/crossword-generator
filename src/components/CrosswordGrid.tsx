interface CrosswordGridProps {
  grid: (string | null)[][];
  size: number;
  showAnswers?: boolean;
}

export function CrosswordGrid({ grid, size, showAnswers = true }: CrosswordGridProps) {
  // Calculate cell numbers
  const cellNumbers: (number | null)[][] = Array(size).fill(null).map(() => Array(size).fill(null));
  let currentNumber = 1;

  for (let y = 0; y < size; y++) {
    for (let x = 0; x < size; x++) {
      if (grid[y][x] != null) {  // Use != to catch both null and undefined
        const startsAcross = 
          (x === 0 || grid[y][x - 1] == null) && 
          x + 1 < size && 
          grid[y][x + 1] != null;
        
        const startsDown = 
          (y === 0 || grid[y - 1][x] == null) && 
          y + 1 < size && 
          grid[y + 1][x] != null;

        if (startsAcross || startsDown) {
          cellNumbers[y][x] = currentNumber;
          console.log(`Clue #${currentNumber} at (${x},${y}) - across:${startsAcross}, down:${startsDown}, letter:${grid[y][x]}`);
          currentNumber++;
        }
      }
    }
  }
  
  console.log(`Total clue numbers: ${currentNumber - 1}`);

  const cellSize = Math.min(40, Math.floor(600 / size));

  return (
    <div className="flex justify-center">
      <div 
        className="inline-grid gap-0 border-2 border-gray-900"
        style={{
          gridTemplateColumns: `repeat(${size}, ${cellSize}px)`,
        }}
      >
        {grid.map((row, y) =>
          row.map((cell, x) => (
            <div
              key={`${x}-${y}`}
              className={`relative border border-gray-300 ${
                cell == null ? 'bg-gray-900' : 'bg-white'
              }`}
              style={{
                width: `${cellSize}px`,
                height: `${cellSize}px`,
              }}
            >
              {cellNumbers[y][x] && cellNumbers[y][x] > 0 && (
                <span className="absolute top-0.5 left-0.5 text-[11px] font-bold text-blue-600 leading-none z-20 bg-white px-0.5">
                  {cellNumbers[y][x]}
                </span>
              )}
              {cell && showAnswers && (
                <div className="absolute inset-0 flex items-center justify-center pt-2">
                  <span className="text-base font-medium text-gray-900 uppercase print:text-sm">
                    {cell}
                  </span>
                </div>
              )}
            </div>
          ))
        )}
      </div>
    </div>
  );
}
