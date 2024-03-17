use crate::{
    buffer::CursorMode,
    editor::Workspace,
    input::{Event, Input, Modifiers},
    renderer::EventOutcome,
};

pub fn on_key(workspace: &mut Workspace, input: Input) -> EventOutcome {
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
            super::edit::insert_char(workspace, ch);
        }
        Input {
            event: Event::Esc, ..
        } => workspace.curr_mut().buf_mut().mode = CursorMode::Normal,
        Input {
            event: Event::Left, ..
        } => super::shift::move_left(workspace),
        Input {
            event: Event::Right,
            ..
        } => super::shift::move_right(workspace),
        Input {
            event: Event::Up, ..
        } => super::shift::move_up(workspace),
        Input {
            event: Event::Down, ..
        } => super::shift::move_down(workspace),
        Input {
            event: Event::Backspace,
            ..
        } => super::edit::delete_char(workspace),
        Input {
            event: Event::Enter,
            ..
        } => super::edit::new_line(workspace),
        Input {
            event: Event::PageUp,
            ..
        } => super::shift::go_to_top_line(workspace),
        Input {
            event: Event::PageDown,
            ..
        } => super::shift::go_to_bottom_line(workspace),
        _ => outcome = EventOutcome::Ignore,
    }

    outcome
}
