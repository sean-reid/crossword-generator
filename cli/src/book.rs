use crossword_core::CrosswordPuzzle;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookConfig {
    pub title: String,
    pub author: Option<String>,
    pub publisher: Option<String>,
    pub edition: Option<String>,
    pub isbn: Option<String>,
    pub copyright_year: Option<String>,
    pub description: Option<String>,
    pub cover_svg_path: Option<String>,
    pub title_svg_path: Option<String>,
    pub grid_size: usize,
    pub puzzles_per_page: usize,
}

impl BookConfig {
    pub fn new(title: String, grid_size: usize) -> Self {
        Self {
            title,
            author: None,
            publisher: None,
            edition: None,
            isbn: None,
            copyright_year: None,
            description: None,
            cover_svg_path: None,
            title_svg_path: None,
            grid_size,
            puzzles_per_page: 1,
        }
    }
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
