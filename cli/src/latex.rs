use crossword_core::CrosswordPuzzle;
use crate::book::{CrosswordBook, KdpFormat};
use anyhow::Result;

pub struct LatexGenerator {}

impl LatexGenerator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn generate_document(&self, book: &CrosswordBook) -> Result<String> {
        let mut latex = String::new();
        
        // Preamble
        latex.push_str(&self.generate_preamble(book.config()));
        
        // Begin document
        latex.push_str("\\begin{document}\n\n");
        
        // Front matter (roman numerals)
        latex.push_str("\\frontmatter\n\n");
        
        // Title page
        latex.push_str(&self.generate_kdp_title_page(book.config())?);
        
        // Copyright page (must be on verso/left/even page)
        latex.push_str("\\clearpage\n");
        latex.push_str(&self.generate_copyright_page(book.config()));
        
        // Table of contents
        latex.push_str("\\clearpage\n");
        latex.push_str(&self.generate_toc(book.puzzle_count()));
        
        // Main matter (arabic numerals, starts on odd/right page)
        latex.push_str("\\cleardoublepage\n");
        latex.push_str("\\mainmatter\n\n");
        
        // Introduction page (will be page 1, odd/right)
        latex.push_str(&self.generate_introduction(book.config()));
        
        // Generate puzzles with facing pages (clues on left, grid on right)
        for (idx, puzzle) in book.puzzles().iter().enumerate() {
            latex.push_str(&self.generate_puzzle_spread(puzzle, idx + 1)?);
        }
        
        // Answer key
        latex.push_str("\\cleardoublepage\n");
        latex.push_str("\\chapter*{Answer Key}\n");
        latex.push_str("\\addcontentsline{toc}{chapter}{Answer Key}\n\n");
        latex.push_str(&self.generate_answer_key(book.puzzles())?);
        
        latex.push_str("\\end{document}\n");
        
        Ok(latex)
    }

    fn generate_preamble(&self, config: &crate::book::BookConfig) -> String {
        let (page_width, page_height, margins) = self.get_kdp_dimensions(config);
        
        format!(
            r"\documentclass[11pt,twoside,openright]{{book}}

% KDP-compliant page setup
\usepackage[paperwidth={:.3}in,paperheight={:.3}in,
            top={:.3}in,bottom={:.3}in,
            inner={:.3}in,outer={:.3}in]{{geometry}}

\usepackage{{tikz}}
\usepackage{{enumitem}}
\usepackage{{amsmath}}
\usepackage{{graphicx}}
\usepackage{{svg}}
\usepackage[T1]{{fontenc}}
\usepackage{{lmodern}}
\usepackage{{xcolor}}
\usepackage{{fancyhdr}}

% Page headers (page numbers only)
\pagestyle{{fancy}}
\fancyhf{{}}
\fancyhead[LE,RO]{{\thepage}}
\renewcommand{{\headrulewidth}}{{0pt}}
\setlength{{\headsep}}{{0.4in}}

% Drop caps for introduction
\usepackage{{lettrine}}
\renewcommand{{\LettrineTextFont}}{{\scshape}}
\setlength{{\DefaultNindent}}{{0pt}}

% Chapter styling (no chapter numbers, cleaner look)
\usepackage{{titlesec}}
\titleformat{{\chapter}}[display]
  {{\normalfont\huge\bfseries}}{{}}{{0pt}}{{\Huge}}
\titlespacing*{{\chapter}}{{0pt}}{{0pt}}{{20pt}}

\setlength{{\parindent}}{{0pt}}

% Tighter list spacing
\setlist[enumerate]{{itemsep=0.2em,parsep=0pt,topsep=0.2em}}

",
            page_width, page_height,
            margins.top, margins.bottom, margins.inner, margins.outer
        )
    }

    fn get_kdp_dimensions(&self, config: &crate::book::BookConfig) -> (f32, f32, Margins) {
        let trim = &config.trim_size;
        
        match config.kdp_format {
            KdpFormat::Paperback => {
                // No bleed for text-only puzzle books
                // Margins based on KDP requirements (assuming 100-400 pages)
                let margins = Margins {
                    top: 0.75,
                    bottom: 0.75,
                    inner: 0.625,  // Gutter for binding
                    outer: 0.5,
                };
                (trim.width, trim.height, margins)
            }
            KdpFormat::Ebook => {
                // Ebook: simpler margins
                let margins = Margins {
                    top: 0.5,
                    bottom: 0.5,
                    inner: 0.5,
                    outer: 0.5,
                };
                (trim.width, trim.height, margins)
            }
        }
    }

    fn generate_kdp_title_page(&self, config: &crate::book::BookConfig) -> Result<String> {
        let mut latex = String::new();
        
        latex.push_str("\\thispagestyle{empty}\n");
        latex.push_str("\\begin{center}\n");
        latex.push_str("\\vspace*{2cm}\n\n");
        
        // Title
        latex.push_str(&format!(
            "{{\\Huge\\bfseries {}}}\n\n",
            escape_latex(&config.title)
        ));
        
        latex.push_str("\\vspace{0.8cm}\n\n");
        
        // Subtitle
        if let Some(ref subtitle) = config.subtitle {
            latex.push_str(&format!(
                "{{\\LARGE {}}}\n\n",
                escape_latex(subtitle)
            ));
            latex.push_str("\\vspace{1cm}\n\n");
        }
        
        // Description
        if let Some(ref desc) = config.description {
            latex.push_str(&format!(
                "{{\\Large\\textit{{{}}}}}\n\n",
                escape_latex(desc)
            ));
            latex.push_str("\\vspace{1.5cm}\n\n");
        }
        
        latex.push_str("\\vfill\n\n");
        
        // Author
        if let Some(ref author) = config.author {
            latex.push_str(&format!(
                "{{\\LARGE {}}}\n\n",
                escape_latex(author)
            ));
        }
        
        latex.push_str("\\vspace{2cm}\n\n");
        latex.push_str("\\end{center}\n");
        latex.push_str("\\clearpage\n\n");
        
        Ok(latex)
    }

    fn generate_copyright_page(&self, config: &crate::book::BookConfig) -> String {
        let mut latex = String::new();
        
        latex.push_str("\\thispagestyle{empty}\n");
        latex.push_str("\\vspace*{\\fill}\n\n");
        latex.push_str("\\begin{center}\n");
        latex.push_str("\\normalsize\n\n");
        
        // Copyright notice
        if let Some(ref year) = config.copyright_year {
            latex.push_str(&format!(
                "Copyright \\copyright{} {}\n\n",
                year,
                config.author.as_deref().unwrap_or("")
            ));
        }
        
        latex.push_str("\\vspace{1.5cm}\n\n");
        
        latex.push_str("All rights reserved.\n\n");
        
        latex.push_str("\\vspace{0.8cm}\n\n");
        
        latex.push_str("\\begin{minipage}{0.8\\textwidth}\n");
        latex.push_str("\\centering\n");
        latex.push_str("No part of this publication may be reproduced, distributed, or transmitted in any form or by any means, without the prior written permission of the publisher.\n");
        latex.push_str("\\end{minipage}\n\n");
        
        latex.push_str("\\vspace{1cm}\n\n");
        
        // Edition
        if let Some(ref edition) = config.edition {
            latex.push_str(&format!("{}\n\n", escape_latex(edition)));
            latex.push_str("\\vspace{0.5cm}\n\n");
        }
        
        // ISBN
        if let Some(ref isbn) = config.isbn {
            latex.push_str(&format!("ISBN: {}\n\n", escape_latex(isbn)));
            latex.push_str("\\vspace{0.5cm}\n\n");
        }
        
        // Publisher
        if let Some(ref publisher) = config.publisher {
            latex.push_str(&format!(
                "Published by {}\n\n",
                escape_latex(publisher)
            ));
        }
        
        latex.push_str("\\end{center}\n");
        latex.push_str("\\vspace*{\\fill}\n");
        latex.push_str("\\clearpage\n\n");
        
        latex
    }

    fn generate_toc(&self, puzzle_count: usize) -> String {
        let mut latex = String::new();
        
        latex.push_str("\\thispagestyle{empty}\n");
        latex.push_str("\\begin{center}\n");
        latex.push_str("{\\Large\\bfseries Contents}\n\n");
        latex.push_str("\\vspace{1cm}\n\n");
        latex.push_str("\\end{center}\n\n");
        
        latex.push_str("\\begin{flushleft}\n");
        latex.push_str("Introduction \\dotfill ~1\n\n");
        latex.push_str(&format!("Puzzles (1--{}) \\dotfill ~2\n\n", puzzle_count));
        latex.push_str("Answer Key \\dotfill ~\\pageref{answerkey}\n\n");
        latex.push_str("\\end{flushleft}\n");
        latex.push_str("\\clearpage\n\n");
        
        latex
    }

    fn generate_introduction(&self, config: &crate::book::BookConfig) -> String {
        let mut latex = String::new();
        
        latex.push_str("\\chapter*{Introduction}\n\n");
        
        // Set paragraph indentation for intro only
        latex.push_str("\\setlength{\\parindent}{1.5em}\n");
        latex.push_str("\\setlength{\\parskip}{0.8em}\n\n");
        
        // First paragraph with drop cap
        latex.push_str("\\lettrine[lines=3,lhang=0.1,loversize=0.15]{C}{rossword} puzzles have captivated minds for over a century, beginning with Arthur Wynne's \\textit{Word-Cross} puzzle published in the \\textit{New York World} on December 21, 1913. What started as a simple diamond-shaped grid has evolved into one of the world's most beloved pastimes, challenging millions of solvers daily.\n\n");
        
        latex.push_str("The beauty of a well-crafted crossword lies in the delicate balance between challenge and satisfaction. Each puzzle is a carefully constructed lattice of interlocking words, where every letter serves double duty, connecting both across and down entries. The best puzzles reward both knowledge and wordplay, offering that satisfying ``aha!'' moment when a difficult clue finally clicks.\n\n");
        
        latex.push_str("This collection is designed to provide hours of engaging entertainment. Whether you're a seasoned cruciverbalist or a curious beginner, these puzzles offer a perfect blend of vocabulary, general knowledge, and lateral thinking.\n\n");
        
        latex.push_str("Each puzzle is printed with the grid on the right page and clues on the left, allowing you to see both simultaneously as you solve. Take your time, work in pencil, and remember: every puzzle has a solution, and the journey to finding it is half the fun.\n\n");
        
        latex.push_str("\\vspace{1.5cm}\n\n");
        
        // Signature
        latex.push_str("\\noindent Happy solving!\n\n");
        
        if let Some(ref author) = config.author {
            latex.push_str("\\vspace{0.8cm}\n\n");
            latex.push_str("\\hfill\\textit{");
            latex.push_str(&escape_latex(author));
            latex.push_str("}\n\n");
        }
        
        // Reset paragraph settings
        latex.push_str("\\setlength{\\parindent}{0pt}\n");
        latex.push_str("\\setlength{\\parskip}{0pt}\n\n");
        
        // Add clearpage to go to next page (page 2)
        latex.push_str("\\clearpage\n\n");
        
        latex
    }

    fn generate_puzzle_spread(&self, puzzle: &CrosswordPuzzle, number: usize) -> Result<String> {
        let mut latex = String::new();
        
        // LEFT PAGE - Clues (Across + Down)
        latex.push_str(&format!("\\label{{puzzle:{}}}\n", number));
        
        // Add section title without forcing page break
        latex.push_str(&format!("{{\\Large\\bfseries Puzzle {}}}\\\\[1cm]\n\n", number));
        latex.push_str("\\addcontentsline{toc}{chapter}{Puzzle ");
        latex.push_str(&number.to_string());
        latex.push_str("}\n\n");
        
        latex.push_str("\\thispagestyle{fancy}\n\n");
        
        // Top-aligned minipages for clues
        latex.push_str("\\noindent\\begin{minipage}[t]{0.48\\textwidth}\n");
        latex.push_str("\\subsection*{Across}\n");
        latex.push_str("\\raggedright\n");
        latex.push_str("\\begin{enumerate}\n");
        for clue in &puzzle.across_clues {
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
        for clue in &puzzle.down_clues {
            latex.push_str(&format!(
                "\\setcounter{{enumi}}{{{}}} \\item {}\n",
                clue.number - 1,
                escape_latex(&clue.clue)
            ));
        }
        latex.push_str("\\end{enumerate}\n");
        latex.push_str("\\end{minipage}\n");
        
        // Go to next page for grid
        latex.push_str("\\clearpage\n\n");
        
        // RIGHT PAGE - Grid only
        latex.push_str("\\thispagestyle{fancy}\n\n");
        
        // Center grid vertically on page
        latex.push_str("\\vspace*{\\fill}\n");
        latex.push_str("\\begin{center}\n");
        latex.push_str(&self.generate_grid(&puzzle.grid)?);
        latex.push_str("\\end{center}\n");
        latex.push_str("\\vspace*{\\fill}\n");
        
        // Go to next page for next puzzle
        latex.push_str("\\clearpage\n\n");
        
        Ok(latex)
    }

    fn generate_grid(&self, grid: &[Vec<Option<char>>]) -> Result<String> {
        let size = grid.len();
        let mut latex = String::new();
        
        // Fixed 70% width for all puzzles to ensure they fit
        let width_ratio = 0.95;
        
        latex.push_str(&format!(
            "\\begin{{tikzpicture}}[x={{{}\\textwidth/{}}},y={{{}\\textwidth/{}}}]\n",
            width_ratio, size, width_ratio, size
        ));
        
        // Number tracking
        let mut numbers = vec![vec![None; size]; size];
        let mut next_number = 1;
        
        for row in 0..size {
            for col in 0..size {
                if grid[row][col].is_some() {
                    let starts_across = col == 0 || grid[row][col - 1].is_none();
                    let has_across = col < size - 1 && grid[row][col + 1].is_some();
                    let starts_down = row == 0 || grid[row - 1][col].is_none();
                    let has_down = row < size - 1 && grid[row + 1][col].is_some();
                    
                    if (starts_across && has_across) || (starts_down && has_down) {
                        numbers[row][col] = Some(next_number);
                        next_number += 1;
                    }
                }
            }
        }
        
        // Draw cells with thinner lines for larger grids
        let stroke_width = if size > 14 { "0.5" } else { "1" };
        
        for row in 0..size {
            for col in 0..size {
                let x = col;
                let y = size - 1 - row;
                
                if grid[row][col].is_some() {
                    latex.push_str(&format!(
                        "\\draw[line width={}pt] ({},{}) rectangle ({},{});\n",
                        stroke_width, x, y, x + 1, y + 1
                    ));
                    
                    if let Some(num) = numbers[row][col] {
                        // Smaller numbers for larger grids
                        let font_size = if size > 14 { "\\tiny" } else { "\\scriptsize" };
                        latex.push_str(&format!(
                            "\\node[anchor=north west,font={},inner sep=0.02] at ({},{}) {{{}}};\n",
                            font_size, x as f32 + 0.05, y as f32 + 0.95, num
                        ));
                    }
                } else {
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

    fn generate_answer_key(&self, puzzles: &[CrosswordPuzzle]) -> Result<String> {
        let mut latex = String::new();
        
        latex.push_str("\\label{answerkey}\n\n");
        
        // 4 puzzles per page, 2x2 grid
        for (page_idx, chunk) in puzzles.chunks(4).enumerate() {
            if page_idx > 0 {
                latex.push_str("\\clearpage\n\n");
            }
            
            for (chunk_idx, puzzle) in chunk.iter().enumerate() {
                let puzzle_num = page_idx * 4 + chunk_idx + 1;
                
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
                
                if chunk_idx % 2 == 1 && chunk_idx < chunk.len() - 1 {
                    latex.push_str("\n\n\\vspace{1cm}\n\n");
                }
            }
            
            if chunk.len() % 2 == 1 {
                latex.push_str("\n\n\\vspace{1cm}\n\n");
            }
        }
        
        Ok(latex)
    }

    fn generate_answer_grid(&self, grid: &[Vec<Option<char>>]) -> Result<String> {
        let size = grid.len();
        let mut latex = String::new();
        
        let scale_factor = 0.85;
        
        latex.push_str(&format!(
            "\\begin{{tikzpicture}}[x={{{}\\linewidth/{}}},y={{{}\\linewidth/{}}}]\n",
            scale_factor, size, scale_factor, size
        ));
        
        for row in 0..size {
            for col in 0..size {
                let x = col;
                let y = size - 1 - row;
                
                if let Some(letter) = grid[row][col] {
                    latex.push_str(&format!(
                        "\\draw ({},{}) rectangle ({},{});\n",
                        x, y, x + 1, y + 1
                    ));
                    latex.push_str(&format!(
                        "\\node[font=\\footnotesize] at ({},{}) {{{}}};\n",
                        x as f32 + 0.5, y as f32 + 0.5, letter
                    ));
                } else {
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
}

struct Margins {
    top: f32,
    bottom: f32,
    inner: f32,
    outer: f32,
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
        assert_eq!(escape_latex("Test & Co."), "Test \\&Co.");
        assert_eq!(escape_latex("$100"), "\\$100");
        assert_eq!(escape_latex("50%"), "50\\%");
        assert_eq!(escape_latex("C++ #include"), "C++ \\#include");
    }
}
