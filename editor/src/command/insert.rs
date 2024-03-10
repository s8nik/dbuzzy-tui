use crate::{
    buffer::{Buffer, CursorMode},
    input::{Event, Input, Modifiers},
    renderer::EventOutcome,
};

use super::movement::CursorMove;

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
            super::transform::insert_char(buffer, ch);
        }
        Input {
            event: Event::Esc, ..
        } => buffer.update_cursor_mode(CursorMode::Normal),
        Input {
            event: Event::Left, ..
        } => super::movement::move_cursor(buffer, CursorMove::Left),
        Input {
            event: Event::Right,
            ..
        } => super::movement::move_cursor(buffer, CursorMove::Right),
        Input {
            event: Event::Up, ..
        } => super::movement::move_cursor(buffer, CursorMove::Up(1)),
        Input {
            event: Event::Down, ..
        } => super::movement::move_cursor(buffer, CursorMove::Down(1)),
        Input {
            event: Event::Backspace,
            ..
        } => super::transform::backspace(buffer),
        Input {
            event: Event::Enter,
            ..
        } => super::transform::new_line(buffer),
        Input {
            event: Event::PageUp,
            ..
        } => super::movement::move_cursor(buffer, CursorMove::Top),
        Input {
            event: Event::PageDown,
            ..
        } => super::movement::move_cursor(buffer, CursorMove::Bottom),
        _ => outcome = EventOutcome::Ignore,
    }

    outcome
}
