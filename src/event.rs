use anyhow::Context;
use crossterm::event::{
    Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind,
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Modifiers {
    shift: bool,
    control: bool,
    alt: bool,
    sup: bool,
    hyper: bool,
    meta: bool,
}

impl From<KeyModifiers> for Modifiers {
    fn from(modifiers: KeyModifiers) -> Self {
        Self {
            shift: modifiers.contains(KeyModifiers::SHIFT),
            control: modifiers.contains(KeyModifiers::CONTROL),
            alt: modifiers.contains(KeyModifiers::ALT),
            sup: modifiers.contains(KeyModifiers::SUPER),
            hyper: modifiers.contains(KeyModifiers::HYPER),
            meta: modifiers.contains(KeyModifiers::META),
        }
    }
}

impl From<&[String]> for Modifiers {
    fn from(values: &[String]) -> Self {
        let mut modifiers = Modifiers::default();

        for name in values {
            modifiers.set_by(&name, true);
        }

        modifiers
    }
}

impl Modifiers {
    const NAMES: [&str; 6] = ["shift", "ctr", "alt", "super", "hyper", "meta"];

    pub fn contain(name: &str) -> bool {
        Self::NAMES.contains(&name)
    }

    pub fn set_by(&mut self, name: &str, value: bool) {
        let Some(position) = Self::NAMES.iter().position(|x| *x == name) else {
            return;
        };

        match position {
            0 => self.shift = value,
            1 => self.control = value,
            2 => self.alt = value,
            3 => self.sup = value,
            4 => self.hyper = value,
            5 => self.meta = value,
            _ => (),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum Event {
    Char(char),
    Backspace,
    Enter,
    Left,
    Right,
    Up,
    Down,
    Tab,
    Delete,
    Home,
    End,
    PageUp,
    PageDown,
    Esc,

    MouseScrollDown,
    MouseScrollUp,

    #[default]
    Null,
}

impl TryFrom<&str> for Event {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let event = if value.len() == 1 {
            let c = value.chars().next().with_context(|| "must exist")?;
            Self::Char(c)
        } else {
            match value {
                "backspace" => Self::Backspace,
                "enter" => Self::Enter,
                "left" => Self::Left,
                "right" => Self::Right,
                "up" => Self::Up,
                "down" => Self::Down,
                "tab" => Self::Tab,
                "delete" => Self::Delete,
                "home" => Self::Home,
                "end" => Self::End,
                "pageup" => Self::PageUp,
                "pagedown" => Self::PageDown,
                "esc" => Self::Esc,
                other => anyhow::bail!("unsupported event: {}", other),
            }
        };

        Ok(event)
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Input {
    pub event: Event,
    pub modifiers: Modifiers,
}

impl From<CrosstermEvent> for Input {
    fn from(event: CrosstermEvent) -> Self {
        match event {
            CrosstermEvent::Key(key) => Self::from(key),
            CrosstermEvent::Mouse(mouse) => Self::from(mouse),
            _ => Self::default(),
        }
    }
}

impl From<KeyEvent> for Input {
    fn from(key: KeyEvent) -> Self {
        let event = match key.code {
            KeyCode::Char(c) => Event::Char(c),
            KeyCode::Backspace => Event::Backspace,
            KeyCode::Enter => Event::Enter,
            KeyCode::Left => Event::Left,
            KeyCode::Right => Event::Right,
            KeyCode::Up => Event::Up,
            KeyCode::Down => Event::Down,
            KeyCode::Tab => Event::Tab,
            KeyCode::Delete => Event::Delete,
            KeyCode::Home => Event::Home,
            KeyCode::End => Event::End,
            KeyCode::PageUp => Event::PageUp,
            KeyCode::PageDown => Event::PageDown,
            KeyCode::Esc => Event::Esc,
            _ => Event::Null,
        };
        let modifiers = key.modifiers.into();
        Self { event, modifiers }
    }
}

impl From<MouseEvent> for Input {
    fn from(mouse: MouseEvent) -> Self {
        let event = match mouse.kind {
            MouseEventKind::ScrollDown => Event::MouseScrollDown,
            MouseEventKind::ScrollUp => Event::MouseScrollUp,
            _ => return Self::default(),
        };
        let modifiers = mouse.modifiers.into();
        Self { event, modifiers }
    }
}
