use std::{
    collections::{BTreeMap, HashMap},
    str::FromStr,
};

use crate::{command::Command, event::Input, mode::CursorMode};

#[derive(Debug)]
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
        self.0.get(&iput)
    }
}

impl Keymaps {
    pub fn init() -> &'static Self {
        let bytes = include_bytes!("default");
        let config = String::from_utf8_lossy(bytes);

        for (i, line) in config.lines().enumerate() {
            let content = line.splitn(2, "#").next().unwrap_or(line);

            if content.is_empty() {
                continue;
            }

            let (definition, command) = split_once(content, ':', i);
            let (mode, sequence) = split_once(definition, ' ', i);

            println!("[{}], [{}]", mode, sequence);
        }

        unimplemented!();
    }

    fn parse(sequence: &str, command: &str) {}
}

fn split_once(slice: &str, sep: char, i: usize) -> (&str, &str) {
    let Some((first, second)) = slice.split_once(sep) else {
        panic!("invalid format at line: {}", i);
    };

    (first.trim(), second.trim())
}
