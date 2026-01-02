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

    /// Author name
    #[arg(short, long)]
    author: Option<String>,

    /// Publisher name
    #[arg(short, long)]
    publisher: Option<String>,

    /// Edition (e.g., "First Edition", "Volume 1")
    #[arg(short, long)]
    edition: Option<String>,

    /// ISBN number
    #[arg(long)]
    isbn: Option<String>,

    /// Copyright year
    #[arg(long)]
    copyright: Option<String>,

    /// Book description for title page
    #[arg(short, long)]
    description: Option<String>,

    /// Path to cover SVG file
    #[arg(long)]
    cover_svg: Option<PathBuf>,

    /// Path to title page SVG/decoration file
    #[arg(long)]
    title_svg: Option<PathBuf>,

    /// Random seed for reproducibility
    #[arg(long)]
    seed: Option<u64>,

    /// Automatically compile PDF with pdflatex
    #[arg(long)]
    compile: bool,

    /// Number of parallel threads (default: number of CPU cores)
    #[arg(short = 'j', long)]
    jobs: Option<usize>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Set number of rayon threads if specified
    if let Some(jobs) = args.jobs {
        rayon::ThreadPoolBuilder::new()
            .num_threads(jobs)
            .build_global()
            .context("Failed to set thread pool size")?;
        println!("Using {} parallel threads", jobs);
    } else {
        println!("Using {} parallel threads (CPU cores)", rayon::current_num_threads());
    }

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

    let mut config = BookConfig::new(args.title.clone(), args.size);
    config.author = args.author;
    config.publisher = args.publisher;
    config.edition = args.edition;
    config.isbn = args.isbn;
    config.copyright_year = args.copyright;
    config.description = args.description;
    
    // Read SVG files if provided
    config.cover_svg_path = args.cover_svg.as_ref()
        .map(|p| p.to_string_lossy().to_string());
    config.title_svg_path = args.title_svg.as_ref()
        .map(|p| p.to_string_lossy().to_string());

    let mut book = CrosswordBook::new(config);

    println!("\nGenerating {} puzzles of size {}x{} in parallel...", args.count, args.size, args.size);
    let pb = ProgressBar::new(args.count as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("=>-"),
    );

    // Generate puzzles in parallel
    use rayon::prelude::*;
    let puzzles: Vec<_> = (0..args.count)
        .into_par_iter()
        .filter_map(|i| {
            match generate_crossword(&dict, args.size) {
                Ok(puzzle) => {
                    pb.inc(1);
                    Some(puzzle)
                }
                Err(e) => {
                    eprintln!("\nWarning: Failed to generate puzzle {}: {}", i + 1, e);
                    pb.inc(1);
                    None
                }
            }
        })
        .collect();

    pb.finish_with_message(format!("Complete! {} successful, {} failed", 
                                   puzzles.len(), 
                                   args.count - puzzles.len()));

    if puzzles.is_empty() {
        anyhow::bail!("No puzzles were generated successfully");
    }

    // Add all puzzles to book
    for puzzle in puzzles {
        book.add_puzzle(puzzle);
    }

    println!("\nGenerating LaTeX document...");
    let latex_gen = LatexGenerator::new();
    let latex_content = latex_gen.generate_document(&book)
        .context("Failed to generate LaTeX document")?;

    fs::write(&args.output, latex_content)
        .context("Failed to write output file")?;

    println!("\n✅ LaTeX: {}", args.output.display());

    if args.compile {
        match compile_pdf(&args.output) {
            Ok(_) => println!("🎉 Done!"),
            Err(e) => {
                eprintln!("\n⚠️  PDF failed: {}", e);
                eprintln!("But .tex file created successfully");
                return Err(e);
            }
        }
    } else {
        println!("To compile: pdflatex {}", args.output.display());
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
    
    // Check if pdflatex is installed
    let check = Command::new("which")
        .arg("pdflatex")
        .output();
    
    if check.is_err() || !check.unwrap().status.success() {
        eprintln!("\n❌ pdflatex not found");
        eprintln!("\nInstall MacTeX:");
        eprintln!("  brew install --cask mactex");
        eprintln!("\nOr generate .tex only (remove --compile flag)");
        anyhow::bail!("pdflatex not installed");
    }
    
    println!("Running pdflatex...");
    let output = Command::new("pdflatex")
        .arg("-interaction=nonstopmode")
        .arg(latex_path)
        .output()
        .context("Failed to run pdflatex")?;
    
    if !output.status.success() {
        eprintln!("\n❌ pdflatex failed");
        eprintln!("\nBasicTeX often has package issues. Install full MacTeX:");
        eprintln!("  brew uninstall --cask basictex");
        eprintln!("  brew install --cask mactex");
        eprintln!("\nOr see: {}", latex_path.with_extension("log").display());
        anyhow::bail!("Compilation failed");
    }
    
    // Second pass
    let _ = Command::new("pdflatex")
        .arg("-interaction=nonstopmode")
        .arg(latex_path)
        .output();
    
    let pdf_path = latex_path.with_extension("pdf");
    if pdf_path.exists() {
        println!("✅ PDF: {}", pdf_path.display());
    }
    
    Ok(())
}
