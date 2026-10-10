use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Dictionary: custom word substitutions (chiffres acceptés ✅).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dictionary {
    /// Word → replacement (exact word matching)
    pub entries: HashMap<String, String>,
    /// Optional super-mode phonetic entries
    pub super_entries: HashMap<String, String>,
}

impl Dictionary {
    pub fn path() -> PathBuf {
        crate::config::Config::config_dir().join("dictionary.json")
    }

    pub fn load() -> Self {
        let path = Self::path();
        if path.exists() {
            std::fs::read_to_string(&path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            Default::default()
        }
    }

    pub fn save(&self) -> std::io::Result<()> {
        let dir = Self::path().parent().unwrap();
        std::fs::create_dir_all(dir)?;
        std::fs::write(Self::path(), serde_json::to_string_pretty(self)?)
    }

    /// Apply dictionary to raw text: word-by-word replacement.
    /// Supports multi-word keys (checked as phrases first).
    pub fn apply(&self, text: &str) -> String {
        let mut result = text.to_string();
        // Apply multi-word entries first (longest first to avoid partial matches)
        let mut entries: Vec<(&String, &String)> = self.entries.iter().collect();
        entries.sort_by_key(|(k, _)| std::cmp::Reverse(k.len()));

        for (from, to) in &entries {
            // Word-boundary replacement
            let pattern = format!(r"\b{}\b", regex_escape(from));
            if let Ok(re) = regex::Regex::new(&pattern) {
                result = re.replace_all(&result, to.as_str()).to_string();
            }
        }
        result
    }
}

impl Default for Dictionary {
    fn default() -> Self {
        let mut entries = HashMap::new();
        // Built-in examples
        entries.insert("c4".to_string(), "Compétition 4 ever".to_string());
        entries.insert("m3".to_string(), "Machine 3".to_string());
        Self {
            entries,
            super_entries: HashMap::new(),
        }
    }
}

/// Minimal regex escape for user-provided strings.
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

/// Check that digits in dictionary entries are preserved.
#[test]
fn test_dictionary_accepts_digits() {
    let mut dict = Dictionary::default();
    dict.entries.insert("c4".into(), "Competition 4 ever".into());
    dict.entries.insert("pro2".into(), "Protocol 2".into());

    let result = dict.apply("I use c4 and pro2");
    assert_eq!(result, "I use Competition 4 ever and Protocol 2");
}