use crossword_core::{CrosswordPuzzle, Clue};
use crate::book::CrosswordBook;
use anyhow::Result;
use std::fs;

// Default ornamental decoration SVG embedded in code
const DEFAULT_DECORATION_SVG: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<svg width="300" height="60" viewBox="0 0 300 60" xmlns="http://www.w3.org/2000/svg">
  <g stroke="black" stroke-width="1.5" fill="none">
    <path d="M 10,30 Q 30,10 50,30 Q 70,50 90,30" stroke-linecap="round"/>
    <rect x="115" y="15" width="15" height="15" fill="black"/>
    <rect x="135" y="15" width="15" height="15"/>
    <rect x="155" y="15" width="15" height="15" fill="black"/>
    <rect x="115" y="30" width="15" height="15"/>
    <rect x="135" y="30" width="15" height="15" fill="black"/>
    <rect x="155" y="30" width="15" height="15"/>
    <rect x="115" y="45" width="15" height="15" fill="black"/>
    <rect x="135" y="45" width="15" height="15"/>
    <rect x="155" y="45" width="15" height="15" fill="black"/>
    <path d="M 190,30 Q 210,10 230,30 Q 250,50 270,30" stroke-linecap="round"/>
  </g>
</svg>"#;

pub struct LatexGenerator {}

impl LatexGenerator {
    pub fn new() -> Self {
        Self {}
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
        
        // Generate answer key
        latex.push_str("\\clearpage\n");
        latex.push_str("\\section*{Answer Key}\n\n");
        latex.push_str(&self.generate_answer_key(book.puzzles())?);
        
        latex.push_str("\\end{document}\n");
        
        Ok(latex)
    }

    fn generate_preamble(&self) -> String {
        String::from(
            r"\documentclass[11pt,letterpaper]{book}
\usepackage[top=0.75in,bottom=0.75in,left=0.6in,right=0.6in,headheight=15pt]{geometry}
\usepackage{tikz}
\usepackage{multicol}
\usepackage{enumitem}
\usepackage{amsmath}
\usepackage{graphicx}
\usepackage{svg}
\usepackage[T1]{fontenc}
\usepackage{lmodern}
\usepackage{xcolor}
\usepackage{fancyhdr}

% Page style with more header space
\pagestyle{fancy}
\fancyhf{}
\fancyhead[C]{\thepage}
\renewcommand{\headrulewidth}{0pt}
\setlength{\headsep}{0.4in}

% Custom title page commands
\newcommand{\subtitle}[1]{\Large #1}
\newcommand{\edition}[1]{\large \textit{#1}}

\setlength{\parindent}{0pt}
\setlength{\columnsep}{1.2em}

% Tighter list spacing
\setlist[enumerate]{itemsep=0.3em,parsep=0pt,topsep=0.3em}

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

    fn generate_answer_key(&self, puzzles: &[CrosswordPuzzle]) -> Result<String> {
        let mut latex = String::new();
        
        // 4 puzzles per page, 2x2 grid
        for (page_idx, chunk) in puzzles.chunks(4).enumerate() {
            if page_idx > 0 {
                latex.push_str("\\clearpage\n\n");
            }
            
            for (chunk_idx, puzzle) in chunk.iter().enumerate() {
                let puzzle_num = page_idx * 4 + chunk_idx + 1;
                
                // Start a minipage for each answer (2 per row)
                if chunk_idx % 2 == 0 {
                    latex.push_str("\\noindent\\begin{minipage}[t]{0.48\\textwidth}\n");
                } else {
                    latex.push_str("\\hfill\n");
                    latex.push_str("\\begin{minipage}[t]{0.48\\textwidth}\n");
                }
                
                latex.push_str("\\centering\n");
                latex.push_str(&format!("{{\\large\\textbf{{Puzzle {}}}}}\n\n", puzzle_num));
                latex.push_str("\\vspace{0.3cm}\n\n");
                latex.push_str(&self.generate_answer_grid(&puzzle.grid)?);
                latex.push_str("\\end{minipage}\n");
                
                // Line break after every 2 puzzles
                if chunk_idx % 2 == 1 && chunk_idx < chunk.len() - 1 {
                    latex.push_str("\n\n\\vspace{1cm}\n\n");
                }
            }
            
            // Add extra space at end if odd number on last page
            if chunk.len() % 2 == 1 {
                latex.push_str("\n\n\\vspace{1cm}\n\n");
            }
        }
        
        Ok(latex)
    }

    fn generate_answer_grid(&self, grid: &[Vec<Option<char>>]) -> Result<String> {
        let size = grid.len();
        let mut latex = String::new();
        
        // Scale appropriately for 2 columns - use a fraction of linewidth
        // This makes each grid fit nicely in its minipage
        let scale_factor = 0.85;
        
        latex.push_str(&format!(
            "\\begin{{tikzpicture}}[x={{{}\\linewidth/{}}},y={{{}\\linewidth/{}}}]\n",
            scale_factor, size, scale_factor, size
        ));
        
        // Draw cells with letters
        for row in 0..size {
            for col in 0..size {
                let x = col;
                let y = size - 1 - row;
                
                if let Some(letter) = grid[row][col] {
                    // White cell with letter
                    latex.push_str(&format!(
                        "\\draw ({},{}) rectangle ({},{});\n",
                        x, y, x + 1, y + 1
                    ));
                    
                    // Add letter in center
                    latex.push_str(&format!(
                        "\\node[font=\\footnotesize] at ({},{}) {{{}}};\n",
                        x as f32 + 0.5, y as f32 + 0.5, letter
                    ));
                } else {
                    // Black cell
                    latex.push_str(&format!(
                        "\\fill ({},{}) rectangle ({},{});\n",
                        x, y, x + 1, y + 1
                    ));
                }
            }
        }
        
        latex.push_str("\\end{tikzpicture}\n");
        
        Ok(latex)
    }

    fn generate_puzzle(&self, puzzle: &CrosswordPuzzle) -> Result<String> {
        let mut latex = String::new();
        
        // Add more space after the section header
        latex.push_str("\\vspace{0.5cm}\n");
        
        // Generate grid
        latex.push_str("\\begin{center}\n");
        latex.push_str(&self.generate_grid(&puzzle.grid)?);
        latex.push_str("\\end{center}\n");
        latex.push_str("\\vspace{0.5cm}\n\n");
        
        // Generate clues
        latex.push_str(&self.generate_clues(&puzzle.across_clues, &puzzle.down_clues));
        
        Ok(latex)
    }

    fn generate_grid(&self, grid: &[Vec<Option<char>>]) -> Result<String> {
        let size = grid.len();
        let mut latex = String::new();
        
        // Calculate dynamic cell size based on grid dimensions
        // Use textwidth to scale proportionally
        // For a 10x10 grid: ~0.7\textwidth, for 16x16: ~0.9\textwidth
        let width_ratio = if size <= 10 {
            0.7
        } else if size <= 15 {
            0.85
        } else {
            0.9
        };
        
        latex.push_str(&format!(
            "\\begin{{tikzpicture}}[x={{{}\\textwidth/{}}},y={{{}\\textwidth/{}}}]\n",
            width_ratio, size, width_ratio, size
        ));
        
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
                let x = col;
                let y = size - 1 - row;
                
                if grid[row][col].is_some() {
                    // White cell
                    latex.push_str(&format!(
                        "\\draw ({},{}) rectangle ({},{});\n",
                        x, y, x + 1, y + 1
                    ));
                    
                    // Add number if present
                    if let Some(num) = numbers[row][col] {
                        latex.push_str(&format!(
                            "\\node[anchor=north west,font=\\scriptsize,inner sep=0.02] at ({},{}) {{{}}};\n",
                            x as f32 + 0.05, y as f32 + 0.95, num
                        ));
                    }
                } else {
                    // Black cell
                    latex.push_str(&format!(
                        "\\fill ({},{}) rectangle ({},{});\n",
                        x, y, x + 1, y + 1
                    ));
                }
            }
        }
        
        latex.push_str("\\end{tikzpicture}\n");
        
        Ok(latex)
    }

    fn generate_clues(&self, across_clues: &[Clue], down_clues: &[Clue]) -> String {
        let mut latex = String::new();
        
        // Use minipages with [t] top alignment instead of multicols
        latex.push_str("\\noindent\\begin{minipage}[t]{0.48\\textwidth}\n");
        latex.push_str("\\subsection*{Across}\n");
        latex.push_str("\\raggedright\n");
        latex.push_str("\\begin{enumerate}\n");
        for clue in across_clues {
            latex.push_str(&format!(
                "\\setcounter{{enumi}}{{{}}} \\item {}\n",
                clue.number - 1,
                escape_latex(&clue.clue)
            ));
        }
        latex.push_str("\\end{enumerate}\n");
        latex.push_str("\\end{minipage}\n");
        latex.push_str("\\hfill\n");
        latex.push_str("\\begin{minipage}[t]{0.48\\textwidth}\n");
        latex.push_str("\\subsection*{Down}\n");
        latex.push_str("\\raggedright\n");
        latex.push_str("\\begin{enumerate}\n");
        for clue in down_clues {
            latex.push_str(&format!(
                "\\setcounter{{enumi}}{{{}}} \\item {}\n",
                clue.number - 1,
                escape_latex(&clue.clue)
            ));
        }
        latex.push_str("\\end{enumerate}\n");
        latex.push_str("\\end{minipage}\n");
        
        latex
    }
}

impl Default for LatexGenerator {
    fn default() -> Self {
        Self::new()
    }
}

fn escape_latex(s: &str) -> String {
    s.replace('\\', "\\textbackslash{}")
        .replace('&', "\\&")
        .replace('%', "\\%")
        .replace('$', "\\$")
        .replace('#', "\\#")
        .replace('_', "\\_")
        .replace('{', "\\{")
        .replace('}', "\\}")
        .replace('~', "\\textasciitilde{}")
        .replace('^', "\\textasciicircum{}")
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
