// Core modules - always compiled
mod dictionary;
mod encoder;
mod solver;
mod solution;

#[macro_use]
mod debug;

// Re-export for CLI use
pub use dictionary::Dictionary;
pub use encoder::CrosswordEncoder;
pub use solver::{solve_with_iterations, solve_encoded};
pub use solution::{Placement, Clue, CrosswordPuzzle, CrosswordMetadata};

// WASM-specific code - only when wasm feature enabled
#[cfg(feature = "wasm")]
mod wasm_interface {
    use super::*;
    use wasm_bindgen::prelude::*;
    use std::sync::Mutex;
    use rand::seq::SliceRandom;
    use serde::Serialize;

    #[derive(Serialize)]
    struct ProblemEstimate {
        word_count: usize,
        estimated_vars: usize,
        estimated_clauses: usize,
        encoding_ms: u32,
        solving_ms: u32,
        total_ms: u32,
    }

    #[derive(Serialize)]
    struct EncodingResult {
        num_vars: usize,
        num_clauses: usize,
        encoding_time_ms: u32,
        estimated_solve_ms: u32,
    }

    static DICTIONARY: Mutex<Option<Dictionary>> = Mutex::new(None);
    static ENCODER_STATE: Mutex<Option<(CrosswordEncoder, Vec<String>, usize)>> = Mutex::new(None);

    #[wasm_bindgen(start)]
    pub fn main() {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    }

    #[wasm_bindgen]
    pub fn initialize() -> Result<JsValue, JsValue> {
        use crate::debug_log;
        
        debug_log!("[WASM] Initializing dictionary...");
        
        let dict = Dictionary::new();
        let stats = dict.stats();
        
        debug_log!("[WASM] Dictionary loaded: {} words", stats.word_count);
        
        let mut dict_lock = DICTIONARY.lock().unwrap();
        *dict_lock = Some(dict);
        
        serde_wasm_bindgen::to_value(&stats)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    #[wasm_bindgen]
    pub fn estimate_encoding_time(size: usize, word_count: usize) -> u32 {
        // Estimate encoding time: ~0.5ms per word per grid cell (placement checks)
        let placements_estimate = word_count * size * 2; // Rough estimate of placement vars
        (placements_estimate as f32 * 0.01) as u32
    }

    #[wasm_bindgen]
    pub fn estimate_problem_size(size: usize) -> Result<JsValue, JsValue> {
        
        let dict_lock = DICTIONARY.lock()
            .map_err(|e| JsValue::from_str(&format!("Lock error: {}", e)))?;
        
        let dict = dict_lock.as_ref()
            .ok_or_else(|| JsValue::from_str("Dictionary not initialized"))?;
        
        let all_words = dict.get_words();
        
        // Calculate word count that would be used
        let suitable_count = all_words.iter()
            .filter(|w| w.len() >= 3 && w.len() <= size)
            .count();
        
        let max_words = match size {
            s if s <= 8 => 80,    // Reduced for speed
            s if s <= 10 => 120,  // Reduced
            s if s <= 12 => 150,  // Major reduction
            s if s <= 15 => 130,
            s if s <= 20 => 100,
            _ => 100,
        };
        
        let word_count = suitable_count.min(max_words);
        
        // Estimate variable count accurately
        let placement_vars = word_count * size * 2;
        
        // Grid vars: size² cells × ~25 chars
        let grid_vars = size * size * 25;
        
        // CC constraint: size² cells × max_dist steps
        let max_dist = (size + 1) * (size + 1) / 2 - 1;
        let cc_vars = size * size * (max_dist + 2); // reachability + is_filled + cc_start
        
        // At-least-k auxiliary vars: rough estimate
        let atk_vars = word_count * 10;
        
        let estimated_vars = placement_vars + grid_vars + cc_vars + atk_vars;
        let estimated_clauses = estimated_vars * 12;
        
        // Estimate times - use VERY conservative estimates
        // Real data: 534k vars = 6.4s encode + 40s solve
        // Use 10x safety margin since SAT solving is highly unpredictable
        let encoding_estimate = ((estimated_vars as f32 * 0.015) as u32).max(2000);
        let solving_estimate = ((estimated_vars as f32 * 0.075) as u32).max(5000);
        
        let result = ProblemEstimate {
            word_count,
            estimated_vars,
            estimated_clauses,
            encoding_ms: encoding_estimate.max(200),
            solving_ms: solving_estimate.max(1000),
            total_ms: (encoding_estimate + solving_estimate).max(1500),
        };
        
        serde_wasm_bindgen::to_value(&result)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    #[wasm_bindgen]
    pub fn encode_problem(size: usize) -> Result<JsValue, JsValue> {
        use crate::debug_log;
        use web_time::Instant;
        
        debug_log!("[WASM] encode_problem: size={}", size);
        
        let dict_lock = DICTIONARY.lock()
            .map_err(|e| JsValue::from_str(&format!("Lock error: {}", e)))?;
        
        let dict = dict_lock.as_ref()
            .ok_or_else(|| JsValue::from_str("Dictionary not initialized"))?;
        
        let all_words = dict.get_words();
        
        // Select words
        let suitable: Vec<String> = all_words.iter()
            .filter(|w| w.len() >= 3 && w.len() <= size)
            .cloned()
            .collect();
        
        let mut by_length: std::collections::HashMap<usize, Vec<String>> = std::collections::HashMap::new();
        for word in suitable {
            by_length.entry(word.len()).or_insert_with(Vec::new).push(word);
        }
        
        let max_words = match size {
            s if s <= 8 => 80,    // Reduced for speed
            s if s <= 10 => 120,  // Reduced
            s if s <= 12 => 150,  // Major reduction
            s if s <= 15 => 130,
            s if s <= 20 => 100,
            _ => 100,
        };
        
        let mut words = Vec::new();
        
        for len in 3..=size.min(15) {
            if let Some(len_words) = by_length.get_mut(&len) {
                len_words.shuffle(&mut rand::thread_rng());
                
                let proportion = if len <= 5 { 0.70 } else if len <= 8 { 0.25 } else { 0.05 };
                let count = ((max_words as f32 * proportion) / 4.0) as usize;
                words.extend(len_words.iter().take(count.max(8)).cloned());
                
                if words.len() >= max_words {
                    break;
                }
            }
        }
        
        words.truncate(max_words);
        
        debug_log!("[WASM] Encoding {} words", words.len());
        
        let start = Instant::now();
        let target_quality = (size * size * 4 / 10).max(20);
        
        let mut encoder = CrosswordEncoder::new(size);
        let (num_vars, num_clauses) = encoder.encode(&words, size, target_quality)
            .map_err(|e| JsValue::from_str(&e))?;
        
        let encoding_time = start.elapsed().as_millis() as u32;
        
        // Estimate solve time
        let estimated_solve_ms = ((num_vars as f32 * 0.085) as u32).max(3000);
        
        debug_log!("[WASM] Encoded: {} vars, {} clauses in {}ms", num_vars, num_clauses, encoding_time);
        debug_log!("[WASM] Estimated solve: {}ms", estimated_solve_ms);
        
        // Store encoder state for solve phase
        let mut state_lock = ENCODER_STATE.lock()
            .map_err(|e| JsValue::from_str(&format!("Lock error: {}", e)))?;
        *state_lock = Some((encoder, words, size));
        
        let result = EncodingResult {
            num_vars,
            num_clauses,
            encoding_time_ms: encoding_time,
            estimated_solve_ms,
        };
        
        serde_wasm_bindgen::to_value(&result)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    #[wasm_bindgen]
    pub fn solve_problem() -> Result<JsValue, JsValue> {
        use crate::debug_log;
        
        debug_log!("[WASM] solve_problem");
        
        let mut state_lock = ENCODER_STATE.lock()
            .map_err(|e| JsValue::from_str(&format!("Lock error: {}", e)))?;
        
        let (encoder, words, size) = state_lock.take()
            .ok_or_else(|| JsValue::from_str("No encoded problem - call encode_problem first"))?;
        
        let dict_lock = DICTIONARY.lock()
            .map_err(|e| JsValue::from_str(&format!("Lock error: {}", e)))?;
        
        let dict = dict_lock.as_ref()
            .ok_or_else(|| JsValue::from_str("Dictionary not initialized"))?;
        
        let (placements, elapsed_ms) = solve_encoded(encoder)
            .map_err(|e| JsValue::from_str(&e))?;
        
        debug_log!("[WASM] Solved: {} placements in {}ms", placements.len(), elapsed_ms);
        
        let puzzle = CrosswordPuzzle::from_placements(
            &placements,
            size,
            |word| dict.get_clue(word),
            elapsed_ms,
        );
        
        debug_log!("[WASM] Puzzle: density={:.1}%, {} words", 
                   puzzle.metadata.density * 100.0, puzzle.metadata.word_count);
        
        serde_wasm_bindgen::to_value(&puzzle)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    #[wasm_bindgen]
    pub fn generate_crossword(size: usize) -> Result<JsValue, JsValue> {
        use crate::debug_log;
        
        debug_log!("[WASM] generate_crossword: size={}", size);
        
        let result = std::panic::catch_unwind(|| -> Result<CrosswordPuzzle, String> {
            let dict_lock = DICTIONARY.lock()
                .map_err(|e| format!("Lock error: {}", e))?;
            
            let dict = dict_lock.as_ref()
                .ok_or_else(|| "Dictionary not initialized".to_string())?;
            
            let all_words = dict.get_words();
            
            let suitable: Vec<String> = all_words.iter()
                .filter(|w| w.len() >= 3 && w.len() <= size)
                .cloned()
                .collect();
            
            let mut by_length: std::collections::HashMap<usize, Vec<String>> = std::collections::HashMap::new();
            for word in suitable {
                by_length.entry(word.len()).or_insert_with(Vec::new).push(word);
            }
            
            let max_words = match size {
            s if s <= 8 => 80,    // Reduced for speed
            s if s <= 10 => 120,  // Reduced
            s if s <= 12 => 150,  // Major reduction
            s if s <= 15 => 130,
            s if s <= 20 => 100,
                _ => 100,
            };
            
            let mut words = Vec::new();
            
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
            
            debug_log!("[WASM] Using {} suitable words", words.len());
            
            let (placements, elapsed_ms, num_vars, num_clauses) = solver::solve_with_iterations(&words, size)?;
            
            debug_log!("[WASM] Solved: {} placements in {}ms ({} vars, {} clauses)", 
                       placements.len(), elapsed_ms, num_vars, num_clauses);
            
            let puzzle = CrosswordPuzzle::from_placements(
                &placements,
                size,
                |word| dict.get_clue(word),
                elapsed_ms,
            );
            
            debug_log!("[WASM] Puzzle: density={:.1}%, {} words", 
                       puzzle.metadata.density * 100.0, puzzle.metadata.word_count);
            
            Ok(puzzle)
        });
        
        match result {
            Ok(Ok(puzzle)) => {
                serde_wasm_bindgen::to_value(&puzzle)
                    .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
            }
            Ok(Err(e)) => {
                Err(JsValue::from_str(&format!("Generation error: {}", e)))
            }
            Err(_) => {
                Err(JsValue::from_str("Panic during generation"))
            }
        }
    }
}

#[cfg(feature = "wasm")]
pub use wasm_interface::*;
