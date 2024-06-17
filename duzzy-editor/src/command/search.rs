use duzzy_lib::EventOutcome;

use crate::{
    editor::Workspace,
    event::{Event, Input},
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
    ws.search_buffer.clear();
    super::visual_to_normal_impl(ws.cur_mut().buf_mut());
}

fn apply_search(ws: &mut Workspace) {
    ws.apply_search();
    super::normal_mode(ws);
    search_next(ws);
}

fn insert_pattern_char(ws: &mut Workspace, ch: char) {
    ws.search_buffer.push(ch);
}

fn remove_pattern_char(ws: &mut Workspace) {
    ws.search_buffer.pop();
}

pub(super) fn search_next(ws: &mut Workspace) {
    search_impl(ws, SearchOrder::Next);
}

pub(super) fn search_prev(ws: &mut Workspace) {
    search_impl(ws, SearchOrder::Prev);
}

fn search_impl(ws: &mut Workspace, order: SearchOrder) {
    let buf = ws.cur().buf();

    let text = buf.text();
    let pos = buf.byte_pos();

    let start_pos = match order {
        SearchOrder::Next => (pos + 1).min(text.len_chars()),
        SearchOrder::Prev => pos.saturating_sub(1),
    };

    let range = ws.search_registry().search(buf.text(), start_pos, order);

    if let Some((start, end)) = range {
        let buf = ws.cur_mut().buf_mut();
        buf.set_pos(buf.curs_pos(end));
        buf.new_selection(start);
        buf.update_selection(end);
    }
}
