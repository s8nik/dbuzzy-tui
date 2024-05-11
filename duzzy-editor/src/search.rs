use ropey::RopeSlice;

pub type MatchRange = (usize, usize);

struct SearchIter<'a> {
    cur_ofs: usize,
    pattern: &'a str,
    pattern_len: usize,
    char_iter: ropey::iter::Chars<'a>,
    possible_matches: Vec<std::str::Chars<'a>>,
}

impl<'a> SearchIter<'a> {
    fn from_rope_slice<'b>(slice: &'b RopeSlice, search_pattern: &'b str) -> SearchIter<'b> {
        assert!(slice.len_chars() != 0, "Text couldn't be empty.");
        assert!(
            !search_pattern.is_empty(),
            "Can't search using an empty search pattern."
        );

        SearchIter {
            cur_ofs: 0,
            char_iter: slice.chars(),
            pattern: search_pattern,
            pattern_len: search_pattern.chars().count(),
            possible_matches: Vec::new(),
        }
    }
}

impl Iterator for SearchIter<'_> {
    type Item = MatchRange;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(next_char) = self.char_iter.by_ref().next() {
            self.cur_ofs += 1;
            self.possible_matches.push(self.pattern.chars());

            let mut i = 0;
            while i < self.possible_matches.len() {
                let pattern_char = self.possible_matches[i].next().unwrap();
                if next_char == pattern_char {
                    if self.possible_matches[i].clone().next().is_none() {
                        let char_match_range = (self.cur_ofs - self.pattern_len, self.cur_ofs);
                        self.possible_matches.clear();
                        return Some(char_match_range);
                    } else {
                        i += 1;
                    }
                } else {
                    let _ = self.possible_matches.swap_remove(i);
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {}
