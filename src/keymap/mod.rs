use std::{
    collections::{BTreeMap, HashMap},
    str::Chars,
};

use serde_yaml::Value;

use crate::{event::Event, mode::CursorMode};

#[derive(Debug)]
pub struct Keymap(BTreeMap<Event, KeymapNode>);

#[derive(Debug)]
pub enum KeymapNode {
    Leaf(String),
    Node(Keymap),
}

#[derive(Debug, Default)]
pub struct Keymaps(HashMap<CursorMode, Keymap>);

impl Keymap {
    pub fn get(&self, event: Event) -> Option<&KeymapNode> {
        self.0.get(&event)
    }
}

impl Keymaps {
    pub fn init() -> &'static Self {
        let raw = include_bytes!("default.yml");
        let maps: HashMap<CursorMode, Value> = serde_yaml::from_slice(raw).expect("default keymap");

        let mut keymaps: HashMap<CursorMode, Keymap> = HashMap::new();

        for (mode, value) in maps {
            if let Value::Mapping(map) = value {
                let mut root = BTreeMap::new();

                for (k, v) in map {
                    match (k, v) {
                        (Value::String(keys), Value::String(command)) => {
                            parse_map(&mut root, keys.split("_").collect(), command);
                        }
                        _ => continue,
                    }
                }

                keymaps.insert(mode, Keymap(root));
            }
        }

        fn parse_map(
            parent: &mut BTreeMap<Event, KeymapNode>,
            mut keys: Vec<&str>,
            command: String,
        ) {
            if keys.len() == 0 {
                return;
            }

            let key = keys.remove(0);
            let event = Event::try_from(key).expect("valid event!");

            if keys.len() == 0 {
                parent.insert(event, KeymapNode::Leaf(command));
            } else {
                if let Some(existing) = parent.get_mut(&event) {
                    if let KeymapNode::Node(ref mut child) = existing {
                        return parse_map(&mut child.0, keys, command);
                    }
                }

                let mut child = BTreeMap::new();
                parse_map(&mut child, keys, command);
                parent.insert(event, KeymapNode::Node(Keymap(child)));
            }
        }

        dbg!(&keymaps);
        unimplemented!();

        Box::leak(Box::new(Keymaps(keymaps)))
    }

    pub fn get(&self, mode: CursorMode) -> Option<&Keymap> {
        self.0.get(&mode)
    }
}
