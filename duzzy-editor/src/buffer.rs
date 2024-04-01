use ropey::Rope;

use crate::selection::Selection;

pub type Pos = (usize, usize);

#[derive(Debug, Default)]
pub struct Buffer {
    text: Rope,
    index: usize,
    offset: usize,
    vscroll: usize,
    mode: Mode,
    selection: Selection,
}

impl Buffer {
    pub const fn mode(&self) -> Mode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    pub const fn text(&self) -> &Rope {
        &self.text
    }

    pub fn text_mut(&mut self) -> &mut Rope {
        &mut self.text
    }

    pub fn set_text(&mut self, text: Rope) {
        self.text = text;
    }

    pub const fn index(&self) -> usize {
        self.index
    }

    pub fn set_index(&mut self, index: usize) {
        self.index = index;
    }

    pub const fn offset(&self) -> usize {
        self.offset
    }

    pub fn set_offset(&mut self, offset: usize) {
        self.offset = offset;
    }

    pub const fn pos(&self) -> Pos {
        (self.index, self.offset)
    }

    pub fn set_pos(&mut self, pos: Pos) {
        self.index = pos.0;
        self.offset = pos.1;
    }

    pub fn byte_pos(&self) -> usize {
        self.offset + self.text.line_to_byte(self.index)
    }

    pub fn curs_pos(&self, pos: usize) -> Pos {
        let index = self.text.byte_to_line(pos);
        let start = self.text.line_to_byte(index);
        let offset = pos - start;
        (index, offset)
    }

    pub const fn vscroll(&self) -> usize {
        self.vscroll
    }

    pub fn update_vscroll(&mut self, max: usize) {
        let upper_bound = self.vscroll + max - 1;

        if self.index < self.vscroll {
            self.vscroll = self.index;
        } else if self.index > upper_bound {
            self.vscroll = self.index - max + 1;
        }
    }

    pub fn selection(&self) -> Option<Selection> {
        (self.mode == Mode::Visual).then_some(self.selection)
    }

    pub fn update_selection(&mut self, pos: usize) {
        if self.mode == Mode::Visual {
            self.selection.update(pos);
        }
    }

    pub fn new_selection(&mut self, pos: usize) {
        self.selection = Selection::new(pos);
    }

    pub fn line_byte(&self, index: usize) -> usize {
        self.text.line_to_byte(index)
    }

    pub fn len_bytes(&self, index: usize) -> usize {
        self.text.line(index).len_bytes()
    }

    pub fn len_lines(&self) -> usize {
        self.text.len_lines()
    }

    pub fn len_chars(&self) -> usize {
        self.text.len_chars()
    }

    pub fn is_insert(&self) -> bool {
        self.mode == Mode::Insert
    }

    pub fn char(&self, pos: usize) -> char {
        self.text.char(pos)
    }
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Mode {
    #[default]
    Normal,
    Insert,
    Visual,
}
