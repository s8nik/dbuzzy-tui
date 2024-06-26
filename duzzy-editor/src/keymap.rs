use std::{
    collections::{BTreeMap, HashMap},
    ops::Deref,
};

use duzzy_lib::event::{Input, Modifiers};
use once_cell::sync::OnceCell;

use crate::{buffer::Mode, command::CmdType};

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Default)]
pub struct Bindings(BTreeMap<Input, Keymap>);

impl Bindings {
    pub fn get(&self, input: &Input) -> Option<&Keymap> {
        self.0.get(input)
    }
}

impl From<Vec<(&str, CmdType)>> for Bindings {
    fn from(mappings: Vec<(&str, CmdType)>) -> Self {
        let mut bindings = Self::default();
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
pub struct Keymaps(HashMap<Mode, Bindings>);

impl Keymaps {
    pub fn init() -> &'static Self {
        static KEYMAPS: OnceCell<Keymaps> = OnceCell::new();
        KEYMAPS.get_or_init(|| {
            let mut map = HashMap::<Mode, Bindings>::new();

            map.insert(Mode::Normal, Self::normal_mode());
            map.insert(Mode::Visual, Self::visual_mode());

            Self(map)
        })
    }

    pub fn get(&self, mode: &Mode) -> Option<&Bindings> {
        self.0.get(mode)
    }

    fn common_bindings() -> Vec<(&'static str, CmdType)> {
        vec![
            ("h", CmdType::MoveLeft),
            ("j", CmdType::MoveDown),
            ("k", CmdType::MoveUp),
            ("l", CmdType::MoveRight),
            ("d", CmdType::Delete),
            ("gg", CmdType::GoToTopLine),
            ("ge", CmdType::GoToBottomLine),
            ("gl", CmdType::GoToLineEnd),
            ("gh", CmdType::GoToLineStart),
            ("w", CmdType::MoveNextWordStart),
            ("e", CmdType::MoveNextWordEnd),
            ("b", CmdType::MovePrevWordStart),
            ("x", CmdType::SelectLine),
            ("y", CmdType::CopyLocal),
            ("<Space>y", CmdType::CopyGlobal),
            ("p", CmdType::PasteLocal),
            ("<Space>p", CmdType::PasteGlobal),
            ("/", CmdType::SearchMode),
            ("n", CmdType::SearchNext),
            ("N", CmdType::SearchPrev),
        ]
    }

    fn normal_mode() -> Bindings {
        let mut bindings = vec![
            ("i", CmdType::InsertMode),
            ("A", CmdType::InsertModeLineEnd),
            ("I", CmdType::InsertModeLineStart),
            ("o", CmdType::InsertModeLineNext),
            ("O", CmdType::InsertModeLinePrev),
            ("u", CmdType::Undo),
            ("U", CmdType::Redo),
            ("v", CmdType::VisualMode),
        ];

        bindings.extend(Self::common_bindings());
        bindings.into()
    }

    fn visual_mode() -> Bindings {
        let mut bindings = vec![("<Esc>", CmdType::NormalMode)];
        bindings.extend(Self::common_bindings());
        bindings.into()
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

        let Ok(event) = key.as_str().try_into() else {
            //@todo: better logs
            return;
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
    use duzzy_lib::event::Event;

    use crate::{buffer::Mode, command::CmdType};

    #[test]
    fn test_keymap() {
        let keymap = super::Keymaps::init();
        let normal = keymap.get(&Mode::Normal).unwrap();

        let node = normal
            .get(&super::Input {
                event: Event::Char('g'),
                ..Default::default()
            })
            .unwrap();

        let super::Keymap::Node(bindings) = node else {
            panic!("failed");
        };

        let leaf = bindings
            .get(&super::Input {
                event: Event::Char('e'),
                ..Default::default()
            })
            .unwrap();

        let expected = super::Keymap::Leaf(CmdType::GoToBottomLine);
        assert_eq!(leaf, &expected);
    }
}
