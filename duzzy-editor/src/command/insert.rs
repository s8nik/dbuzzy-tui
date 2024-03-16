use crate::{
    buffer::CursorMode,
    editor::Workspace,
    input::{Event, Input, Modifiers},
    renderer::EventOutcome,
};

pub fn on_key(editor: &mut Workspace, input: Input) -> EventOutcome {
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
            super::edit::insert_char(editor, ch);
        }
        Input {
            event: Event::Esc, ..
        } => editor.curr_mut().buf_mut().mode = CursorMode::Normal,
        Input {
            event: Event::Left, ..
        } => super::shift::move_left(editor),
        Input {
            event: Event::Right,
            ..
        } => super::shift::move_right(editor),
        Input {
            event: Event::Up, ..
        } => super::shift::move_up(editor),
        Input {
            event: Event::Down, ..
        } => super::shift::move_down(editor),
        Input {
            event: Event::Backspace,
            ..
        } => super::edit::delete_char(editor),
        Input {
            event: Event::Enter,
            ..
        } => super::edit::new_line(editor),
        Input {
            event: Event::PageUp,
            ..
        } => super::shift::go_to_top_line(editor),
        Input {
            event: Event::PageDown,
            ..
        } => super::shift::go_to_bottom_line(editor),
        _ => outcome = EventOutcome::Ignore,
    }

    outcome
}
