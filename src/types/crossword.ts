export interface Clue {
  number: number;
  word: string;
  clue: string;
  x: number;
  y: number;
}

export interface CrosswordMetadata {
  density: number;
  word_count: number;
  total_letters: number;
  generation_time_ms: number;
}

export interface CrosswordPuzzle {
  grid: (string | null)[][];
  across_clues: Clue[];
  down_clues: Clue[];
  metadata: CrosswordMetadata;
}

export interface DictionaryStats {
  word_count: number;
  avg_word_length: number;
  max_word_length: number;
}

export type WorkerMessageType = 
  | 'INIT'
  | 'GENERATE'
  | 'CANCEL'
  | 'READY'
  | 'ESTIMATE'
  | 'ENCODING_COMPLETE'
  | 'PROGRESS'
  | 'SUCCESS'
  | 'ERROR';

export interface WorkerMessage {
  type: WorkerMessageType;
  payload?: any;
}

export type GenerationState = 
  | 'UNINITIALIZED'
  | 'INITIALIZING'
  | 'READY'
  | 'GENERATING'
  | 'COMPLETE'
  | 'ERROR';
