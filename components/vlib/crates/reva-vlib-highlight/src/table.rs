//! Token tables: styles per class, and each word's class.

use std::collections::HashMap;

/// The classes the tokenizer finds by itself.
const LEXICAL: &[&str] = &["com", "stk", "str", "num", "colon", "def"];

/// Token tables merged: SGR codes per class, and each word's class.
#[derive(Default, Debug)]
pub struct Table {
    pub sgr: HashMap<String, String>,
    pub words: HashMap<Vec<u8>, String>,
}

impl Table {
    /// Add the lines of one table text (later lines win).
    pub fn add(&mut self, text: &str) {
        for line in text.lines() {
            let mut f = line.split_ascii_whitespace();
            match (f.next(), f.next()) {
                (Some("style"), Some(class)) => {
                    let sgr = f.next().unwrap_or("0");
                    self.sgr.insert(class.to_string(), sgr.to_string());
                }
                (Some("words"), Some(class)) => {
                    for w in f {
                        self.words.insert(w.as_bytes().to_vec(), class.to_string());
                    }
                }
                _ => {}
            }
        }
    }

    /// The class of word `w`, as written or in lower case.
    pub fn class_of(&self, w: &[u8]) -> Option<&str> {
        self.words
            .get(w)
            .or_else(|| self.words.get(&w.to_ascii_lowercase()))
            .map(String::as_str)
    }

    /// A class the tokenizer named, as the table's own string (plain if
    /// the table has no style for a lexical class); other classes as given.
    pub fn lexical<'t>(&'t self, class: &'t str) -> &'t str {
        if LEXICAL.contains(&class) {
            self.sgr
                .get_key_value(class)
                .map_or("", |(s, _)| s.as_str())
        } else {
            class
        }
    }
}
