use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Paragraph, Widget},
};
use ropey::RopeSlice;

use crate::{
    buffer::Mode,
    editor::Editor,
    selection::{selection_spans, SelectedRange, SpanKind},
};

#[derive(Default, Copy, Clone)]
pub struct Viewport {
    pub width: usize,
    pub height: usize,
}

pub struct Cursor {
    pub x: u16,
    pub y: u16,
    pub mode: Mode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventOutcome {
    Render,
    Ignore,
    Exit,
}

pub struct Renderer<'a> {
    editor: &'a Editor,
    status: StatusLine<'a>,
    theme: Theme,
}

impl<'a> Renderer<'a> {
    pub fn new(editor: &'a Editor) -> Self {
        let theme = Theme::default();

        let workspace = &editor.workspace;
        let mode = workspace.cur().buf().mode();
        let search_pattern = workspace.search_registry().buffer();

        let status = StatusLine::new(mode, search_pattern);

        Self {
            editor,
            status,
            theme,
        }
    }

    fn update_viewport(&self, width: u16, height: u16) {
        let mut viewport = self.editor.viewport.borrow_mut();

        viewport.width = width as _;
        viewport.height = height as _;
    }

    fn line(
        &self,
        line_idx: usize,
        max_len: usize,
        line: RopeSlice<'a>,
        selection: Option<SelectedRange>,
    ) -> Line<'_> {
        let default_line = |line: RopeSlice<'a>| Line::raw(line).style(self.theme.text_style);

        let Some(range) = selection else {
            return default_line(line);
        };

        if range.0 == range.1 {
            return default_line(line);
        }

        let span_style = |kind: SpanKind| match kind {
            SpanKind::Nothing => self.theme.text_style,
            SpanKind::Selection => self.theme.selection_style,
        };

        let spans = selection_spans(line_idx, max_len, line, range)
            .into_iter()
            .map(|s| Span::styled(s.slice, span_style(s.kind)))
            .collect::<Vec<_>>();

        if !spans.is_empty() {
            Line::from(spans)
        } else {
            default_line(line)
        }
    }

    #[inline]
    pub fn text(&self) -> Option<Text> {
        let buf = self.editor.workspace.cur().buf();

        let text = buf.text();
        let viewport = self.editor.viewport.borrow();
        let selection = buf.selection().map(|s| s.range());

        let vscroll = buf.vscroll();
        let max_y = viewport.height.min(text.len_lines());

        let mut lines = Vec::with_capacity(max_y);
        for y in 0..max_y {
            let index = y + vscroll;
            let line = text.line(index);

            let line_idx = text.line_to_byte(index);
            let max_len = viewport.width.min(line.len_chars().saturating_sub(1));

            lines.push(self.line(line_idx, max_len, line, selection));
        }

        Some(Text::from(lines))
    }
}

impl Widget for Renderer<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        buf.set_style(area, self.theme.base_style);

        let [main, status] =
            Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(area);

        self.update_viewport(main.width, main.height);

        if let Some(text) = self.text() {
            let inner = Paragraph::new(text);
            inner.render(main, buf);
        }

        let cursor = self.editor.cursor();
        buf.get_mut(cursor.x, cursor.y)
            .set_style(self.theme.cursor_style);

        self.status.render(status, buf);
    }
}

pub struct Theme {
    pub base_style: Style,
    pub text_style: Style,
    pub cursor_style: Style,
    pub selection_style: Style,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            base_style: Style::default().bg(color::RICH_BLACK),
            text_style: Style::default().fg(color::LAVENDER),
            cursor_style: Style::default().bg(color::COOL_GRAY),
            selection_style: Style::default().bg(color::COOL_GRAY),
        }
    }
}

pub struct StatusLine<'a> {
    mode: Mode,
    search_pattern: &'a str,
    line_style: Style,
    text_style: Style,
}

impl<'a> StatusLine<'a> {
    fn new(mode: Mode, search_pattern: &'a str) -> Self {
        Self {
            mode,
            search_pattern,
            line_style: Style::default().fg(color::LAVENDER).bg(color::COOL_GRAY),
            text_style: Style::default().fg(color::RICH_BLACK).bg(color::LAVENDER),
        }
    }
}

impl Widget for StatusLine<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let constraints = [Constraint::Length(10), Constraint::Min(0)];
        let [left, right] = Layout::horizontal(constraints).areas(area);

        let mode_paragraph = Paragraph::new(self.mode.as_ref())
            .centered()
            .style(self.text_style);

        let search_paragraph = Paragraph::new(self.search_pattern)
            .left_aligned()
            .style(self.line_style);

        mode_paragraph.render(left, buf);
        search_paragraph.render(right, buf);
    }
}

pub(crate) mod color {
    use super::Color;

    pub const LAVENDER: Color = Color::Rgb(238, 238, 255);
    pub const RICH_BLACK: Color = Color::Rgb(17, 21, 28);
    pub const COOL_GRAY: Color = Color::Rgb(127, 124, 175);
}
