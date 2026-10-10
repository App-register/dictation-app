use serde::{Deserialize, Serialize};

/// A single formatting rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub name: String,
    pub enabled: bool,
    pub match_mode: MatchMode,        // "contains", "exact", "regex"
    pub pattern: String,
    pub replacement: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MatchMode {
    Contains,
    Exact,
    Regex,
}

/// Formatting engine — applies all rules in order + built-in toggles.
pub struct FormattingEngine {
    pub rules: Vec<Rule>,
    pub space_before_punctuation: bool,
    pub trailing_space: bool,
    pub text_to_number: bool,
    pub auto_capitalize_threshold: usize,
}

impl FormattingEngine {
    pub fn from_config(cfg: &crate::Config) -> Self {
        let rules_path = crate::config::Config::config_dir().join("formatting_rules.json");
        let custom_rules = if rules_path.exists() {
            std::fs::read_to_string(&rules_path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            Vec::new()
        };

        Self {
            rules: custom_rules,
            space_before_punctuation: cfg.space_before_punctuation,
            trailing_space: cfg.trailing_space,
            text_to_number: cfg.text_to_number,
            auto_capitalize_threshold: cfg.auto_capitalize_threshold,
        }
    }

    /// Process text through the full pipeline (single pass).
    pub fn process(&self, text: &str) -> String {
        let mut result = text.to_string();

        // 1. Custom dictionary rules (already applied by Dictionary module)
        // 2. Custom formatting rules (in order)
        for rule in &self.rules {
            if !rule.enabled { continue; }
            result = match rule.match_mode {
                MatchMode::Contains => {
                    result.replace(&rule.pattern, &rule.replacement)
                }
                MatchMode::Exact => {
                    if result == rule.pattern {
                        rule.replacement.clone()
                    } else {
                        result
                    }
                }
                MatchMode::Regex => {
                    if let Ok(re) = regex::Regex::new(&rule.pattern) {
                        re.replace_all(&result, &rule.replacement).to_string()
                    } else {
                        result
                    }
                }
            };
        }

        // 3. Text-to-number conversion (if enabled)
        if self.text_to_number {
            result = self.convert_text_numbers(&result);
        }

        // 4. Space before punctuation (French typography)
        if self.space_before_punctuation {
            result = result
                .replace("?", " ?")
                .replace("!", " !")
                .replace(" ;", ";") // Already fine
                .replace(":", " :");
            // Clean double spaces
            while result.contains("  ") {
                result = result.replace("  ", " ");
            }
        }

        // 5. Auto-capitalize (if transcription is long enough)
        let word_count = result.split_whitespace().count();
        if word_count >= self.auto_capitalize_threshold {
            // Capitalize first letter
            if let Some(c) = result.chars().next() {
                if c.is_lowercase() {
                    result = c.to_uppercase().to_string() + &result[1..];
                }
            }
        } else {
            // Short text: lowercase (for inline corrections)
            result = result.to_lowercase();
        }

        // 6. Trailing space
        if self.trailing_space && !result.ends_with(' ') {
            result.push(' ');
        }

        result
    }

    /// Basic French/English text-to-number conversion.
    fn convert_text_numbers(&self, text: &str) -> String {
        let mut result = text.to_string();
        // French numbers
        let fr_map = [
            ("zéro", "0"), ("un", "1"), ("deux", "2"), ("trois", "3"),
            ("quatre", "4"), ("cinq", "5"), ("six", "6"), ("sept", "7"),
            ("huit", "8"), ("neuf", "9"), ("dix", "10"),
            ("onze", "11"), ("douze", "12"), ("treize", "13"),
            ("quatorze", "14"), ("quinze", "15"), ("seize", "16"),
            ("vingt", "20"), ("trente", "30"), ("quarante", "40"),
            ("cinquante", "50"), ("soixante", "60"),
            ("cent", "100"), ("mille", "1000"),
        ];
        // English numbers
        let en_map = [
            ("zero", "0"), ("one", "1"), ("two", "2"), ("three", "3"),
            ("four", "4"), ("five", "5"), ("six", "6"), ("seven", "7"),
            ("eight", "8"), ("nine", "9"), ("ten", "10"),
            ("eleven", "11"), ("twelve", "12"), ("thirteen", "13"),
            ("fourteen", "14"), ("fifteen", "15"), ("sixteen", "16"),
            ("seventeen", "17"), ("eighteen", "18"), ("nineteen", "19"),
            ("twenty", "20"), ("thirty", "30"), ("forty", "40"),
            ("fifty", "50"), ("sixty", "60"), ("seventy", "70"),
            ("eighty", "80"), ("ninety", "90"),
            ("hundred", "100"), ("thousand", "1000"),
        ];

        for (word, num) in fr_map.iter().chain(en_map.iter()) {
            let pattern = format!(r"\b{}\b", regex_escape(word));
            if let Ok(re) = regex::Regex::new(&pattern) {
                result = re.replace_all(&result, *num).to_string();
            }
        }

        result
    }
}

fn regex_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '.' | '*' | '+' | '?' | '(' | ')' | '[' | ']' | '{' | '}' | '\\' | '|' | '^' | '$' => {
                out.push('\\');
                out.push(c);
            }
            _ => out.push(c),
        }
    }
    out
}