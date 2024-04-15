use crate::{
    editor::Workspace,
    event::{Event, Input, Modifiers},
    renderer::EventOutcome,
};

pub fn on_key(ws: &mut Workspace, input: Input) -> EventOutcome {
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
        } => super::modify::insert_char(ws, ch),
        Input {
            event: Event::Esc, ..
        } => super::switch::normal_mode(ws),
        Input {
            event: Event::Left, ..
        } => super::motion::move_left(ws),
        Input {
            event: Event::Right,
            ..
        } => super::motion::move_right(ws),
        Input {
            event: Event::Up, ..
        } => super::motion::move_up(ws),
        Input {
            event: Event::Down, ..
        } => super::motion::move_down(ws),
        Input {
            event: Event::Backspace,
            ..
        } => super::modify::delete_backspace(ws),
        Input {
            event: Event::Enter,
            ..
        } => super::modify::new_line(ws),
        Input {
            event: Event::PageUp,
            ..
        } => super::motion::go_to_top_line(ws),
        Input {
            event: Event::PageDown,
            ..
        } => super::motion::go_to_bottom_line(ws),
        _ => outcome = EventOutcome::Ignore,
    }

    outcome
}
