use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictionaryStats {
    pub word_count: usize,
    pub avg_word_length: f32,
    pub max_word_length: usize,
}

pub struct Dictionary {
    entries: HashMap<String, String>,
    words: Vec<String>,
}

impl Dictionary {
    pub fn new() -> Self {
        let dict_text = include_str!("../Oxford_English_Dictionary.txt");
        let mut entries = HashMap::new();
        
        for line in dict_text.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            
            if let Some(first_char) = trimmed.chars().next() {
                if first_char.is_uppercase() && first_char.is_alphabetic() {
                    let parts: Vec<&str> = trimmed.splitn(2, "  ").collect();
                    
                    if parts.len() == 2 {
                        let word = parts[0].trim();
                        let definition = parts[1].trim();
                        
                        if !word.is_empty() && word.chars().all(|c| c.is_alphabetic() || c == '-') {
                            let mut word_clean = word.replace("-", "");
                            word_clean = word_clean.trim_end_matches(|c: char| c.is_ascii_digit()).to_string();
                            
                            if !word_clean.is_empty() {
                                let def_lower = definition.to_lowercase();
                                let is_reference = def_lower.starts_with("var. of")
                                    || def_lower.starts_with("variant of")
                                    || def_lower.starts_with("see ")
                                    || def_lower.starts_with("= ")
                                    || def_lower.starts_with("of *")
                                    || (def_lower.starts_with("of ") && def_lower.contains("*"));
                                
                                if !is_reference {
                                    entries.insert(word_clean.to_uppercase(), definition.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
        
        let words: Vec<String> = entries
            .iter()
            .filter(|(w, def)| {
                let len = w.len();
                let valid_word = len >= 3 && len <= 15 && w.chars().all(|c| c.is_ascii_alphabetic());
                
                let def_lower = def.to_lowercase();
                let not_special = !def_lower.starts_with("prefix")
                    && !def_lower.starts_with("suffix")
                    && !def_lower.starts_with("abbr.")
                    && !def_lower.contains("abbr. ")
                    && !w.ends_with('.');
                
                let clue = Self::extract_clue(def);
                let clean_clue = clue != "Definition not available" 
                    && !clue.to_lowercase().contains(&w.to_lowercase())
                    && clue.len() > 10
                    && !clue.to_lowercase().starts_with("of ")
                    && !clue.contains(") ")
                    && !clue.ends_with(")")
                    && !clue.contains("*");
                
                valid_word && not_special && clean_clue
            })
            .map(|(w, _)| w.clone())
            .collect();
        
        Dictionary { entries, words }
    }
    
    pub fn get_words(&self) -> &[String] {
        &self.words
    }
    
    pub fn get_clue(&self, word: &str) -> String {
        let word_upper = word.to_uppercase();
        if let Some(def) = self.entries.get(&word_upper) {
            Self::extract_clue(def)
        } else {
            "Definition not available".to_string()
        }
    }
    
    fn extract_clue(definition: &str) -> String {
        if definition.trim().is_empty() {
            return "Definition not available".to_string();
        }
        
        let mut def = definition.trim().to_string();
        
        // Remove style labels
        for label in &["literary ", "formal ", "archaic "] {
            if def.to_lowercase().starts_with(label) {
                def = def[label.len()..].to_string();
            }
        }
        
        // Handle em-dash + part of speech
        if def.starts_with('—') || def.starts_with('–') || def.starts_with("--") {
            if let Some(period_pos) = def.find(". ") {
                def = def[period_pos + 2..].to_string();
            }
        }
        
        // Remove part of speech at start
        for marker in &["attrib. adj. ", "attrib.adj. ", "n.pl. ", "v.tr. ", "v.intr. ", "adv. ", "adj. ", "n. ", "v. ", "prep. ", "conj. "] {
            if def.to_lowercase().starts_with(marker) {
                def = def[marker.len()..].to_string();
                break;
            }
        }
        
        def = def.trim().to_string();
        
        // Remove plural/conjugation notes at start
        if def.starts_with('(') && def.len() > 3 {
            if let Some(close) = def.find(')') {
                if close < 25 {
                    def = def[close + 1..].trim().to_string();
                }
            }
        }
        
        // Extract first numbered definition
        if let Some(digit_pos) = def.find(|c: char| c.is_ascii_digit()) {
            if digit_pos > 0 {
                def = def[digit_pos + 1..].trim_start().to_string();
            } else {
                def = def[1..].trim_start().to_string();
            }
        }
        
        // Remove usage labels
        for label in &["colloq. ", "esp. ", "usu. "] {
            if def.to_lowercase().starts_with(label) {
                def = def[label.len()..].to_string();
            }
        }
        
        // Remove usage parentheticals
        if def.starts_with('(') {
            if let Some(close) = def.find(')') {
                let content = &def[1..close].to_lowercase();
                if content.contains("foll") || content.contains("usu") || content.contains("often") {
                    def = def[close + 1..].trim().to_string();
                }
            }
        }
        
        // Remove secondary em-dash definitions
        if let Some(pos) = def.find(" —") {
            def = def[..pos].trim().to_string();
        }
        
        // Stop at next numbered definition
        let mut search_pos = 0;
        while let Some(period_pos) = def[search_pos..].find(". ") {
            let abs_pos = search_pos + period_pos;
            let after_period = &def[abs_pos + 2..];
            
            if after_period.chars().next().map_or(false, |c| c.is_ascii_digit()) {
                def = def[..abs_pos].to_string();
                break;
            }
            search_pos = abs_pos + 2;
        }
        
        // Remove parentheticals
        let mut iter = 0;
        while let Some(open) = def.find('(') {
            if iter > 3 { break; }
            iter += 1;
            
            if let Some(close) = def[open..].find(')') {
                let before = def[..open].trim();
                let after = def[open + close + 1..].trim();
                def = if before.is_empty() {
                    after.to_string()
                } else if after.is_empty() {
                    before.to_string()
                } else {
                    format!("{} {}", before, after)
                };
            } else {
                break;
            }
        }
        
        // Split on semicolon
        def = def.split("; ").next().unwrap_or(&def).trim().to_string();
        
        // Remove control characters
        def = def.chars().filter(|c| !c.is_control() || c.is_whitespace()).collect::<String>();
        
        // Remove etymology
        if let Some(pos) = def.rfind('[') {
            def = def[..pos].trim().to_string();
        }
        
        // Remove trailing POS
        for suffix in &[" n. & adj", " adj. & n", " n. & v", " v. & n"] {
            if def.to_lowercase().ends_with(suffix) {
                def = def[..def.len() - suffix.len()].trim().to_string();
                break;
            }
        }
        
        // Remove trailing single-word POS
        if let Some(last_space) = def.rfind(' ') {
            let after_space = &def[last_space + 1..];
            if after_space == "adj" || after_space == "adv" || after_space == "n" || after_space == "v" {
                def = def[..last_space].trim().to_string();
            }
        }
        
        // Remove derivative forms at end
        loop {
            let original_len = def.len();
            let parts: Vec<&str> = def.rsplitn(3, ' ').collect();
            if parts.len() >= 2 {
                let last = parts[0].trim_end_matches('.');
                if ["adj", "adv", "n", "v", "prep", "conj", "pron"].contains(&last) {
                    let mut words: Vec<&str> = def.split_whitespace().collect();
                    if words.len() >= 2 {
                        words.truncate(words.len() - 2);
                        def = words.join(" ");
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
            if def.len() >= original_len {
                break;
            }
        }
        
        def = def.trim_end_matches('.').trim().to_string();
        
        if def.len() < 3 {
            return "Definition not available".to_string();
        }
        
        // FINAL: Stop at letter enumeration (after all other cleanup)
        for letter in ['a', 'b', 'c', 'd', 'e'] {
            let pattern1 = format!(". {}", letter);
            let pattern2 = format!(" {} ", letter);
            
            if let Some(pos) = def.find(&pattern1) {
                def = def[..pos].to_string();
                break;
            } else if let Some(pos) = def.find(&pattern2) {
                def = def[..pos].to_string();
                break;
            }
        }
        
        // Strip leading enumeration letter
        def = def.trim().to_string();
        if def.len() > 2 {
            let first = def.chars().next();
            let second = def.chars().nth(1);
            if matches!(first, Some('A') | Some('a') | Some('B') | Some('b') | Some('C') | Some('c'))
                && second == Some(' ') {
                def = def[2..].trim().to_string();
            }
        }
        
        def = def.trim().to_string();
        
        if def.len() < 3 {
            return "Definition not available".to_string();
        }
        
        // Normalize capitalization
        def = def.to_lowercase();
        let mut chars = def.chars();
        if let Some(first) = chars.next() {
            def = first.to_uppercase().collect::<String>() + chars.as_str();
        } else {
            return "Definition not available".to_string();
        }
        
        // VERY FINAL: Strip leading POS that got capitalized (like "N.s-shaped")
        for marker in ["N.", "V.", "Adj.", "Adv.", "Prep.", "Conj."] {
            if def.starts_with(marker) {
                def = def[marker.len()..].to_string();
                // Capitalize first letter again after stripping
                let mut chars = def.chars();
                if let Some(first) = chars.next() {
                    def = first.to_uppercase().collect::<String>() + chars.as_str();
                }
                break;
            }
        }
        
        def
    }
    
    pub fn stats(&self) -> DictionaryStats {
        let total_len: usize = self.words.iter().map(|w| w.len()).sum();
        let avg_len = if self.words.is_empty() {
            0.0
        } else {
            total_len as f32 / self.words.len() as f32
        };
        
        let max_len = self.words.iter().map(|w| w.len()).max().unwrap_or(0);
        
        DictionaryStats {
            word_count: self.words.len(),
            avg_word_length: avg_len,
            max_word_length: max_len,
        }
    }
}
