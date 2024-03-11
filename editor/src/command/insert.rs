use crate::{
    buffer::{Buffer, CursorMode},
    input::{Event, Input, Modifiers},
    renderer::EventOutcome,
};

use super::shift::Shift;

pub fn on_key(buffer: &mut Buffer, input: Input) -> EventOutcome {
    if let Input {
        event: Event::Char('q'),
        modifiers: Modifiers { ctr: true, .. },
    } = input
    {
        return EventOutcome::Exit;
    }

    let mut outcome = EventOutcome::Render;

    match input {
        Input {
            event: Event::Char(ch),
            ..
        } => {
            super::edit::insert_char(buffer, ch);
        }
        Input {
            event: Event::Esc, ..
        } => buffer.update_cursor_mode(CursorMode::Normal),
        Input {
            event: Event::Left, ..
        } => super::shift::shift_cursor(buffer, Shift::Left),
        Input {
            event: Event::Right,
            ..
        } => super::shift::shift_cursor(buffer, Shift::Right),
        Input {
            event: Event::Up, ..
        } => super::shift::shift_cursor(buffer, Shift::Up(1)),
        Input {
            event: Event::Down, ..
        } => super::shift::shift_cursor(buffer, Shift::Down(1)),
        Input {
            event: Event::Backspace,
            ..
        } => super::edit::backspace(buffer),
        Input {
            event: Event::Enter,
            ..
        } => super::edit::new_line(buffer),
        Input {
            event: Event::PageUp,
            ..
        } => super::shift::shift_cursor(buffer, Shift::Top),
        Input {
            event: Event::PageDown,
            ..
        } => super::shift::shift_cursor(buffer, Shift::Bottom),
        _ => outcome = EventOutcome::Ignore,
    }

    outcome
}
