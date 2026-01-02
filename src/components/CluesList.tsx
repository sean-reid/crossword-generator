import { Clue } from '../types/crossword';

interface CluesListProps {
  acrossClues: Clue[];
  downClues: Clue[];
}

export function CluesList({ acrossClues, downClues }: CluesListProps) {
  return (
    <div className="grid md:grid-cols-2 gap-8">
      <div>
        <h3 className="text-lg font-semibold mb-3">Across</h3>
        <div className="space-y-2">
          {acrossClues.map((clue) => (
            <div key={`across-${clue.number}`} className="text-sm">
              <span className="font-medium">{clue.number}.</span>{' '}
              <span className="text-gray-700">{clue.clue}</span>
            </div>
          ))}
        </div>
      </div>

      <div>
        <h3 className="text-lg font-semibold mb-3">Down</h3>
        <div className="space-y-2">
          {downClues.map((clue) => (
            <div key={`down-${clue.number}`} className="text-sm">
              <span className="font-medium">{clue.number}.</span>{' '}
              <span className="text-gray-700">{clue.clue}</span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
