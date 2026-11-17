//! Lucide icon name → codepoint table, parsed from the bundled JSON.

use ahash::AHashMap;

static CODEPOINTS: &str = include_str!("../fonts/lucide-codepoints.json");

pub struct IconTable(AHashMap<&'static str, char>);

impl IconTable {
    pub fn load() -> Self {
        let mut map = AHashMap::with_capacity(2048);
        // The file is a flat `{ "name": 57400, ... }` object.
        for line in CODEPOINTS.lines() {
            let line = line.trim().trim_end_matches(',');
            let Some((k, v)) = line.split_once(':') else { continue };
            let name = k.trim().trim_matches('"');
            if let Some(c) = v.trim().parse::<u32>().ok().and_then(char::from_u32) {
                map.insert(name, c);
            }
        }
        Self(map)
    }

    pub fn get(&self, name: &str) -> Option<char> {
        self.0.get(name).copied()
    }
}
