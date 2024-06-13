use crate::{
    editor::Workspace,
    event::{Event, Input},
    renderer::EventOutcome,
    search::SearchOrder,
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
    super::normal_mode(ws);
    search_next(ws);
}

fn insert_pattern_char(ws: &mut Workspace, ch: char) {
    ws.search_registry_mut().insert_char(ch);
}

fn remove_pattern_char(ws: &mut Workspace) {
    ws.search_registry_mut().remove_char();
}

pub(super) fn search_next(ws: &mut Workspace) {
    search_impl(ws, SearchOrder::Next);
}

pub(super) fn search_prev(ws: &mut Workspace) {
    search_impl(ws, SearchOrder::Prev);
}

fn search_impl(ws: &mut Workspace, order: SearchOrder) {
    let buf = ws.cur().buf();
    let range = ws
        .search_registry()
        .search(buf.text(), buf.byte_pos(), order);

    if let Some((start, end)) = range {
        let buf = ws.cur_mut().buf_mut();
        buf.set_pos(buf.curs_pos(start));
        buf.new_selection(start);
        buf.update_selection(end);
    }
}
