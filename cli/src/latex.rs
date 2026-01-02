use crossword_core::{CrosswordPuzzle, Clue};
use crate::book::CrosswordBook;
use anyhow::Result;

pub struct LatexGenerator {
    cell_size: f32,
}

impl LatexGenerator {
    pub fn new() -> Self {
        Self {
            cell_size: 0.6,
        }
    }

    pub fn generate_document(&self, book: &CrosswordBook) -> Result<String> {
        let mut latex = String::new();
        
        // Preamble
        latex.push_str(&self.generate_preamble(book.config().title.as_str()));
        
        // Begin document
        latex.push_str("\\begin{document}\n");
        latex.push_str("\\maketitle\n\n");
        
        // Generate each puzzle
        for (idx, puzzle) in book.puzzles().iter().enumerate() {
            latex.push_str(&format!("\\section*{{Puzzle {}}}\n\n", idx + 1));
            latex.push_str(&self.generate_puzzle(puzzle)?);
            latex.push_str("\\clearpage\n\n");
        }
        
        latex.push_str("\\end{document}\n");
        
        Ok(latex)
    }

    fn generate_preamble(&self, title: &str) -> String {
        format!(
            r"\documentclass[11pt,letterpaper]{{article}}
\usepackage[margin=0.75in]{{geometry}}
\usepackage{{tikz}}
\usepackage{{multicol}}
\usepackage{{enumitem}}
\usepackage{{amsmath}}

\title{{{title}}}
\date{{\today}}

\setlength{{\parindent}}{{0pt}}
\setlength{{\columnsep}}{{1.5em}}

",
            title = escape_latex(title)
        )
    }

    fn generate_puzzle(&self, puzzle: &CrosswordPuzzle) -> Result<String> {
        let mut latex = String::new();
        
        // Generate grid
        latex.push_str("\\begin{center}\n");
        latex.push_str(&self.generate_grid(&puzzle.grid)?);
        latex.push_str("\\end{center}\n\n");
        
        // Generate clues
        latex.push_str(&self.generate_clues(&puzzle.across_clues, &puzzle.down_clues));
        
        Ok(latex)
    }

    fn generate_grid(&self, grid: &[Vec<Option<char>>]) -> Result<String> {
        let size = grid.len();
        let mut latex = String::new();
        
        latex.push_str("\\begin{tikzpicture}[scale=1]\n");
        
        // Number tracking for clues
        let mut numbers = vec![vec![None; size]; size];
        let mut next_number = 1;
        
        // Assign numbers to cells that start words
        for row in 0..size {
            for col in 0..size {
                if grid[row][col].is_some() {
                    // Check if this starts an across word
                    let starts_across = col == 0 || grid[row][col - 1].is_none();
                    let has_across = col < size - 1 && grid[row][col + 1].is_some();
                    
                    // Check if this starts a down word
                    let starts_down = row == 0 || grid[row - 1][col].is_none();
                    let has_down = row < size - 1 && grid[row + 1][col].is_some();
                    
                    if (starts_across && has_across) || (starts_down && has_down) {
                        numbers[row][col] = Some(next_number);
                        next_number += 1;
                    }
                }
            }
        }
        
        // Draw cells
        for row in 0..size {
            for col in 0..size {
                let x = col as f32 * self.cell_size;
                let y = (size - 1 - row) as f32 * self.cell_size;
                
                if grid[row][col].is_some() {
                    // White cell
                    latex.push_str(&format!(
                        "\\draw ({:.2},{:.2}) rectangle ({:.2},{:.2});\n",
                        x, y, x + self.cell_size, y + self.cell_size
                    ));
                    
                    // Add number if present
                    if let Some(num) = numbers[row][col] {
                        latex.push_str(&format!(
                            "\\node[anchor=north west,font=\\tiny] at ({:.2},{:.2}) {{{}}};\n",
                            x + 0.05, y + self.cell_size - 0.05, num
                        ));
                    }
                } else {
                    // Black cell
                    latex.push_str(&format!(
                        "\\fill ({:.2},{:.2}) rectangle ({:.2},{:.2});\n",
                        x, y, x + self.cell_size, y + self.cell_size
                    ));
                }
            }
        }
        
        latex.push_str("\\end{tikzpicture}\n");
        
        Ok(latex)
    }

    fn generate_clues(&self, across_clues: &[Clue], down_clues: &[Clue]) -> String {
        let mut latex = String::new();
        
        latex.push_str("\\begin{multicols}{2}\n");
        
        // Across clues
        latex.push_str("\\subsection*{Across}\n");
        latex.push_str("\\begin{enumerate}[itemsep=0.5em]\n");
        for clue in across_clues {
            latex.push_str(&format!(
                "\\setcounter{{enumi}}{{{}}} \\item {}\n",
                clue.number - 1,
                escape_latex(&clue.clue)
            ));
        }
        latex.push_str("\\end{enumerate}\n\n");
        
        latex.push_str("\\columnbreak\n\n");
        
        // Down clues
        latex.push_str("\\subsection*{Down}\n");
        latex.push_str("\\begin{enumerate}[itemsep=0.5em]\n");
        for clue in down_clues {
            latex.push_str(&format!(
                "\\setcounter{{enumi}}{{{}}} \\item {}\n",
                clue.number - 1,
                escape_latex(&clue.clue)
            ));
        }
        latex.push_str("\\end{enumerate}\n");
        
        latex.push_str("\\end{multicols}\n");
        
        latex
    }
}

impl Default for LatexGenerator {
    fn default() -> Self {
        Self::new()
    }
}

fn escape_latex(s: &str) -> String {
    s.replace('&', "\\&")
        .replace('%', "\\%")
        .replace('$', "\\$")
        .replace('#', "\\#")
        .replace('_', "\\_")
        .replace('{', "\\{")
        .replace('}', "\\}")
        .replace('~', "\\textasciitilde{}")
        .replace('^', "\\textasciicircum{}")
        .replace('\\', "\\textbackslash{}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_latex_escaping() {
        assert_eq!(escape_latex("Test & Co."), "Test \\& Co.");
        assert_eq!(escape_latex("$100"), "\\$100");
        assert_eq!(escape_latex("50%"), "50\\%");
        assert_eq!(escape_latex("C++ #include"), "C++ \\#include");
    }
}
