use varisat::{CnfFormula, ExtendFormula, Lit, Var};
use std::collections::HashMap;
use crate::solution::Placement;

pub struct CrosswordEncoder {
    formula: CnfFormula,
    var_counter: usize,
    placement_vars: HashMap<(String, usize, usize, bool), Var>,
    grid_vars: HashMap<(usize, usize, char), Var>,
    possible_placements: Vec<Vec<Vec<Vec<Var>>>>,
}

impl CrosswordEncoder {
    pub fn new(size: usize) -> Self {
        CrosswordEncoder {
            formula: CnfFormula::new(),
            var_counter: 1,
            placement_vars: HashMap::new(),
            grid_vars: HashMap::new(),
            possible_placements: vec![vec![vec![Vec::new(); 2]; size]; size],
        }
    }
    
    fn new_var(&mut self) -> Var {
        let v = Var::from_dimacs(self.var_counter as isize);
        self.var_counter += 1;
        v
    }
    
    pub fn encode(&mut self, words: &[String], size: usize, min_quality: usize) -> Result<(usize, usize), String> {
        use crate::debug_log;
        
        debug_log!("[ENCODER] Encoding {} words for {}x{} grid", words.len(), size, size);
        
        let chars: Vec<char> = words.iter()
            .flat_map(|w| w.chars())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        
        // Create grid variables
        for y in 0..size {
            for x in 0..size {
                for &ch in &chars {
                    let var = self.new_var();
                    self.grid_vars.insert((x, y, ch), var);
                }
            }
        }
        
        // At most one char per cell
        for y in 0..size {
            for x in 0..size {
                let cell_chars: Vec<Var> = chars.iter()
                    .filter_map(|&ch| self.grid_vars.get(&(x, y, ch)).copied())
                    .collect();
                self.at_most_one(&cell_chars);
            }
        }
        
        // Create placement variables and encode placement => grid chars
        for word in words {
            let word_chars: Vec<char> = word.chars().collect();
            let mut all_placements = Vec::new();
            
            for y in 0..size {
                for x in 0..size {
                    // Horizontal
                    if x + word.len() <= size {
                        let pvar = self.new_var();
                        self.placement_vars.insert((word.clone(), x, y, true), pvar);
                        all_placements.push(pvar);
                        self.possible_placements[y][x][0].push(pvar);
                        
                        // pvar => grid chars match
                        for (i, &ch) in word_chars.iter().enumerate() {
                            if let Some(&gvar) = self.grid_vars.get(&(x + i, y, ch)) {
                                self.formula.add_clause(&[pvar.negative(), gvar.positive()]);
                            }
                        }
                        
                        // pvar => boundaries empty
                        if x > 0 {
                            for &ch in &chars {
                                if let Some(&gvar) = self.grid_vars.get(&(x - 1, y, ch)) {
                                    self.formula.add_clause(&[pvar.negative(), gvar.negative()]);
                                }
                            }
                        }
                        if x + word.len() < size {
                            for &ch in &chars {
                                if let Some(&gvar) = self.grid_vars.get(&(x + word.len(), y, ch)) {
                                    self.formula.add_clause(&[pvar.negative(), gvar.negative()]);
                                }
                            }
                        }
                    }
                    
                    // Vertical
                    if y + word.len() <= size {
                        let pvar = self.new_var();
                        self.placement_vars.insert((word.clone(), x, y, false), pvar);
                        all_placements.push(pvar);
                        self.possible_placements[y][x][1].push(pvar);
                        
                        // pvar => grid chars match
                        for (i, &ch) in word_chars.iter().enumerate() {
                            if let Some(&gvar) = self.grid_vars.get(&(x, y + i, ch)) {
                                self.formula.add_clause(&[pvar.negative(), gvar.positive()]);
                            }
                        }
                        
                        // pvar => boundaries empty
                        if y > 0 {
                            for &ch in &chars {
                                if let Some(&gvar) = self.grid_vars.get(&(x, y - 1, ch)) {
                                    self.formula.add_clause(&[pvar.negative(), gvar.negative()]);
                                }
                            }
                        }
                        if y + word.len() < size {
                            for &ch in &chars {
                                if let Some(&gvar) = self.grid_vars.get(&(x, y + word.len(), ch)) {
                                    self.formula.add_clause(&[pvar.negative(), gvar.negative()]);
                                }
                            }
                        }
                    }
                }
            }
            
            // At most one placement per word
            self.at_most_one(&all_placements);
        }
        
        debug_log!("[ENCODER] Created {} placement vars", self.placement_vars.len());
        
        // Require at least one horizontal and one vertical word
        let horiz_placements: Vec<Var> = self.placement_vars.iter()
            .filter(|((_, _, _, h), _)| *h)
            .map(|(_, &v)| v)
            .collect();
        
        let vert_placements: Vec<Var> = self.placement_vars.iter()
            .filter(|((_, _, _, h), _)| !*h)
            .map(|(_, &v)| v)
            .collect();
        
        if !horiz_placements.is_empty() {
            let clause: Vec<Lit> = horiz_placements.iter().map(|&v| v.positive()).collect();
            self.formula.add_clause(&clause);
            debug_log!("[ENCODER] Required at least 1 horizontal word");
        }
        
        if !vert_placements.is_empty() {
            let clause: Vec<Lit> = vert_placements.iter().map(|&v| v.positive()).collect();
            self.formula.add_clause(&clause);
            debug_log!("[ENCODER] Required at least 1 vertical word");
        }
        
        // Strengthen: require at least 3 of each orientation for better connectivity
        if horiz_placements.len() >= 3 {
            self.at_least_k(&horiz_placements, 3);
        }
        if vert_placements.len() >= 3 {
            self.at_least_k(&vert_placements, 3);
        }
        
        // KEY FIX: Grid chars can ONLY be set by placements (bidirectional)
        let mut _total_bidirectional_clauses = 0;
        for y in 0..size {
            for x in 0..size {
                for &ch in &chars {
                    if let Some(&gvar) = self.grid_vars.get(&(x, y, ch)) {
                        // Collect all placements that could set this grid cell to this char
                        let mut covering_placements = Vec::new();
                        
                        for ((word, px, py, horiz), &pvar) in &self.placement_vars {
                            let word_chars: Vec<char> = word.chars().collect();
                            
                            // Check if this placement covers (x,y) with character ch
                            for (i, &wch) in word_chars.iter().enumerate() {
                                if wch == ch {
                                    let (cx, cy) = if *horiz {
                                        (px + i, *py)
                                    } else {
                                        (*px, py + i)
                                    };
                                    
                                    if cx == x && cy == y {
                                        covering_placements.push(pvar);
                                        break;
                                    }
                                }
                            }
                        }
                        
                        // gvar => at least one covering placement
                        if !covering_placements.is_empty() {
                            let mut clause = vec![gvar.negative()];
                            clause.extend(covering_placements.iter().map(|&v| v.positive()));
                            self.formula.add_clause(&clause);
                            _total_bidirectional_clauses += 1;
                        } else {
                            // No placement can set this, so it must be false
                            self.formula.add_clause(&[gvar.negative()]);
                        }
                    }
                }
            }
        }
        
        debug_log!("[ENCODER] Added {} bidirectional grid clauses", _total_bidirectional_clauses);
        
        // Sequence validation - CHECK EVERY POSITION like Python does
        for y in 0..size {
            for x in 0..size {
                if x + 1 < size {
                    let placements = self.possible_placements[y][x][0].clone();
                    self.add_sequence_constraint(x, y, true, size, &chars, &placements);
                }
                if y + 1 < size {
                    let placements = self.possible_placements[y][x][1].clone();
                    self.add_sequence_constraint(x, y, false, size, &chars, &placements);
                }
            }
        }
        
        debug_log!("[ENCODER] Added sequence validation");
        
        // Connected component constraint - returns is_filled vars
        let is_filled = self.add_connectivity_constraint(size, &chars);
        
        debug_log!("[ENCODER] Added connectivity constraint");
        
        // DENSITY constraint - require minimum percentage of cells filled
        let min_filled_cells = (size * size * 5 / 10).max(15);  // 50% minimum
        
        let filled_vars: Vec<Var> = is_filled.iter()
            .flat_map(|row| row.iter())
            .copied()
            .collect();
        
        if !filled_vars.is_empty() {
            debug_log!("[ENCODER] Adding density constraint: at least {} of {} cells filled", min_filled_cells, filled_vars.len());
            self.at_least_k(&filled_vars, min_filled_cells);
        }
        
        // Quality constraint (still useful for word selection) - require minimum total word length
        if min_quality > 0 {
            let all_placements: Vec<Var> = self.placement_vars.values().copied().collect();
            if !all_placements.is_empty() {
                // Lower min since density is directly enforced
                let min_words = (min_quality / 10).max(6);
                self.at_least_k(&all_placements, min_words);
                debug_log!("[ENCODER] Quality constraint: at least {} placements", min_words);
            }
        }
        
        debug_log!("[ENCODER] Encoding complete");
        
        let num_vars = self.var_counter - 1;
        let num_clauses = self.formula.len();
        
        Ok((num_vars, num_clauses))
    }
    
    fn add_sequence_constraint(
        &mut self,
        x: usize,
        y: usize,
        horizontal: bool,
        _size: usize,
        chars: &[char],
        placements: &[Var],
    ) {
        // Simplified since bidirectional grid constraint handles most of this
        // Just ensure: if sequence pattern exists, a placement must be there
        
        let curr_chars: Vec<Var> = chars.iter()
            .filter_map(|&ch| self.grid_vars.get(&(x, y, ch)).copied())
            .collect();
        
        let (next_x, next_y) = if horizontal { (x + 1, y) } else { (x, y + 1) };
        let next_chars: Vec<Var> = chars.iter()
            .filter_map(|&ch| self.grid_vars.get(&(next_x, next_y, ch)).copied())
            .collect();
        
        // (any curr char AND any next char AND prev empty) => at least one placement
        for &cc in &curr_chars {
            for &nc in &next_chars {
                let mut clause = vec![cc.negative(), nc.negative()];
                clause.extend(placements.iter().map(|&v| v.positive()));
                
                // Add prev empty condition
                if horizontal && x > 0 || !horizontal && y > 0 {
                    let (prev_x, prev_y) = if horizontal { (x - 1, y) } else { (x, y - 1) };
                    let prev_chars: Vec<Var> = chars.iter()
                        .filter_map(|&ch| self.grid_vars.get(&(prev_x, prev_y, ch)).copied())
                        .collect();
                    clause.extend(prev_chars.iter().map(|&v| v.positive()));
                }
                
                self.formula.add_clause(&clause);
            }
        }
    }
    
    fn at_most_one(&mut self, vars: &[Var]) {
        for i in 0..vars.len() {
            for j in (i + 1)..vars.len() {
                self.formula.add_clause(&[vars[i].negative(), vars[j].negative()]);
            }
        }
    }
    
    fn at_least_k(&mut self, vars: &[Var], k: usize) {
        use crate::debug_log;
        
        let n = vars.len();
        if k == 0 || k > n {
            return;
        }
        
        debug_log!("[ENCODER] at_least_k: k={}, n={}", k, n);
        
        if k == 1 {
            // At least one must be true
            let clause: Vec<Lit> = vars.iter().map(|&v| v.positive()).collect();
            self.formula.add_clause(&clause);
            return;
        }
        
        // Sequential counter encoding
        // aux[i][j] = "at least j of the first i variables are true"
        let mut aux: Vec<Vec<Option<Var>>> = vec![vec![None; k + 1]; n + 1];
        
        // Base case: aux[0][0] is true (0 of first 0 are true)
        let base_var = self.new_var();
        self.formula.add_clause(&[base_var.positive()]);
        aux[0][0] = Some(base_var);
        
        for i in 1..=n {
            let x = vars[i - 1];
            
            for j in 0..=k.min(i) {
                let v = self.new_var();
                aux[i][j] = Some(v);
                
                if j == 0 {
                    // aux[i][0] always true (at least 0)
                    self.formula.add_clause(&[v.positive()]);
                } else if j <= i - 1 && j - 1 < i - 1 {
                    // aux[i][j] can be true if:
                    // 1. aux[i-1][j] is true (already have j without x)
                    // 2. aux[i-1][j-1] is true AND x is true (have j-1, plus x makes j)
                    
                    if let (Some(prev_j), Some(prev_jm1)) = (aux[i-1].get(j).and_then(|&o| o), aux[i-1].get(j-1).and_then(|&o| o)) {
                        // v => (prev_j OR (prev_jm1 AND x))
                        self.formula.add_clause(&[v.negative(), prev_j.positive(), prev_jm1.positive()]);
                        self.formula.add_clause(&[v.negative(), prev_j.positive(), x.positive()]);
                        
                        // (prev_j AND NOT x) => v
                        self.formula.add_clause(&[prev_j.negative(), x.positive(), v.positive()]);
                        
                        // (prev_jm1 AND x) => v
                        self.formula.add_clause(&[prev_jm1.negative(), x.negative(), v.positive()]);
                    } else if let Some(prev_j) = aux[i-1].get(j).and_then(|&o| o) {
                        // Only prev_j path available
                        self.formula.add_clause(&[v.negative(), prev_j.positive()]);
                        self.formula.add_clause(&[prev_j.negative(), v.positive()]);
                    }
                } else if j == i {
                    // aux[i][i] = all i variables must be true
                    if let Some(prev) = aux[i-1].get(j-1).and_then(|&o| o) {
                        // v <=> (prev AND x)
                        self.formula.add_clause(&[v.negative(), prev.positive()]);
                        self.formula.add_clause(&[v.negative(), x.positive()]);
                        self.formula.add_clause(&[prev.negative(), x.negative(), v.positive()]);
                    }
                }
            }
        }
        
        // Require aux[n][k]
        if let Some(final_var) = aux[n][k] {
            self.formula.add_clause(&[final_var.positive()]);
            debug_log!("[ENCODER] Requiring aux[{}][{}] = true", n, k);
        }
    }
    
    pub fn get_formula(&self) -> &CnfFormula {
        &self.formula
    }
    
    pub fn extract_placements(&self, model: &[Lit]) -> Vec<Placement> {
        use crate::debug_log;
        use std::collections::HashSet;
        
        let model_set: HashSet<Lit> = model.iter().copied().collect();
        let mut placements = Vec::new();
        
        for ((word, x, y, horiz), &var) in &self.placement_vars {
            if model_set.contains(&var.positive()) {
                debug_log!("[ENCODER] Placed: {} at ({},{}) {}", 
                          word, x, y, if *horiz { "across" } else { "down" });
                placements.push(Placement {
                    word: word.clone(),
                    x: *x,
                    y: *y,
                    horizontal: *horiz,
                });
            }
        }
        
        // Validate: check if grid chars match placements
        debug_log!("[ENCODER] Validating grid matches placements...");
        for ((x, y, ch), &var) in &self.grid_vars {
            if model_set.contains(&var.positive()) {
                // This grid cell is set to ch - verify at least one placement covers it
                let mut covered = false;
                for p in &placements {
                    for (i, wch) in p.word.chars().enumerate() {
                        let (px, py) = if p.horizontal {
                            (p.x + i, p.y)
                        } else {
                            (p.x, p.y + i)
                        };
                        if px == *x && py == *y && wch == *ch {
                            covered = true;
                            break;
                        }
                    }
                    if covered { break; }
                }
                
                if !covered {
                    debug_log!("[ENCODER] WARNING: Grid[{}][{}]='{}' not covered by any placement!", x, y, ch);
                }
            }
        }
        
        debug_log!("[ENCODER] Extracted {} placements", placements.len());
        placements
    }
    
    fn add_connectivity_constraint(&mut self, size: usize, chars: &[char]) -> Vec<Vec<Var>> {
        use crate::debug_log;
        
        // Python lines 145-179: Connected component constraint
        // All filled cells must be reachable from a designated start cell
        // RETURNS is_filled so we can use it for density constraint
        
        let max_dist = (size + 1) * (size + 1) / 2 - 1;
        
        debug_log!("[ENCODER] Adding CC constraint with max_dist={}", max_dist);
        
        // Variables for CC start selection
        let mut cc_start_row: Vec<Var> = Vec::new();
        for _ in 0..size {
            cc_start_row.push(self.new_var());
        }
        
        let mut cc_start: Vec<Vec<Var>> = Vec::new();
        for _ in 0..size {
            let mut row = Vec::new();
            for _ in 0..size {
                row.push(self.new_var());
            }
            cc_start.push(row);
        }
        
        // Reachability variables: in_cc[y][x][i] = "cell (x,y) reaches CC start in <=i steps"
        let mut in_cc: Vec<Vec<Vec<Var>>> = Vec::new();
        for _ in 0..size {
            let mut row = Vec::new();
            for _ in 0..size {
                let mut steps = Vec::new();
                for _ in 0..=max_dist {
                    steps.push(self.new_var());
                }
                row.push(steps);
            }
            in_cc.push(row);
        }
        
        // Build "cell is filled" variables
        let mut is_filled: Vec<Vec<Var>> = Vec::new();
        for y in 0..size {
            let mut row = Vec::new();
            for x in 0..size {
                let filled_var = self.new_var();
                
                let cell_chars: Vec<Var> = chars.iter()
                    .filter_map(|&ch| self.grid_vars.get(&(x, y, ch)).copied())
                    .collect();
                
                // filled <=> at least one char
                if !cell_chars.is_empty() {
                    let mut clause = vec![filled_var.negative()];
                    clause.extend(cell_chars.iter().map(|&v| v.positive()));
                    self.formula.add_clause(&clause);
                    
                    for &cv in &cell_chars {
                        self.formula.add_clause(&[cv.negative(), filled_var.positive()]);
                    }
                } else {
                    // Can't be filled
                    self.formula.add_clause(&[filled_var.negative()]);
                }
                
                row.push(filled_var);
            }
            is_filled.push(row);
        }
        
        // CC start selection (first filled cell in reading order)
        for y in 0..size {
            // cc_start_row[y] <=> (no filled in prev rows AND at least one filled in row y)
            let mut no_prev = Vec::new();
            for py in 0..y {
                no_prev.push(cc_start_row[py].negative());
            }
            
            let mut any_in_row = Vec::new();
            for x in 0..size {
                any_in_row.push(is_filled[y][x].positive());
            }
            
            // cc_start_row[y] => no prev rows have start
            for &npr in &no_prev {
                self.formula.add_clause(&[cc_start_row[y].negative(), npr]);
            }
            
            // cc_start_row[y] => at least one in row
            if !any_in_row.is_empty() {
                let mut clause = vec![cc_start_row[y].negative()];
                clause.extend(any_in_row.iter().cloned());
                self.formula.add_clause(&clause);
            }
            
            for x in 0..size {
                // cc_start[y][x] <=> (cc_start_row[y] AND no start before x AND filled at x,y)
                
                let mut no_prev_x = Vec::new();
                for px in 0..x {
                    no_prev_x.push(cc_start[y][px].negative());
                }
                
                // cc_start[y][x] => cc_start_row[y]
                self.formula.add_clause(&[cc_start[y][x].negative(), cc_start_row[y].positive()]);
                
                // cc_start[y][x] => filled
                self.formula.add_clause(&[cc_start[y][x].negative(), is_filled[y][x].positive()]);
                
                // cc_start[y][x] => no prev in row
                for &npx in &no_prev_x {
                    self.formula.add_clause(&[cc_start[y][x].negative(), npx]);
                }
                
                // CC start reaches itself in 0 steps
                // cc_start[y][x] <=> in_cc[y][x][0]
                self.formula.add_clause(&[cc_start[y][x].negative(), in_cc[y][x][0].positive()]);
                self.formula.add_clause(&[cc_start[y][x].positive(), in_cc[y][x][0].negative()]);
                
                // Reachability propagation
                for i in 1..=max_dist.min(20) {  // Limit to 20 steps for performance
                    // in_cc[y][x][i] => filled AND (in_cc[y][x][i-1] OR neighbor_reaches_in_i-1)
                    
                    self.formula.add_clause(&[in_cc[y][x][i].negative(), is_filled[y][x].positive()]);
                    
                    let mut reasons = vec![in_cc[y][x][i - 1].positive()];
                    if x > 0 { reasons.push(in_cc[y][x - 1][i - 1].positive()); }
                    if x + 1 < size { reasons.push(in_cc[y][x + 1][i - 1].positive()); }
                    if y > 0 { reasons.push(in_cc[y - 1][x][i - 1].positive()); }
                    if y + 1 < size { reasons.push(in_cc[y + 1][x][i - 1].positive()); }
                    
                    let mut clause = vec![in_cc[y][x][i].negative()];
                    clause.extend(reasons);
                    self.formula.add_clause(&clause);
                }
                
                // All filled cells must reach CC start (within max steps)
                let final_dist = max_dist.min(20);
                self.formula.add_clause(&[is_filled[y][x].negative(), in_cc[y][x][final_dist].positive()]);
            }
        }
        
        debug_log!("[ENCODER] Added full reachability CC constraint");
        
        is_filled
    }
}
