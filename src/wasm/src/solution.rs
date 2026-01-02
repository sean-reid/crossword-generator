use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Placement {
    pub word: String,
    pub x: usize,
    pub y: usize,
    pub horizontal: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clue {
    pub number: usize,
    pub word: String,
    pub clue: String,
    pub x: usize,
    pub y: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrosswordMetadata {
    pub density: f32,
    pub word_count: usize,
    pub total_letters: usize,
    pub generation_time_ms: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrosswordPuzzle {
    pub grid: Vec<Vec<Option<char>>>,
    pub across_clues: Vec<Clue>,
    pub down_clues: Vec<Clue>,
    pub metadata: CrosswordMetadata,
}

impl CrosswordPuzzle {
    pub fn from_placements(
        placements: &[Placement],
        size: usize,
        clue_fn: impl Fn(&str) -> String,
        generation_time_ms: u32,
    ) -> Self {
        let mut grid = vec![vec![None; size]; size];
        
        for placement in placements {
            let chars: Vec<char> = placement.word.chars().collect();
            for (i, &ch) in chars.iter().enumerate() {
                let (x, y) = if placement.horizontal {
                    (placement.x + i, placement.y)
                } else {
                    (placement.x, placement.y + i)
                };
                grid[y][x] = Some(ch);
            }
        }
        
        let mut cell_numbers = vec![vec![0; size]; size];
        let mut current_number = 1;
        
        for y in 0..size {
            for x in 0..size {
                if grid[y][x].is_some() {
                    let starts_across = (x == 0 || grid[y][x - 1].is_none()) 
                        && x + 1 < size && grid[y][x + 1].is_some();
                    let starts_down = (y == 0 || grid[y - 1][x].is_none()) 
                        && y + 1 < size && grid[y + 1][x].is_some();
                    
                    if starts_across || starts_down {
                        cell_numbers[y][x] = current_number;
                        current_number += 1;
                    }
                }
            }
        }
        
        let mut across_clues = Vec::new();
        let mut down_clues = Vec::new();
        
        for placement in placements {
            let number = cell_numbers[placement.y][placement.x];
            let clue = Clue {
                number,
                word: placement.word.clone(),
                clue: clue_fn(&placement.word),
                x: placement.x,
                y: placement.y,
            };
            
            if placement.horizontal {
                across_clues.push(clue);
            } else {
                down_clues.push(clue);
            }
        }
        
        across_clues.sort_by_key(|c| c.number);
        down_clues.sort_by_key(|c| c.number);
        
        use std::collections::HashSet;
        let mut filled_cells = HashSet::new();
        for placement in placements {
            for i in 0..placement.word.len() {
                let (x, y) = if placement.horizontal {
                    (placement.x + i, placement.y)
                } else {
                    (placement.x, placement.y + i)
                };
                filled_cells.insert((x, y));
            }
        }
        
        let total_cells = size * size;
        let density = filled_cells.len() as f32 / total_cells as f32;
        let total_letters = placements.iter().map(|p| p.word.len()).sum();
        
        CrosswordPuzzle {
            grid,
            across_clues,
            down_clues,
            metadata: CrosswordMetadata {
                density,
                word_count: placements.len(),
                total_letters,
                generation_time_ms,
            },
        }
    }
}
