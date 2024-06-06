use crate::{buffer::Mode, editor::Workspace, event::Input, renderer::EventOutcome};

pub fn on_key(ws: &mut Workspace, input: Input) -> EventOutcome {
    todo!()
}

pub(super) fn search_mode(ws: &mut Workspace) {
    ws.cur_mut().buf_mut().set_mode(Mode::Search)
}

pub(super) fn search_next(ws: &mut Workspace) {
    todo!()
}

pub(super) fn search_prev(ws: &mut Workspace) {
    todo!()
}
