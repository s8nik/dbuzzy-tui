use ropey::{Rope, RopeSlice};

pub type SelectedRange = (usize, usize);

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct Selection {
    anchor: usize,
    head: usize,
}

impl Selection {
    pub const fn new(pos: usize) -> Self {
        Self {
            anchor: pos,
            head: pos,
        }
    }

    pub fn start(&self) -> usize {
        self.head.min(self.anchor)
    }

    pub fn end(&self) -> usize {
        self.head.max(self.anchor)
    }

    pub fn range(&self) -> SelectedRange {
        (self.start(), self.end())
    }

    pub fn slice(self, rope: &Rope) -> RopeSlice<'_> {
        let (start, mut end) = self.range();

        if end == rope.len_chars() {
            end -= 1;
        }

        rope.slice(start..=end)
    }

    pub fn update(&mut self, pos: usize) {
        self.head = pos;
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SpanKind {
    Nothing,
    Selection,
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug)]
pub struct SelectionSpan<'a> {
    pub slice: RopeSlice<'a>,
    pub kind: SpanKind,
}

struct SpanIterator<'a> {
    cursor: usize,
    line: RopeSlice<'a>,
    range: SelectedRange,
}

impl<'a> SpanIterator<'a> {
    pub const fn new(line: RopeSlice<'a>, range: SelectedRange) -> Self {
        Self {
            cursor: 0,
            line,
            range,
        }
    }
}

impl<'a> Iterator for SpanIterator<'a> {
    type Item = SelectionSpan<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.cursor;
        let (start, end) = self.range;
        let line_len = self.line.len_chars();

        if start == end {
            return None;
        }

        let mut kind = SpanKind::Nothing;

        let slice = if current == start {
            self.cursor = end;
            kind = SpanKind::Selection;
            self.line.slice(current..=end)
        } else if current == end && current != line_len {
            self.cursor = line_len;
            self.line.slice((end + 1).min(line_len)..line_len)
        } else if current == 0 {
            self.cursor = start;
            self.line.slice(current..start)
        } else {
            return None;
        };

        if slice.len_bytes() != 0 {
            Some(Self::Item { slice, kind })
        } else {
            None
        }
    }
}

pub fn selection_spans(
    line_idx: usize,
    max_len: usize,
    line: RopeSlice<'_>,
    selection: SelectedRange,
) -> Vec<SelectionSpan> {
    let (start, end) = selection;
    let overlaps = start < line_idx + max_len && line_idx <= end;

    if overlaps && line == "\n" {
        return vec![SelectionSpan {
            slice: RopeSlice::from(" "),
            kind: SpanKind::Selection,
        }];
    }

    let in_line_range = (
        start.saturating_sub(line_idx).min(max_len),
        end.saturating_sub(line_idx).min(max_len),
    );

    overlaps
        .then(|| SpanIterator::new(line, in_line_range).collect())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use ropey::RopeSlice;

    use super::{selection_spans, Selection, SelectionSpan, SpanIterator, SpanKind};

    #[test]
    fn test_select_all() {
        let text = ropey::Rope::from_str("test test");

        let mut selection = Selection::new(0);
        selection.update(text.len_chars() - 1);

        let mut iter = SpanIterator::new(text.slice(..), selection.range());

        assert_eq!(
            iter.next(),
            Some(SelectionSpan {
                kind: SpanKind::Selection,
                slice: RopeSlice::from("test test")
            })
        );

        assert_eq!(iter.next(), None,);
    }

    #[test]
    fn test_select_slice() {
        let text = ropey::Rope::from_str("test test");

        let mut selection = Selection::new(3);
        selection.update(6);

        let mut iter = SpanIterator::new(text.slice(..), selection.range());

        assert_eq!(
            iter.next(),
            Some(SelectionSpan {
                kind: SpanKind::Nothing,
                slice: RopeSlice::from("tes")
            })
        );

        assert_eq!(
            iter.next(),
            Some(SelectionSpan {
                kind: SpanKind::Selection,
                slice: RopeSlice::from("t te")
            })
        );

        assert_eq!(
            iter.next(),
            Some(SelectionSpan {
                kind: SpanKind::Nothing,
                slice: RopeSlice::from("st")
            })
        );

        assert_eq!(iter.next(), None,);

        let mut selection = Selection::new(3);
        selection.update(2);

        let mut iter = SpanIterator::new(text.slice(..), selection.range());

        assert_eq!(
            iter.next(),
            Some(SelectionSpan {
                kind: SpanKind::Nothing,
                slice: RopeSlice::from("te")
            })
        );

        assert_eq!(
            iter.next(),
            Some(SelectionSpan {
                kind: SpanKind::Selection,
                slice: RopeSlice::from("st")
            })
        );

        assert_eq!(
            iter.next(),
            Some(SelectionSpan {
                kind: SpanKind::Nothing,
                slice: RopeSlice::from(" test")
            })
        );

        assert_eq!(iter.next(), None,);
    }

    #[test]
    fn test_selection_spans() {
        let text = ropey::Rope::from_str("test test\ntest line 2");
        let selection = (3, 14);

        let line = text.line(0);
        let line_idx = text.line_to_char(0);
        let max_len = line.len_chars() - 1;
        let mut spans = selection_spans(line_idx, max_len, line, selection).into_iter();

        assert_eq!(spans.len(), 2);

        assert_eq!(
            spans.next(),
            Some(SelectionSpan {
                kind: SpanKind::Nothing,
                slice: RopeSlice::from("tes"),
            })
        );

        assert_eq!(
            spans.next(),
            Some(SelectionSpan {
                kind: SpanKind::Selection,
                slice: RopeSlice::from("t test\n"),
            })
        );

        let line = text.line(1);
        let line_idx = text.line_to_char(1);
        let max_len = line.len_chars();
        let mut spans = selection_spans(line_idx, max_len, line, selection).into_iter();

        assert_eq!(spans.len(), 2);

        assert_eq!(
            spans.next(),
            Some(SelectionSpan {
                kind: SpanKind::Selection,
                slice: RopeSlice::from("test "),
            })
        );

        assert_eq!(
            spans.next(),
            Some(SelectionSpan {
                kind: SpanKind::Nothing,
                slice: RopeSlice::from("line 2"),
            })
        );
    }
}
