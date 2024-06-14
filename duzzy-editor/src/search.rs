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
    pattern: SmartString,
    bad_match_table: HashMap<char, (usize, usize)>,
}

impl SearchRegistry {
    pub fn new(pattern: SmartString) -> Self {
        assert!(!pattern.is_empty(), "The search pattern cannot be empty.");

        let pattern_len = pattern.len();
        let mut bad_match_table = HashMap::new();
        for (i, c) in pattern.chars().enumerate() {
            let shifts = (1.max(i), 1.max(pattern_len - i - 1));
            bad_match_table.insert(c, shifts);
        }

        Self {
            pattern,
            bad_match_table,
        }
    }

    pub fn search(&self, text: &Rope, start_pos: usize, order: SearchOrder) -> Option<MatchRange> {
        assert!(text.len_chars() != 0, "The text cannot be empty.");
        assert!(
            !self.pattern.is_empty(),
            "Can't search using an empty search pattern."
        );

        let index = if start_pos >= text.len_chars() {
            text.len_chars() - 1
        } else {
            start_pos
        };

        match order {
            SearchOrder::Next => self.search_next(text, index),
            SearchOrder::Prev => self.search_prev(text, index),
        }
    }

    fn search_next(&self, text: &Rope, mut index: usize) -> Option<MatchRange> {
        let pattern_len = self.pattern.len();

        while index <= text.len_chars() - pattern_len {
            let mut skips = 0;

            for i in (0..pattern_len).rev() {
                let c_text = text.char(index + i);
                let c_pattern = self.pattern.chars().nth(i)?;

                if c_text != c_pattern {
                    match self.bad_match_table.get(&c_text) {
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

    fn search_prev(&self, text: &Rope, mut index: usize) -> Option<MatchRange> {
        let pattern_len = self.pattern.len();

        while index + 1 >= pattern_len {
            let mut skips = 0;

            for i in (0..pattern_len).rev() {
                let c_text = text.char(index - i);
                let c_pattern = self.pattern.chars().nth(pattern_len - i - 1)?;

                if c_text != c_pattern {
                    match self.bad_match_table.get(&c_text) {
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
        let registry = SearchRegistry::new("lo".into());

        assert_eq!(registry.search(&text, 0, SearchOrder::Next), Some((0, 1)));
        assert_eq!(registry.search(&text, 3, SearchOrder::Next), Some((6, 7)));
        assert_eq!(registry.search(&text, 7, SearchOrder::Next), None);

        let registry = SearchRegistry::new("test".into());

        assert_eq!(registry.search(&text, 0, SearchOrder::Next), Some((2, 5)));
        assert_eq!(registry.search(&text, 4, SearchOrder::Next), None);
    }

    #[test]
    fn test_search_next_multiple_line() {
        let text = Rope::from_str("foo line1\nbar line2");
        let registry = SearchRegistry::new("line".into());

        assert_eq!(registry.search(&text, 0, SearchOrder::Next), Some((4, 7)));
        assert_eq!(registry.search(&text, 6, SearchOrder::Next), Some((14, 17)));
    }

    #[test]
    fn test_search_prev_multiple_line() {
        let text = Rope::from_str("foo \n loo\n boo");
        let registry = SearchRegistry::new("oo".into());

        assert_eq!(registry.search(&text, 11, SearchOrder::Prev), Some((7, 8)));
        assert_eq!(registry.search(&text, 6, SearchOrder::Prev), Some((1, 2)));
        assert_eq!(registry.search(&text, 1, SearchOrder::Prev), None);

        let text = Rope::from_str("foo line1\nbar line2");
        let registry = SearchRegistry::new("line".into());

        assert_eq!(
            registry.search(&text, 18, SearchOrder::Prev),
            Some((14, 17))
        );
        assert_eq!(registry.search(&text, 15, SearchOrder::Prev), Some((4, 7)));
    }
}
