use std::collections::{BTreeMap, HashMap};

use crate::{event::Event, mode::CursorMode};

pub struct Keymap(BTreeMap<Event, KeymapNode>);

pub enum KeymapNode {
    Leaf(String),
    Node(Keymap),
}

#[derive(Default)]
pub struct Keymaps(HashMap<CursorMode, Keymap>);

impl Keymap {
    pub fn get(&self, event: Event) -> Option<&KeymapNode> {
        self.0.get(&event)
    }
}

impl Keymaps {
    //TODO: refactoring
    pub fn init() -> Self {
        let mut maps = HashMap::new();
        let mut nmap = BTreeMap::new();

        nmap.insert(Event::Char('i'), KeymapNode::Leaf("insert_mode".to_owned()));
        nmap.insert(
            Event::Char('h'),
            KeymapNode::Leaf("move_cursor_back".to_owned()),
        );
        nmap.insert(
            Event::Char('j'),
            KeymapNode::Leaf("move_cursor_down".to_owned()),
        );
        nmap.insert(
            Event::Char('k'),
            KeymapNode::Leaf("move_cursor_up".to_owned()),
        );
        nmap.insert(
            Event::Char('l'),
            KeymapNode::Leaf("move_cursor_forward".to_owned()),
        );

        maps.insert(CursorMode::Normal, Keymap(nmap));
        Keymaps(maps)
    }

    pub fn get(&self, mode: CursorMode) -> Option<&Keymap> {
        self.0.get(&mode)
    }
}
