use clap::Parser;
use crossword_core::{Dictionary, solve_with_iterations, CrosswordPuzzle};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::path::PathBuf;
use anyhow::{Result, Context};
use rand::seq::SliceRandom;

mod latex;
mod book;
mod cover;

use latex::LatexGenerator;
use book::{BookConfig, CrosswordBook};
use cover::CoverGenerator;

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

    /// Cover subtitle (replaces "PUZZLES" in template)
    #[arg(long)]
    subtitle: Option<String>,

    /// Random seed for reproducibility
    #[arg(long)]
    seed: Option<u64>,

    /// Automatically compile PDF with pdflatex
    #[arg(long)]
    compile: bool,

    /// Number of parallel threads (default: number of CPU cores)
    #[arg(short = 'j', long)]
    jobs: Option<usize>,

    /// Path to word allowlist file (one word per line, filters dictionary)
    #[arg(long)]
    allowlist: Option<PathBuf>,

    /// KDP format: paperback or ebook (default: paperback)
    #[arg(long, default_value = "paperback")]
    kdp_format: String,

    /// Trim size for paperback (default: 8x10, options: 5x8, 5.5x8.5, 6x9, 7x10, 8x10)
    #[arg(long, default_value = "8x10")]
    trim_size: String,

    /// Path to cover template SVG (uses kdp-format to determine type)
    #[arg(long)]
    cover_template: Option<PathBuf>,

    /// Generate cover file (requires cover-template)
    #[arg(long)]
    generate_cover: bool,

    /// Use color interior for spine width calculation (affects cover)
    #[arg(long)]
    color_interior: bool,
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
    let dict = if let Some(ref allowlist_path) = args.allowlist {
        let allowlist_text = fs::read_to_string(allowlist_path)
            .context("Failed to read allowlist file")?;
        println!("Using custom allowlist: {}", allowlist_path.display());
        Dictionary::with_allowlist(Some(&allowlist_text))
    } else {
        // Use default embedded allowlist
        let default_allowlist = include_str!("../../wasm/clean_allowlist.txt");
        println!("Using default allowlist");
        Dictionary::with_allowlist(Some(default_allowlist))
    };
    let stats = dict.stats();
    println!("Dictionary loaded: {} words (filtered)", stats.word_count);

    let mut config = BookConfig::new(args.title.clone(), args.size);
    config.subtitle = args.subtitle.clone();
    config.author = args.author;
    config.publisher = args.publisher;
    config.edition = args.edition;
    config.isbn = args.isbn;
    config.copyright_year = args.copyright;
    config.description = args.description;
    
    // Set KDP format
    config.kdp_format = match args.kdp_format.to_lowercase().as_str() {
        "ebook" => book::KdpFormat::Ebook,
        _ => book::KdpFormat::Paperback,
    };
    
    // Set trim size
    config.trim_size = book::TrimSize::from_string(&args.trim_size)?;
    
    // Clone values we'll need later for cover generation
    let title_for_cover = config.title.clone();
    let author_for_cover = config.author.clone();
    let subtitle_for_cover = args.subtitle.clone();
    let trim_size_for_cover = config.trim_size.clone();
    let kdp_format_for_cover = config.kdp_format.clone();

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

    // Generate cover if requested
    if args.generate_cover {
        println!("\nGenerating cover...");
        
        if let Some(template) = args.cover_template.as_ref() {
            let is_paperback = matches!(kdp_format_for_cover, book::KdpFormat::Paperback);
            
            let cover_gen = CoverGenerator::new(
                book.puzzle_count() + 6,  // Add front matter pages
                trim_size_for_cover.width,
                trim_size_for_cover.height,
            );
            
            let cover_svg = if is_paperback {
                cover_gen.generate_paperback_cover(
                    &template.to_string_lossy(),
                    &title_for_cover,
                    subtitle_for_cover.as_deref().unwrap_or("PUZZLES"),
                    author_for_cover.as_deref().unwrap_or(""),
                    args.color_interior,
                )?
            } else {
                cover_gen.generate_ebook_cover(
                    &template.to_string_lossy(),
                    &title_for_cover,
                    subtitle_for_cover.as_deref().unwrap_or("PUZZLES"),
                    author_for_cover.as_deref().unwrap_or(""),
                )?
            };
            
            let cover_path = args.output.with_extension("cover.svg");
            fs::write(&cover_path, cover_svg)?;
            println!("✅ Cover: {}", cover_path.display());
        } else {
            eprintln!("⚠️  No cover template provided. Use --cover-template");
        }
    }

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
    let _output = Command::new("pdflatex")
        .arg("-interaction=nonstopmode")
        .arg(latex_path)
        .output()
        .context("Failed to run pdflatex")?;
    
    // Second pass for references
    let _ = Command::new("pdflatex")
        .arg("-interaction=nonstopmode")
        .arg(latex_path)
        .output();
    
    // Check if PDF was actually created (even if pdflatex had warnings)
    let pdf_path = latex_path.with_extension("pdf");
    if pdf_path.exists() {
        println!("✅ PDF: {}", pdf_path.display());
        Ok(())
    } else {
        eprintln!("\n❌ pdflatex failed - no PDF created");
        eprintln!("\nBasicTeX often has package issues. Install full MacTeX:");
        eprintln!("  brew uninstall --cask basictex");
        eprintln!("  brew install --cask mactex");
        eprintln!("\nSee: {}", latex_path.with_extension("log").display());
        anyhow::bail!("Compilation failed")
    }
}
