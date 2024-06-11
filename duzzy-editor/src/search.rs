use std::collections::HashMap;

use ropey::Rope;

use crate::SmartString;

pub type MatchRange = (usize, usize);

#[derive(Default)]
pub struct SearchRegistry {
    buffer: SmartString,
    pattern: Option<SmartString>,
    bad_match_table: HashMap<char, usize>,
}

impl SearchRegistry {
    pub fn insert_char(&mut self, ch: char) {
        self.buffer.push(ch);
    }

    pub fn remove_char(&mut self) {
        self.buffer.pop();
    }

    pub fn pattern(&self) -> Option<&str> {
        self.pattern.as_deref()
    }

    #[cfg(test)]
    pub fn set_pattern(&mut self, pattern: SmartString) {
        self.buffer = pattern;
        self.apply();
    }

    pub fn apply(&mut self) {
        if self.buffer.is_empty() {
            return;
        }

        self.pattern = Some(self.buffer.to_owned());
        self.bad_match_table = HashMap::new();

        let pattern_len = self.buffer.len();
        for (i, c) in self.buffer.chars().enumerate() {
            let shift = 1.max(pattern_len - i - 1);
            self.bad_match_table.insert(c, shift);
        }

        self.buffer.clear();
    }

    pub fn cancel(&mut self) {
        self.buffer.clear();
    }

    pub fn search(&self, text: &Rope, start_pos: usize) -> Option<MatchRange> {
        assert!(text.len_chars() != 0, "The text cannot be empty.");

        let pattern = self.pattern.as_ref()?;
        let pattern_len = pattern.len();

        assert!(
            !pattern.is_empty(),
            "Can't search using an empty search pattern."
        );

        let mut index = start_pos;

        while index <= text.len_chars() - pattern_len {
            let mut skips = 0;

            for i in (0..pattern_len).rev() {
                let ch_text = text.char(index + i);
                let ch_pattern = pattern.chars().nth(i)?;

                if ch_pattern != ch_text {
                    match self.bad_match_table.get(&ch_text) {
                        Some(s) => skips = *s,
                        None => skips = pattern_len,
                    };

                    break;
                }
            }

            if skips == 0 {
                let offset = index + pattern_len;
                return Some((index, offset));
            }

            index += skips;
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use ropey::Rope;

    use super::SearchRegistry;

    #[test]
    fn test_search_single_line() {
        let text = Rope::from_str("lotestlol");
        let mut registry = SearchRegistry::default();

        registry.set_pattern("lo".into());

        assert_eq!(registry.search(&text, 0), Some((0, 2)));
        assert_eq!(registry.search(&text, 3), Some((6, 8)));
        assert_eq!(registry.search(&text, 7), None);

        registry.set_pattern("test".into());

        assert_eq!(registry.search(&text, 0), Some((2, 6)));
        assert_eq!(registry.search(&text, 4), None);
    }

    #[test]
    fn test_search_multiple_line() {
        let text = Rope::from_str("foo line1\nbar line2");
        let mut registry = SearchRegistry::default();
        registry.set_pattern("line".into());

        assert_eq!(registry.search(&text, 0), Some((4, 8)));
        assert_eq!(registry.search(&text, 6), Some((14, 18)));
    }
}
