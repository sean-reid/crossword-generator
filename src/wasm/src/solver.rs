use varisat::solver::Solver;
use crate::encoder::CrosswordEncoder;
use crate::solution::Placement;
use web_time::Instant;

pub fn solve_with_iterations(
    words: &[String],
    size: usize,
) -> Result<(Vec<Placement>, u32, usize, usize), String> {
    use crate::debug_log;
    
    let start = Instant::now();
    
    // Quality target controls density
    // Quality = sum of all placed word lengths
    // Higher target = more words = higher density
    // Current: 40% target density (size² * 0.4)
    let target_quality = (size * size * 4 / 10).max(20);
    
    debug_log!("[SOLVER] Solving with quality={} (target ~40% density)", target_quality);
    
    let mut encoder = CrosswordEncoder::new(size);
    let (num_vars, num_clauses) = encoder.encode(words, size, target_quality)?;
    
    let encoding_time = start.elapsed().as_millis() as u32;
    debug_log!("[SOLVER] Encoded in {}ms: {} vars, {} clauses", encoding_time, num_vars, num_clauses);
    
    // Estimate solve time based on actual observations
    // Real data: 333k vars = 28.4s solve
    // Use 0.085ms per var (matches observed data)
    let estimated_solve_ms = ((num_vars as f32 * 0.085) as u32).max(3000);
    debug_log!("[SOLVER] Estimated solve time: {}ms", estimated_solve_ms);
    
    let mut solver = Solver::new();
    solver.add_formula(encoder.get_formula());
    
    debug_log!("[SOLVER] Starting SAT solver...");
    
    match solver.solve() {
        Ok(true) => {
            if let Some(model) = solver.model() {
                let placements = encoder.extract_placements(&model);
                let elapsed = start.elapsed().as_millis() as u32;
                
                if placements.is_empty() {
                    Err("No placements found".to_string())
                } else {
                    debug_log!("[SOLVER] Total time {}ms", elapsed);
                    Ok((placements, elapsed, num_vars, num_clauses))
                }
            } else {
                Err("No model available".to_string())
            }
        }
        Ok(false) => Err("UNSAT".to_string()),
        Err(e) => Err(format!("Solver error: {:?}", e)),
    }
}

pub fn solve_encoded(encoder: CrosswordEncoder) -> Result<(Vec<Placement>, u32), String> {
    use crate::debug_log;
    use web_time::Instant;
    
    let start = Instant::now();
    
    debug_log!("[SOLVER] Solving encoded problem...");
    
    let mut solver = Solver::new();
    solver.add_formula(encoder.get_formula());
    
    match solver.solve() {
        Ok(true) => {
            if let Some(model) = solver.model() {
                let placements = encoder.extract_placements(&model);
                let elapsed = start.elapsed().as_millis() as u32;
                
                if placements.is_empty() {
                    Err("No placements found".to_string())
                } else {
                    debug_log!("[SOLVER] Solved in {}ms", elapsed);
                    Ok((placements, elapsed))
                }
            } else {
                Err("No model available".to_string())
            }
        }
        Ok(false) => Err("UNSAT".to_string()),
        Err(e) => Err(format!("Solver error: {:?}", e)),
    }
}
