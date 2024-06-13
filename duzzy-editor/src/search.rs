use std::collections::HashMap;

use ropey::Rope;

use crate::SmartString;

pub type MatchRange = (usize, usize);

#[derive(PartialEq)]
pub enum SearchOrder {
    Next,
    Prev,
}

#[derive(Default)]
pub struct SearchRegistry {
    buffer: SmartString,
    pattern: Option<SmartString>,
    bad_match_table: HashMap<char, (usize, usize)>,
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
            let shifts = (1.max(i), 1.max(pattern_len - i - 1));
            self.bad_match_table.insert(c, shifts);
        }

        self.buffer.clear();
    }

    pub fn cancel(&mut self) {
        self.buffer.clear();
    }

    pub fn search(&self, text: &Rope, start_pos: usize, order: SearchOrder) -> Option<MatchRange> {
        assert!(text.len_chars() != 0, "The text cannot be empty.");

        let pattern = self.pattern.as_ref()?;

        assert!(
            !pattern.is_empty(),
            "Can't search using an empty search pattern."
        );

        let index = if start_pos >= text.len_chars() {
            text.len_chars() - 1
        } else {
            start_pos
        };

        match order {
            SearchOrder::Next => Self::search_next(pattern, text, index, &self.bad_match_table),
            SearchOrder::Prev => Self::search_prev(pattern, text, index, &self.bad_match_table),
        }
    }

    fn search_next(
        pattern: &SmartString,
        text: &Rope,
        mut index: usize,
        bmt: &HashMap<char, (usize, usize)>,
    ) -> Option<MatchRange> {
        let pattern_len = pattern.len();

        while index <= text.len_chars() - pattern_len {
            let mut skips = 0;

            for i in (0..pattern_len).rev() {
                let c_text = text.char(index + i);
                let c_pattern = pattern.chars().nth(i)?;

                if c_text != c_pattern {
                    match bmt.get(&c_text) {
                        Some((_, forward)) => skips = *forward,
                        None if c_text.is_ascii_whitespace() => skips = 1,
                        None => skips = pattern_len,
                    };

                    break;
                }
            }

            if skips == 0 {
                let offset = index + pattern_len - 1;
                return Some((index, offset));
            }

            index += skips;
        }

        None
    }

    fn search_prev(
        pattern: &SmartString,
        text: &Rope,
        mut index: usize,
        bmt: &HashMap<char, (usize, usize)>,
    ) -> Option<MatchRange> {
        let pattern_len = pattern.len();

        while index + 1 >= pattern_len {
            let mut skips = 0;

            for i in (0..pattern_len).rev() {
                let c_text = text.char(index - i);
                let c_pattern = pattern.chars().nth(pattern_len - i - 1)?;

                if c_text != c_pattern {
                    match bmt.get(&c_text) {
                        Some((backward, _)) => skips = *backward,
                        None if c_text.is_ascii_whitespace() => skips = 1,
                        None => skips = pattern_len,
                    };

                    break;
                }
            }

            if skips == 0 {
                let offset = index - pattern_len + 1;
                return Some((offset, index));
            }

            if index < skips {
                break;
            }

            index -= skips;
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use ropey::Rope;

    use super::{SearchOrder, SearchRegistry};

    #[test]
    fn test_search_next_single_line() {
        let text = Rope::from_str("lotestlol");
        let mut registry = SearchRegistry::default();

        registry.set_pattern("lo".into());

        assert_eq!(registry.search(&text, 0, SearchOrder::Next), Some((0, 1)));
        assert_eq!(registry.search(&text, 3, SearchOrder::Next), Some((6, 7)));
        assert_eq!(registry.search(&text, 7, SearchOrder::Next), None);

        registry.set_pattern("test".into());

        assert_eq!(registry.search(&text, 0, SearchOrder::Next), Some((2, 5)));
        assert_eq!(registry.search(&text, 4, SearchOrder::Next), None);
    }

    #[test]
    fn test_search_next_multiple_line() {
        let text = Rope::from_str("foo line1\nbar line2");
        let mut registry = SearchRegistry::default();
        registry.set_pattern("line".into());

        assert_eq!(registry.search(&text, 0, SearchOrder::Next), Some((4, 7)));
        assert_eq!(registry.search(&text, 6, SearchOrder::Next), Some((14, 17)));
    }

    #[test]
    fn test_search_prev_multiple_line() {
        let text = Rope::from_str("foo \n loo\n boo");
        let mut registry = SearchRegistry::default();

        registry.set_pattern("oo".into());
        assert_eq!(registry.search(&text, 11, SearchOrder::Prev), Some((7, 8)));
        assert_eq!(registry.search(&text, 6, SearchOrder::Prev), Some((1, 2)));
        assert_eq!(registry.search(&text, 1, SearchOrder::Prev), None);

        let text = Rope::from_str("foo line1\nbar line2");
        let mut registry = SearchRegistry::default();
        registry.set_pattern("line".into());

        assert_eq!(
            registry.search(&text, 18, SearchOrder::Prev),
            Some((14, 17))
        );
        assert_eq!(registry.search(&text, 15, SearchOrder::Prev), Some((4, 7)));
    }
}
