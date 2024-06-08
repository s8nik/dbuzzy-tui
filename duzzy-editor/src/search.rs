use std::collections::HashMap;

use ropey::{Rope, RopeSlice};

pub type MatchRange = (usize, usize);

pub struct SearchRegistry<'a> {
    done: bool,
    index: usize,
    matches: Vec<MatchRange>,
    iter: SearchIter<'a>,
}

impl<'a> SearchRegistry<'a> {
    pub fn new(text: &'a Rope, pattern: &'a str) -> Self {
        Self {
            index: 0,
            done: false,
            matches: vec![],
            iter: SearchIter::from_rope_slice(text.slice(..), pattern),
        }
    }

    pub fn pattern(&self) -> &str {
        self.iter.pattern
    }

    pub fn next(&mut self) -> Option<MatchRange> {
        if self.done {
            if self.index == self.matches.len() - 1 {
                self.index = 0;
            }

            return self.matches.get(self.index).cloned();
        }

        let Some(range) = self.iter.next() else {
            self.done = true;
            self.index = 0;
            return self.matches.get(self.index).cloned();
        };

        self.index += 1;
        self.matches.push(range);

        Some(range)
    }

    pub fn prev(&mut self) -> Option<MatchRange> {
        self.index = if self.index == 0 {
            self.matches.len() - 1
        } else {
            self.index - 1
        };

        self.matches.get(self.index).cloned()
    }
}

#[derive(Debug)]
struct SearchIter<'a> {
    text: RopeSlice<'a>,
    offset: usize,
    pattern: &'a str,
    pattern_len: usize,
    bad_match_table: HashMap<char, usize>,
}

impl<'a> SearchIter<'a> {
    fn from_rope_slice(slice: RopeSlice<'a>, search_pattern: &'a str) -> SearchIter<'a> {
        assert!(slice.len_chars() != 0, "The text cannot be empty.");
        assert!(
            !search_pattern.is_empty(),
            "Can't search using an empty search pattern."
        );

        let pattern_len = search_pattern.len();

        SearchIter {
            offset: 0,
            text: slice,
            pattern: search_pattern,
            pattern_len,
            bad_match_table: Self::bad_match_table(search_pattern, pattern_len),
        }
    }

    fn bad_match_table(search_pattern: &str, pattern_len: usize) -> HashMap<char, usize> {
        let mut table = HashMap::new();

        for (i, c) in search_pattern.chars().enumerate() {
            let shift = 1.max(pattern_len - i - 1);
            table.insert(c, shift);
        }

        table
    }
}

impl Iterator for SearchIter<'_> {
    type Item = MatchRange;

    fn next(&mut self) -> Option<Self::Item> {
        let mut index = self.offset;

        while index <= self.text.len_chars() - self.pattern_len {
            let mut skips = 0;

            for i in (0..self.pattern_len).rev() {
                let ch_text = self.text.char(index + i);
                let ch_pattern = self.pattern.chars().nth(i)?;

                if ch_pattern != ch_text {
                    match self.bad_match_table.get(&ch_text) {
                        Some(s) => skips = *s,
                        None => skips = self.pattern_len,
                    };

                    break;
                }
            }

            if skips == 0 {
                let offset = index + self.pattern_len;
                self.offset = offset;
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

    use super::SearchIter;

    #[test]
    fn test_search_single_line() {
        let text = Rope::from_str("lotestlol");
        let mut iter = SearchIter::from_rope_slice(text.slice(..), "lo");

        assert_eq!(iter.next(), Some((0, 2)));
        assert_eq!(iter.next(), Some((6, 8)));
        assert_eq!(iter.next(), None);

        let mut iter = SearchIter::from_rope_slice(text.slice(..), "test");
        assert_eq!(iter.next(), Some((2, 6)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_search_multiple_line() {
        let text = Rope::from_str("foo line1\nbar line2");
        let mut iter = SearchIter::from_rope_slice(text.slice(..), "line");

        assert_eq!(iter.next(), Some((4, 8)));
        assert_eq!(iter.next(), Some((14, 18)));
    }
}
