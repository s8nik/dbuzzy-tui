use std::{
    collections::{BTreeMap, HashMap},
    ops::Deref,
    str::FromStr,
};

use crate::{
    command::Command,
    event::{Event, Input, Modifiers},
    mode::CursorMode,
};

#[derive(Debug, Default)]
pub struct Bindings(BTreeMap<Input, Keymap>);

#[derive(Debug)]
pub enum Keymap {
    Leaf(Command),
    Node(Bindings),
}

#[derive(Debug, Default)]
pub struct Keymaps(HashMap<CursorMode, Bindings>);

impl Bindings {
    pub fn get(&self, input: Input) -> Option<&Keymap> {
        self.0.get(&input)
    }
}

impl Keymaps {
    pub fn init() -> &'static Self {
        let bytes = include_bytes!("default");
        let config = String::from_utf8_lossy(bytes);

        let mut map = HashMap::<CursorMode, Bindings>::new();
        for (i, line) in config.lines().enumerate() {
            let content = line.splitn(2, "#").next().unwrap_or(line);

            if content.is_empty() {
                continue;
            }

            let (definition, command) = split_once(content, ':', i);
            let (mode, sequence) = split_once(definition, ' ', i);

            let cursor = CursorMode::from_str(mode).expect("valid cursor mode");
            if !map.contains_key(&cursor) {
                map.insert(cursor, Bindings::default());
            }

            let root = map.get_mut(&cursor).expect("root");
            Self::parse(root, sequence, command);
        }

        dbg!(map);
        unimplemented!();
    }

    fn parse(root: &mut Bindings, sequence: &str, command: &str) {
        let target = sequence.to_lowercase();
        let re = regex::Regex::new(r"<(.*?)>").expect("valid pattern");

        let mut specials: Vec<String> = re
            .captures_iter(&target)
            .map(|capture| capture[1].to_string())
            .collect();

        let modifiers: Modifiers = specials.deref().into();
        specials.retain(|x| !Modifiers::contain(x));

        let mut sequence: Vec<String> = re
            .replace_all(&target, "")
            .chars()
            .map(|c| c.to_string())
            .collect();

        sequence.extend(specials);
        sequence.sort_by(|a, b| {
            let pos_a = target.find(a).unwrap_or(usize::MAX);
            let pos_b = target.find(b).unwrap_or(usize::MAX);
            pos_a.cmp(&pos_b)
        });
        sequence.reverse();

        Self::parse_sequence(root, modifiers, sequence, command);
    }

    fn parse_sequence(
        parent: &mut Bindings,
        modifiers: Modifiers,
        mut sequence: Vec<String>,
        command: &str,
    ) {
        let Some(key) = sequence.pop() else {
            return;
        };

        let event: Event = match key.as_str().try_into() {
            Ok(e) => e,
            Err(e) => {
                // @todo: better loggs
                return;
            }
        };

        let input = Input { event, modifiers };

        if sequence.is_empty() {
            let command = Command::from_str(command).expect("unsupported command");
            parent.0.insert(input, Keymap::Leaf(command));
        } else {
            if let Some(Keymap::Node(ref mut child)) = parent.0.get_mut(&input) {
                return Self::parse_sequence(child, modifiers, sequence, command);
            }

            let mut child = Bindings::default();
            Self::parse_sequence(&mut child, modifiers, sequence, command);
            parent.0.insert(input, Keymap::Node(child));
        }
    }
}

fn split_once(slice: &str, sep: char, i: usize) -> (&str, &str) {
    let Some((first, second)) = slice.split_once(sep) else {
        panic!("invalid format at line: {}", i);
    };

    (first.trim(), second.trim())
}
