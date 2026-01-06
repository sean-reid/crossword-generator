use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Note: This module uses `rand` for random definition selection.
// Make sure `rand` is included in Cargo.toml dependencies.
//
// This implementation is faithful to the Wordset dictionary format.
// Wordset definitions are already clean and well-formatted, so minimal
// processing is needed.

// Wordset JSON structure
#[derive(Debug, Deserialize)]
struct WordsetMeaning {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    def: Option<String>,
    #[serde(default)]
    example: Option<String>,
    #[serde(default)]
    speech_part: Option<String>,
    #[serde(default)]
    synonyms: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct WordsetEntry {
    #[serde(default)]
    word: Option<String>,
    #[serde(default)]
    wordset_id: Option<String>,
    #[serde(default)]
    meanings: Vec<WordsetMeaning>,
    #[serde(default)]
    editors: Vec<String>,
    #[serde(default)]
    contributors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictionaryStats {
    pub word_count: usize,
    pub avg_word_length: f32,
    pub max_word_length: usize,
}

pub struct Dictionary {
    /// Maps uppercase word -> list of clean definitions
    entries: HashMap<String, Vec<String>>,
    /// Filtered list of valid crossword words (uppercase)
    words: Vec<String>,
}

impl Dictionary {
    pub fn new() -> Self {
        Self::with_allowlist(None)
    }

    /// Load dictionary from embedded all_words.json with optional allowlist filtering
    pub fn with_allowlist(allowlist: Option<&str>) -> Self {
        // Load the embedded JSON
        let json_text = include_str!("../all_words.json");
        
        // Parse allowlist if provided
        let allowed_words: Option<std::collections::HashSet<String>> = allowlist.map(|list| {
            list.lines()
                .map(|line| line.trim().to_uppercase())
                .filter(|line| !line.is_empty() && line.chars().all(|c| c.is_ascii_alphabetic()))
                .collect()
        });

        // Parse JSON - use serde_json::Value for more robust handling
        let json_value: serde_json::Value = serde_json::from_str(json_text)
            .expect("Failed to parse all_words.json");
        
        let wordset_dict = match json_value.as_object() {
            Some(obj) => obj,
            None => panic!("all_words.json root is not an object"),
        };

        let mut entries: HashMap<String, Vec<String>> = HashMap::new();

        for (word_key, entry_value) in wordset_dict.iter() {
            // Skip if entry is null
            if entry_value.is_null() {
                continue;
            }
            
            // Try to deserialize this entry
            let entry: WordsetEntry = match serde_json::from_value(entry_value.clone()) {
                Ok(e) => e,
                Err(_) => continue, // Skip malformed entries
            };
            
            // Get the word, skip if null
            let word = match &entry.word {
                Some(w) => w,
                None => continue,
            };
            
            // Normalize word: uppercase, remove hyphens/spaces
            let normalized = Self::normalize_word(word);
            
            if normalized.is_empty() {
                continue;
            }

            // Check allowlist
            if let Some(ref allowed) = allowed_words {
                if !allowed.contains(&normalized) {
                    continue;
                }
            }

            // Extract and clean definitions
            let definitions = Self::extract_definitions(&entry.meanings);
            
            if !definitions.is_empty() {
                entries.insert(normalized, definitions);
            }
        }

        // Filter to valid crossword words with clean clues
        let words: Vec<String> = entries
            .iter()
            .filter(|(word, defs)| {
                let len = word.len();
                let valid_length = len >= 3 && len <= 15;
                let valid_chars = word.chars().all(|c| c.is_ascii_alphabetic());
                let has_clean_clue = Self::get_random_definition(defs).is_some();
                
                valid_length && valid_chars && has_clean_clue
            })
            .map(|(w, _)| w.clone())
            .collect();

        Dictionary { entries, words }
    }

    /// Normalize a word for lookup: uppercase, strip hyphens/spaces
    fn normalize_word(word: &str) -> String {
        word.chars()
            .filter(|c| c.is_ascii_alphabetic())
            .collect::<String>()
            .to_uppercase()
    }

    /// Extract definitions from Wordset meanings
    /// Wordset data is already clean, so minimal processing needed
    fn extract_definitions(meanings: &[WordsetMeaning]) -> Vec<String> {
        let mut definitions = Vec::new();

        for meaning in meanings {
            // Skip if definition or speech_part is null
            let def = match &meaning.def {
                Some(d) => d,
                None => continue,
            };
            
            let speech_part = match &meaning.speech_part {
                Some(sp) => sp,
                None => continue,
            };
            
            // Skip certain parts of speech that aren't good for crosswords
            let pos_lower = speech_part.to_lowercase();
            if pos_lower.contains("prefix")
                || pos_lower.contains("suffix")
                || pos_lower.contains("combining form")
                || pos_lower.contains("abbreviation")
            {
                continue;
            }

            // Wordset definitions are already clean - just capitalize first letter
            let trimmed = def.trim();
            if trimmed.is_empty() || trimmed.len() < 10 {
                continue;
            }

            // Capitalize first letter
            let mut chars = trimmed.chars();
            if let Some(first) = chars.next() {
                let capitalized = first.to_uppercase().collect::<String>() + chars.as_str();
                definitions.push(capitalized);
            }
        }

        definitions
    }

    /// Get a random definition from a list of definitions
    /// Filters for quality but returns a random one for variety in crosswords
    #[cfg(feature = "wasm")]
    fn get_random_definition(definitions: &[String]) -> Option<String> {
        use rand::seq::SliceRandom;
        
        let filtered: Vec<&String> = definitions
            .iter()
            .filter(|d| {
                // Basic quality filters
                d.len() >= 10 && d.len() <= 200
            })
            .collect();
        
        filtered.choose(&mut rand::thread_rng()).map(|s| (*s).clone())
    }
    
    /// Get a random definition from a list of definitions (non-WASM version)
    #[cfg(not(feature = "wasm"))]
    fn get_random_definition(definitions: &[String]) -> Option<String> {
        use rand::seq::SliceRandom;
        
        let filtered: Vec<&String> = definitions
            .iter()
            .filter(|d| {
                // Basic quality filters
                d.len() >= 10 && d.len() <= 200
            })
            .collect();
        
        filtered.choose(&mut rand::thread_rng()).map(|s| (*s).clone())
    }

    pub fn get_words(&self) -> &[String] {
        &self.words
    }

    /// Get a crossword clue for a word
    /// 
    /// Returns a random valid definition for variety, or "Definition not available"
    pub fn get_clue(&self, word: &str) -> String {
        let word_upper = word.to_uppercase();
        
        if let Some(definitions) = self.entries.get(&word_upper) {
            Self::get_random_definition(definitions)
                .unwrap_or_else(|| "Definition not available".to_string())
        } else {
            "Definition not available".to_string()
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_word() {
        assert_eq!(Dictionary::normalize_word("hello"), "HELLO");
        assert_eq!(Dictionary::normalize_word("a cappella"), "ACAPPELLA");
        assert_eq!(Dictionary::normalize_word("anno-Domini"), "ANNODOMINI");
        assert_eq!(Dictionary::normalize_word("test-word"), "TESTWORD");
    }

    #[test]
    fn test_dictionary_loading() {
        // This test requires the actual JSON file
        // Just test that it doesn't panic
        let dict = Dictionary::new();
        assert!(dict.get_words().len() > 0);
    }
}
