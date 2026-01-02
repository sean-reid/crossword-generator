use crossword_core::{CrosswordPuzzle, Clue};
use crate::book::CrosswordBook;
use anyhow::Result;
use std::fs;

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
        
        // Preamble with SVG support
        latex.push_str(&self.generate_preamble());
        
        // Begin document
        latex.push_str("\\begin{document}\n\n");
        
        // Generate professional title page
        latex.push_str(&self.generate_title_page(book.config())?);
        
        // Generate each puzzle
        for (idx, puzzle) in book.puzzles().iter().enumerate() {
            latex.push_str(&format!("\\section*{{Puzzle {}}}\n\n", idx + 1));
            latex.push_str(&self.generate_puzzle(puzzle)?);
            latex.push_str("\\clearpage\n\n");
        }
        
        latex.push_str("\\end{document}\n");
        
        Ok(latex)
    }

    fn generate_preamble(&self) -> String {
        String::from(
            r"\documentclass[11pt,letterpaper]{book}
\usepackage[margin=0.75in]{geometry}
\usepackage{tikz}
\usepackage{multicol}
\usepackage{enumitem}
\usepackage{amsmath}
\usepackage{graphicx}
\usepackage{svg}
\usepackage[T1]{fontenc}
\usepackage{lmodern}
\usepackage{xcolor}

% Custom title page commands
\newcommand{\subtitle}[1]{\Large #1}
\newcommand{\edition}[1]{\large \textit{#1}}

\setlength{\parindent}{0pt}
\setlength{\columnsep}{1.5em}

% Remove default title formatting
\makeatletter
\renewcommand{\maketitle}{}
\makeatother

"
        )
    }

    fn generate_title_page(&self, config: &crate::book::BookConfig) -> Result<String> {
        let mut latex = String::new();
        
        latex.push_str("\\begin{titlepage}\n");
        latex.push_str("\\centering\n");
        latex.push_str("\\vspace*{2cm}\n\n");
        
        // Cover SVG if provided
        if let Some(ref cover_svg) = config.cover_svg_path {
            if fs::metadata(cover_svg).is_ok() {
                latex.push_str(&format!(
                    "\\includesvg[width=0.7\\textwidth]{{{}}}\n\n\\vspace{{2cm}}\n\n",
                    escape_latex(cover_svg)
                ));
            } else {
                eprintln!("Warning: Cover file not found: {}", cover_svg);
            }
        }
        
        // Title
        latex.push_str(&format!(
            "{{\\Huge\\bfseries {}}}\n\n\\vspace{{0.5cm}}\n\n",
            escape_latex(&config.title)
        ));
        
        // Title page decoration SVG
        if let Some(ref title_svg) = config.title_svg_path {
            if fs::metadata(title_svg).is_ok() {
                latex.push_str(&format!(
                    "\\includesvg[width=0.5\\textwidth]{{{}}}\n\n\\vspace{{1cm}}\n\n",
                    escape_latex(title_svg)
                ));
            } else {
                eprintln!("Warning: Title decoration file not found: {}", title_svg);
                latex.push_str(&self.generate_default_decoration());
            }
        } else {
            // Use embedded default decoration
            latex.push_str(&self.generate_default_decoration());
        }
        
        // Description
        if let Some(ref desc) = config.description {
            latex.push_str(&format!(
                "{{\\Large\\textit{{{}}}}}\n\n\\vspace{{1.5cm}}\n\n",
                escape_latex(desc)
            ));
        }
        
        // Author
        if let Some(ref author) = config.author {
            latex.push_str(&format!(
                "{{\\Large {}}}\n\n\\vspace{{0.3cm}}\n\n",
                escape_latex(author)
            ));
        }
        
        // Edition
        if let Some(ref edition) = config.edition {
            latex.push_str(&format!(
                "{{\\large\\textit{{{}}}}}\n\n\\vspace{{1cm}}\n\n",
                escape_latex(edition)
            ));
        }
        
        latex.push_str("\\vfill\n\n");
        
        // Publisher and ISBN at bottom
        if config.publisher.is_some() || config.isbn.is_some() {
            latex.push_str("\\begin{minipage}{0.8\\textwidth}\n");
            latex.push_str("\\centering\n");
            
            if let Some(ref publisher) = config.publisher {
                latex.push_str(&format!(
                    "{{\\large {}}}\n\n\\vspace{{0.2cm}}\n\n",
                    escape_latex(publisher)
                ));
            }
            
            if let Some(ref isbn) = config.isbn {
                latex.push_str(&format!(
                    "ISBN: {}\n\n\\vspace{{0.2cm}}\n\n",
                    escape_latex(isbn)
                ));
            }
            
            latex.push_str("\\end{minipage}\n\n");
        }
        
        // Copyright notice
        if let Some(ref year) = config.copyright_year {
            latex.push_str("\\vspace{0.5cm}\n\n");
            latex.push_str(&format!(
                "{{\\small Copyright \\copyright{} {}}}\n\n",
                year,
                config.author.as_deref().unwrap_or("All Rights Reserved")
            ));
            latex.push_str("\\vspace{0.1cm}\n\n");
            latex.push_str("{\\small All rights reserved.}\n\n");
        }
        
        latex.push_str("\\end{titlepage}\n\n");
        
        // Add copyright/info page
        latex.push_str("\\clearpage\n");
        latex.push_str("\\thispagestyle{empty}\n");
        latex.push_str("\\vspace*{\\fill}\n");
        latex.push_str("\\begin{center}\n");
        latex.push_str("\\textit{This page intentionally left blank.}\n");
        latex.push_str("\\end{center}\n");
        latex.push_str("\\vspace*{\\fill}\n");
        latex.push_str("\\clearpage\n\n");
        
        Ok(latex)
    }

    fn generate_default_decoration(&self) -> String {
        // Generate inline TikZ decoration instead of SVG
        // This avoids file path dependencies
        String::from(
            r"\begin{tikzpicture}[scale=0.4]
  \draw[line width=1pt] (0,1.5) .. controls (1,0.5) and (2.5,0.5) .. (3.5,1.5);
  \draw[line width=1pt] (4,1.5) .. controls (5,2.5) and (6.5,2.5) .. (7.5,1.5);
  \fill (9,0.5) rectangle (10,1.5);
  \draw (10.5,0.5) rectangle (11.5,1.5);
  \fill (12,0.5) rectangle (13,1.5);
  \draw (9,2) rectangle (10,3);
  \fill (10.5,2) rectangle (11.5,3);
  \draw (12,2) rectangle (13,3);
  \fill (9,3.5) rectangle (10,4.5);
  \draw (10.5,3.5) rectangle (11.5,4.5);
  \fill (12,3.5) rectangle (13,4.5);
  \draw[line width=1pt] (14.5,1.5) .. controls (15.5,0.5) and (17,0.5) .. (18,1.5);
  \draw[line width=1pt] (18.5,1.5) .. controls (19.5,2.5) and (21,2.5) .. (22,1.5);
\end{tikzpicture}

\\[1cm]

"
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
