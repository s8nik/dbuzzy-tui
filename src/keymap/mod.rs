use std::{
    collections::{BTreeMap, HashMap},
    ops::Deref,
    str::FromStr,
};

use crate::{
    cursor::CursorMode,
    input::{Event, Input, Modifiers},
};

#[derive(Debug, Default)]
pub struct Bindings(BTreeMap<Input, Keymap>);

impl Bindings {
    pub fn get(&self, input: Input) -> Option<&Keymap> {
        self.0.get(&input)
    }
}

#[derive(Debug)]
pub enum Keymap {
    Leaf(String),
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
        let bytes = include_bytes!("default");
        let config = String::from_utf8_lossy(bytes);

        let mut map = HashMap::<CursorMode, Bindings>::new();
        for (i, line) in config.lines().enumerate() {
            let content = line.split('#').next().unwrap_or(line);

            if content.is_empty() {
                continue;
            }

            let (definition, command) = split_once(content, ':', i);
            let (mode, sequence) = split_once(definition, ' ', i);

            let cursor = CursorMode::from_str(mode).expect("valid cursor mode");
            map.entry(cursor).or_default();

            let root = map.get_mut(&cursor).expect("root");
            Self::parse(root, sequence, command);
        }

        Box::leak(Box::new(Keymaps(map)))
    }

    fn parse(root: &mut Bindings, sequence: &str, command: &str) {
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

        Self::parse_keys(root, modifiers, keys, command);
    }

    fn parse_keys(
        parent: &mut Bindings,
        modifiers: Modifiers,
        mut keys: Vec<String>,
        command: &str,
    ) {
        let Some(key) = keys.pop() else {
            return;
        };

        let event: Event = match key.as_str().try_into() {
            Ok(e) => e,
            Err(e) => {
                // @todo: better loggs
                println!("{e}");
                return;
            }
        };

        let input = Input { event, modifiers };

        if keys.is_empty() {
            parent.0.insert(input, Keymap::Leaf(command.to_owned()));
        } else {
            if let Some(Keymap::Node(ref mut child)) = parent.0.get_mut(&input) {
                return Self::parse_keys(child, modifiers, keys, command);
            }

            let mut child = Bindings::default();
            Self::parse_keys(&mut child, modifiers, keys, command);
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
