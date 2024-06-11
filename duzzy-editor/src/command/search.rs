use crate::{
    editor::Workspace,
    event::{Event, Input},
    renderer::EventOutcome,
};

pub fn on_key(ws: &mut Workspace, input: Input) -> EventOutcome {
    let mut outcome = EventOutcome::Render;

    match input {
        Input {
            event: Event::Esc, ..
        } => cancel_search(ws),
        Input {
            event: Event::Enter,
            ..
        } => apply_search(ws),
        Input {
            event: Event::Char(ch),
            ..
        } => insert_pattern_char(ws, ch),
        Input {
            event: Event::Backspace,
            ..
        } => remove_pattern_char(ws),
        _ => outcome = EventOutcome::Ignore,
    };

    outcome
}

fn cancel_search(ws: &mut Workspace) {
    ws.search_registry_mut().cancel();
    super::visual_to_normal_impl(ws.cur_mut().buf_mut());
}

fn apply_search(ws: &mut Workspace) {
    ws.search_registry_mut().apply();
}

fn insert_pattern_char(ws: &mut Workspace, ch: char) {
    ws.search_registry_mut().insert_char(ch);
}

fn remove_pattern_char(ws: &mut Workspace) {
    ws.search_registry_mut().remove_char();
}

pub(super) fn search_next(ws: &mut Workspace) {
    todo!()
}

pub(super) fn search_prev(ws: &mut Workspace) {
    todo!()
}
