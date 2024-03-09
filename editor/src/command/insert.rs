use crate::{buffer::Buffer, input::Input, renderer::EventOutcome};

pub fn on_key(buffer: &mut Buffer, input: Input) -> EventOutcome {
    match input {
        Input {
            event: crate::input::Event::Char('q'),
            modifiers: crate::input::Modifiers { ctr: true, .. },
        } => EventOutcome::Exit,
        Input {
            event: crate::input::Event::Char(ch),
            modifiers: _,
        } => {
            super::transform::insert_char(buffer, ch);
            EventOutcome::Render
        }
        _ => EventOutcome::Ignore,
    }
}
