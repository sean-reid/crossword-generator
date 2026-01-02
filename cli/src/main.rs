use clap::Parser;
use crossword_core::{Dictionary, solve_with_iterations, CrosswordPuzzle};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::path::PathBuf;
use anyhow::{Result, Context};
use rand::seq::SliceRandom;

mod latex;
mod book;

use latex::LatexGenerator;
use book::{BookConfig, CrosswordBook};

#[derive(Parser, Debug)]
#[command(name = "crossword-cli")]
#[command(about = "Generate LaTeX crossword puzzle books", long_about = None)]
struct Args {
    /// Number of puzzles to generate
    #[arg(short, long, default_value = "10")]
    count: usize,

    /// Output LaTeX file path
    #[arg(short, long, default_value = "crossword_book.tex")]
    output: PathBuf,

    /// Grid size (NxN)
    #[arg(short, long, default_value = "16")]
    size: usize,

    /// Book title
    #[arg(short, long, default_value = "Crossword Puzzle Book")]
    title: String,

    /// Random seed for reproducibility
    #[arg(long)]
    seed: Option<u64>,

    /// Automatically compile PDF with pdflatex
    #[arg(long)]
    compile: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Set random seed if provided
    if let Some(seed) = args.seed {
        use rand::SeedableRng;
        rand::rngs::StdRng::seed_from_u64(seed);
        println!("Using random seed: {}", seed);
    }

    println!("Initializing dictionary...");
    let dict = Dictionary::new();
    let stats = dict.stats();
    println!("Dictionary loaded: {} words", stats.word_count);

    let config = BookConfig {
        title: args.title.clone(),
        grid_size: args.size,
        puzzles_per_page: 1,
    };

    let mut book = CrosswordBook::new(config);

    println!("\nGenerating {} puzzles of size {}x{}...", args.count, args.size, args.size);
    let pb = ProgressBar::new(args.count as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("=>-"),
    );

    let mut successful = 0;
    let mut failed = 0;

    for i in 0..args.count {
        pb.set_message(format!("Puzzle {}/{}", i + 1, args.count));
        
        match generate_crossword(&dict, args.size) {
            Ok(puzzle) => {
                book.add_puzzle(puzzle);
                successful += 1;
            }
            Err(e) => {
                eprintln!("\nWarning: Failed to generate puzzle {}: {}", i + 1, e);
                failed += 1;
            }
        }
        
        pb.inc(1);
    }

    pb.finish_with_message(format!("Complete! {} successful, {} failed", successful, failed));

    if successful == 0 {
        anyhow::bail!("No puzzles were generated successfully");
    }

    println!("\nGenerating LaTeX document...");
    let latex_gen = LatexGenerator::new();
    let latex_content = latex_gen.generate_document(&book)
        .context("Failed to generate LaTeX document")?;

    fs::write(&args.output, latex_content)
        .context("Failed to write output file")?;

    println!("LaTeX document written to: {}", args.output.display());

    if args.compile {
        println!("\nCompiling PDF with pdflatex...");
        compile_pdf(&args.output)?;
    } else {
        println!("\nTo compile to PDF, run:");
        println!("  pdflatex {}", args.output.display());
        println!("  pdflatex {}  # Run twice for references", args.output.display());
    }

    Ok(())
}

fn generate_crossword(dict: &Dictionary, size: usize) -> Result<CrosswordPuzzle> {
    let all_words = dict.get_words();
    
    // Filter suitable words
    let suitable: Vec<String> = all_words.iter()
        .filter(|w| w.len() >= 3 && w.len() <= size)
        .cloned()
        .collect();
    
    // Group by length
    let mut by_length: std::collections::HashMap<usize, Vec<String>> = std::collections::HashMap::new();
    for word in suitable {
        by_length.entry(word.len()).or_insert_with(Vec::new).push(word);
    }
    
    // Determine max words based on size
    let max_words = match size {
        s if s <= 8 => 80,
        s if s <= 10 => 120,
        s if s <= 12 => 150,
        s if s <= 15 => 130,
        s if s <= 20 => 100,
        _ => 100,
    };
    
    let mut words = Vec::new();
    
    // Select words with length distribution
    for len in 3..=size.min(15) {
        if let Some(len_words) = by_length.get_mut(&len) {
            len_words.shuffle(&mut rand::thread_rng());
            
            let proportion = if len <= 5 {
                0.70
            } else if len <= 8 {
                0.25
            } else {
                0.05
            };
            
            let count = ((max_words as f32 * proportion) / 4.0) as usize;
            words.extend(len_words.iter().take(count.max(8)).cloned());
            
            if words.len() >= max_words {
                break;
            }
        }
    }
    
    words.truncate(max_words);
    
    // Solve the crossword
    let (placements, elapsed_ms, _num_vars, _num_clauses) = solve_with_iterations(&words, size)
        .map_err(|e| anyhow::anyhow!("Solver failed: {}", e))?;
    
    // Create puzzle
    let puzzle = CrosswordPuzzle::from_placements(
        &placements,
        size,
        |word| dict.get_clue(word),
        elapsed_ms,
    );
    
    Ok(puzzle)
}

fn compile_pdf(latex_path: &PathBuf) -> Result<()> {
    use std::process::Command;
    
    let output = Command::new("pdflatex")
        .arg("-interaction=nonstopmode")
        .arg(latex_path)
        .output()
        .context("Failed to run pdflatex - is it installed?")?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("pdflatex failed: {}", stderr);
    }
    
    // Run twice for references
    let output = Command::new("pdflatex")
        .arg("-interaction=nonstopmode")
        .arg(latex_path)
        .output()
        .context("Failed to run pdflatex second time")?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("pdflatex failed on second pass: {}", stderr);
    }
    
    let pdf_path = latex_path.with_extension("pdf");
    println!("PDF generated: {}", pdf_path.display());
    
    Ok(())
}
