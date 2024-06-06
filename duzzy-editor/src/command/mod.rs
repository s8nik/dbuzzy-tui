mod clip;
mod input;
mod modify;
mod motion;
mod revert;
mod search;
mod select;
mod switch;

use std::{collections::HashMap, sync::Arc};

pub use input::on_key as input_on_key;
pub use search::on_key as search_on_key;

use clip::*;
use modify::*;
use motion::*;
use revert::{redo, undo};
use search::*;
use select::*;
use switch::*;

use crate::{
    buffer::Buffer,
    editor::Workspace,
    event::Input,
    keymap::{Keymap, Keymaps},
};

pub type Callback = fn(&mut Workspace);

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, PartialOrd)]
pub enum CmdType {
    InsertMode,
    MoveLeft,
    MoveDown,
    MoveUp,
    MoveRight,
    MoveNextWordEnd,
    MoveNextWordStart,
    MovePrevWordStart,
    InsertModeLineEnd,
    InsertModeLineStart,
    InsertModeLineNext,
    InsertModeLinePrev,
    Delete,
    GoToTopLine,
    GoToBottomLine,
    GoToLineStart,
    GoToLineEnd,
    Undo,
    Redo,
    NormalMode,
    VisualMode,
    SelectLine,
    CopyLocal,
    CopyGlobal,
    PasteLocal,
    PasteGlobal,
    SearchMode,
    SearchNext,
    SearchPrev,
}

pub struct Command {
    type_: CmdType,
    callback: Callback,
}

impl Command {
    pub fn new(type_: CmdType, callback: Callback) -> Self {
        Self { type_, callback }
    }

    pub fn call(&self, content: &mut Workspace) {
        (self.callback)(content);
    }
}

pub struct CommandRegistry {
    commands: HashMap<CmdType, Arc<Command>>,
}

impl CommandRegistry {
    pub fn register() -> Self {
        let commands = vec![
            Command::new(CmdType::InsertMode, insert_mode_inplace),
            Command::new(CmdType::MoveLeft, move_left),
            Command::new(CmdType::MoveDown, move_down),
            Command::new(CmdType::MoveUp, move_up),
            Command::new(CmdType::MoveRight, move_right),
            Command::new(CmdType::MoveNextWordEnd, move_next_word_end),
            Command::new(CmdType::MoveNextWordStart, move_next_word_start),
            Command::new(CmdType::MovePrevWordStart, move_prev_word_start),
            Command::new(CmdType::InsertModeLineEnd, insert_mode_line_end),
            Command::new(CmdType::InsertModeLineStart, insert_mode_line_start),
            Command::new(CmdType::InsertModeLineNext, insert_mode_line_next),
            Command::new(CmdType::InsertModeLinePrev, insert_mode_line_prev),
            Command::new(CmdType::Delete, delete),
            Command::new(CmdType::GoToTopLine, go_to_top_line),
            Command::new(CmdType::GoToBottomLine, go_to_bottom_line),
            Command::new(CmdType::GoToLineEnd, go_to_line_end),
            Command::new(CmdType::GoToLineStart, go_to_line_start),
            Command::new(CmdType::Undo, undo),
            Command::new(CmdType::Redo, redo),
            Command::new(CmdType::VisualMode, visual_mode),
            Command::new(CmdType::NormalMode, normal_mode),
            Command::new(CmdType::SelectLine, select_line),
            Command::new(CmdType::CopyLocal, copy_local),
            Command::new(CmdType::CopyGlobal, copy_global),
            Command::new(CmdType::PasteLocal, paste_local),
            Command::new(CmdType::PasteGlobal, paste_global),
            Command::new(CmdType::SearchMode, search_mode),
            Command::new(CmdType::SearchNext, search_next),
            Command::new(CmdType::SearchPrev, search_prev),
        ];

        let mut map = HashMap::new();
        for command in commands {
            map.insert(command.type_, Arc::new(command));
        }

        Self { commands: map }
    }

    pub fn get(&self, type_: &CmdType) -> Option<Arc<Command>> {
        self.commands.get(type_).cloned()
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::register()
    }
}

#[derive(Default)]
pub struct CommandFinder {
    registry: CommandRegistry,
    current: Option<&'static Keymap>,
}

impl CommandFinder {
    pub fn reset(&mut self) {
        self.current = None;
    }

    pub fn find(
        &mut self,
        keymaps: &'static Keymaps,
        buffer: &Buffer,
        input: Input,
    ) -> Option<Arc<Command>> {
        let bindings = keymaps.get(&buffer.mode())?;

        self.current = match self.current {
            Some(node) => match node {
                Keymap::Leaf(_) => self.current,
                Keymap::Node(next) => next.get(input),
            },
            None => bindings.get(input),
        };

        if let Some(Keymap::Leaf(command)) = self.current {
            return self.registry.get(command);
        }

        None
    }
}
