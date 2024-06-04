use std::collections::HashMap;

use ropey::RopeSlice;

pub type MatchRange = (usize, usize);

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
        let mut index = 0;
        let mut chars = self.text.chars().skip(self.offset);

        while index <= self.text.len_chars() - self.offset - self.pattern_len {
            let mut skips = 0;

            for i in (0..self.pattern_len - 1).rev() {
                let ch_text = chars.nth(index + i).expect("text char");
                let ch_pattern = self.pattern.chars().nth(i).expect("pattern char");

                if ch_pattern != ch_text {
                    match self.bad_match_table.get(&ch_text) {
                        Some(s) => skips = *s,
                        None => skips = self.pattern_len,
                    }
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
    fn test_search() {
        let text = Rope::from_str("lotestlol");
        let mut iter = SearchIter::from_rope_slice(text.slice(..), "lo");

        assert_eq!(iter.next(), Some((0, 2)));
        assert_eq!(iter.next(), Some((6, 8)));
        assert_eq!(iter.next(), None);
    }
}
