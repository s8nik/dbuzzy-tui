use crate::{
    editor::Workspace,
    input::{Event, Input, Modifiers},
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
        } => super::adjustment::insert_char(ws, ch),
        Input {
            event: Event::Esc, ..
        } => super::switch_mode::normal_mode_inplace(ws),
        Input {
            event: Event::Left, ..
        } => super::movement::move_left(ws),
        Input {
            event: Event::Right,
            ..
        } => super::movement::move_right(ws),
        Input {
            event: Event::Up, ..
        } => super::movement::move_up(ws),
        Input {
            event: Event::Down, ..
        } => super::movement::move_down(ws),
        Input {
            event: Event::Backspace,
            ..
        } => super::adjustment::delete_char(ws),
        Input {
            event: Event::Enter,
            ..
        } => super::adjustment::new_line(ws),
        Input {
            event: Event::PageUp,
            ..
        } => super::movement::go_to_top_line(ws),
        Input {
            event: Event::PageDown,
            ..
        } => super::movement::go_to_bottom_line(ws),
        _ => outcome = EventOutcome::Ignore,
    }

    outcome
}
