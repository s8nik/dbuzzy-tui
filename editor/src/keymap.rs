use std::{
    collections::{BTreeMap, HashMap},
    ops::Deref,
};

use crate::{
    buffer::CursorMode,
    command::CmdType,
    input::{Event, Input, Modifiers},
};

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Default)]
pub struct Bindings(BTreeMap<Input, Keymap>);

impl Bindings {
    pub fn get(&self, input: Input) -> Option<&Keymap> {
        self.0.get(&input)
    }
}

impl From<Vec<(&str, CmdType)>> for Bindings {
    fn from(mappings: Vec<(&str, CmdType)>) -> Self {
        let mut bindings = Bindings::default();
        for (sequence, command_type) in mappings {
            Keymaps::parse(&mut bindings, sequence, command_type);
        }

        bindings
    }
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug)]
pub enum Keymap {
    Leaf(CmdType),
    Node(Bindings),
}

#[derive(Debug, Default)]
pub struct Keymaps(HashMap<CursorMode, Bindings>);

impl Keymaps {
    pub fn get(&self, mode: &CursorMode) -> Option<&Bindings> {
        self.0.get(mode)
    }
}

impl Keymaps {
    pub fn init() -> &'static Self {
        let mut map = HashMap::<CursorMode, Bindings>::new();

        map.insert(CursorMode::Normal, Self::normal_mode());

        Box::leak(Box::new(Keymaps(map)))
    }

    fn normal_mode() -> Bindings {
        let mappings = vec![
            ("i", CmdType::InsertMode),
            ("h", CmdType::MoveLeft),
            ("j", CmdType::MoveDown),
            ("k", CmdType::MoveUp),
            ("l", CmdType::MoveRight),
            ("A", CmdType::InsertModeLineEnd),
            ("I", CmdType::InsertModeLineStart),
            ("o", CmdType::InsertModeLineNext),
            ("O", CmdType::InsertModeLinePrev),
            ("d", CmdType::DeleteChar),
            ("gg", CmdType::GoToTopLine),
            ("ge", CmdType::GoToBottomLine),
            ("gl", CmdType::GoToLineEnd),
            ("gh", CmdType::GoToLineStart),
        ];

        mappings.into()
    }

    fn parse(root: &mut Bindings, sequence: &str, command_type: CmdType) {
        let re = regex::Regex::new(r"<(.*?)>").expect("valid pattern");

        let mut specials: Vec<String> = re
            .captures_iter(sequence)
            .map(|capture| capture[1].to_string())
            .collect();

        let modifiers: Modifiers = specials.deref().into();
        specials.retain(|x| !Modifiers::contain(&x.to_lowercase()));

        let mut keys: Vec<String> = re
            .replace_all(sequence, "")
            .chars()
            .map(|c| c.to_string())
            .collect();

        keys.extend(specials);
        keys.sort_by(|a, b| {
            let pos_a = sequence.find(a).unwrap_or(usize::MAX);
            let pos_b = sequence.find(b).unwrap_or(usize::MAX);
            pos_a.cmp(&pos_b)
        });
        keys.reverse();

        Self::parse_keys(root, modifiers, keys, command_type);
    }

    fn parse_keys(
        parent: &mut Bindings,
        modifiers: Modifiers,
        mut keys: Vec<String>,
        command_type: CmdType,
    ) {
        let Some(key) = keys.pop() else {
            return;
        };

        let event: Event = match key.as_str().try_into() {
            Ok(e) => e,
            Err(e) => {
                log::error!("parse keys error: {e}");
                return;
            }
        };

        let input = Input { event, modifiers };

        if keys.is_empty() {
            parent.0.insert(input, Keymap::Leaf(command_type));
        } else {
            if let Some(Keymap::Node(ref mut child)) = parent.0.get_mut(&input) {
                return Self::parse_keys(child, modifiers, keys, command_type);
            }

            let mut child = Bindings::default();
            Self::parse_keys(&mut child, modifiers, keys, command_type);
            parent.0.insert(input, Keymap::Node(child));
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::command::CmdType;

    #[test]
    fn test_keymap() {
        let keymap = super::Keymaps::init();

        let normal = keymap.get(&super::CursorMode::Normal).unwrap();

        let node = normal
            .get(super::Input {
                event: super::Event::Char('g'),
                ..Default::default()
            })
            .unwrap();

        let super::Keymap::Node(bindings) = node else {
            panic!("failed");
        };

        let leaf = bindings
            .get(super::Input {
                event: super::Event::Char('e'),
                ..Default::default()
            })
            .unwrap();

        let expected = super::Keymap::Leaf(CmdType::GoToBottomLine);
        assert_eq!(leaf, &expected);
    }
}
