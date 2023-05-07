// The structure of this code was inspired by the example in tui-textarea (https://github.com/rhysd/tui-textarea/blob/main/src/input.rs).

use anyhow::Context;
#[cfg(feature = "crossterm")]
use crossterm::event::{
    Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind,
};

#[derive(Debug, Copy, Clone, Default, Eq, PartialEq, PartialOrd, Ord)]
pub enum Event {
    Char(char),
    F(u8),
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
            let c = value.chars().next().with_context(|| "char should exist!")?;
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
                other => anyhow::bail!("{other} event doesn't exist"),
            }
        };

        Ok(event)
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct Input {
    pub event: Event,
    pub ctrl: bool,
    pub alt: bool,
}

#[cfg(feature = "crossterm")]
impl From<CrosstermEvent> for Input {
    fn from(event: CrosstermEvent) -> Self {
        match event {
            CrosstermEvent::Key(key) => Self::from(key),
            CrosstermEvent::Mouse(mouse) => Self::from(mouse),
            _ => Self::default(),
        }
    }
}

#[cfg(feature = "crossterm")]
impl From<KeyEvent> for Input {
    fn from(key: KeyEvent) -> Self {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let alt = key.modifiers.contains(KeyModifiers::ALT);
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
            KeyCode::F(x) => Event::F(x),
            _ => Event::Null,
        };
        Self { event, ctrl, alt }
    }
}

#[cfg(feature = "crossterm")]
impl From<MouseEvent> for Input {
    fn from(mouse: MouseEvent) -> Self {
        let event = match mouse.kind {
            MouseEventKind::ScrollDown => Event::MouseScrollDown,
            MouseEventKind::ScrollUp => Event::MouseScrollUp,
            _ => return Self::default(),
        };
        let ctrl = mouse.modifiers.contains(KeyModifiers::CONTROL);
        let alt = mouse.modifiers.contains(KeyModifiers::ALT);
        Self { event, ctrl, alt }
    }
}
