use crossword_core::CrosswordPuzzle;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookConfig {
    pub title: String,
    pub grid_size: usize,
    pub puzzles_per_page: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CrosswordBook {
    config: BookConfig,
    puzzles: Vec<CrosswordPuzzle>,
}

impl CrosswordBook {
    pub fn new(config: BookConfig) -> Self {
        Self {
            config,
            puzzles: Vec::new(),
        }
    }

    pub fn add_puzzle(&mut self, puzzle: CrosswordPuzzle) {
        self.puzzles.push(puzzle);
    }

    pub fn puzzles(&self) -> &[CrosswordPuzzle] {
        &self.puzzles
    }

    pub fn puzzle_count(&self) -> usize {
        self.puzzles.len()
    }

    pub fn config(&self) -> &BookConfig {
        &self.config
    }

    pub fn save_to_file(&self, path: &std::path::Path) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)
    }

    pub fn load_from_file(path: &std::path::Path) -> std::io::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let book = serde_json::from_str(&json)?;
        Ok(book)
    }
}
