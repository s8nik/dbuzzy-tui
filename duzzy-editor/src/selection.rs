use ropey::RopeSlice;

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

        if current == start {
            self.cursor = end;

            Some(SelectionSpan {
                slice: self.line.slice(current..end),
                kind: SpanKind::Selection,
            })
        } else if current == end && current != line_len {
            self.cursor = line_len;

            Some(SelectionSpan {
                slice: self.line.slice(end..line_len),
                kind: SpanKind::Nothing,
            })
        } else if current == 0 {
            self.cursor = start;

            Some(SelectionSpan {
                slice: self.line.slice(current..start),
                kind: SpanKind::Nothing,
            })
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

    let overlaps = start <= line_idx + max_len && line_idx <= end;

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
        let len = text.len_chars();

        let mut selection = Selection::new(0);
        selection.update(len);

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
                slice: RopeSlice::from("t t")
            })
        );

        assert_eq!(
            iter.next(),
            Some(SelectionSpan {
                kind: SpanKind::Nothing,
                slice: RopeSlice::from("est")
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
        let max_len = line.len_chars();
        let spans = selection_spans(line_idx, max_len, line, selection);

        assert_eq!(spans.len(), 2);

        assert_eq!(
            spans[0],
            SelectionSpan {
                kind: SpanKind::Nothing,
                slice: RopeSlice::from("tes"),
            }
        );

        assert_eq!(
            spans[1],
            SelectionSpan {
                kind: SpanKind::Selection,
                slice: RopeSlice::from("t test\n"),
            }
        );

        let line = text.line(1);
        let line_idx = text.line_to_char(1);
        let max_len = line.len_chars();
        let spans = selection_spans(line_idx, max_len, line, selection);

        assert_eq!(spans.len(), 2);

        assert_eq!(
            spans[0],
            SelectionSpan {
                kind: SpanKind::Selection,
                slice: RopeSlice::from("test"),
            }
        );

        assert_eq!(
            spans[1],
            SelectionSpan {
                kind: SpanKind::Nothing,
                slice: RopeSlice::from(" line 2"),
            }
        );
    }
}
