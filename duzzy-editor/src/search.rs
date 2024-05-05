use ropey::RopeSlice;

pub type MatchRange = (usize, usize);

struct SearchIter<'a> {
    cur_index: usize,
    pattern: &'a str,
    pattern_len: usize,
    chars: ropey::iter::Chars<'a>,
    matches: Vec<std::str::Chars<'a>>,
}

impl<'a> SearchIter<'a> {
    fn from_rope_slice<'b>(slice: &'b RopeSlice, search_pattern: &'b str) -> SearchIter<'b> {
        assert!(
            !search_pattern.is_empty(),
            "Can't search using an empty search pattern."
        );

        SearchIter {
            cur_index: 0,
            chars: slice.chars(),
            pattern: search_pattern,
            pattern_len: search_pattern.chars().count(),
            matches: Vec::new(),
        }
    }
}

impl Iterator for SearchIter<'_> {
    type Item = MatchRange;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

#[cfg(test)]
mod tests {}
