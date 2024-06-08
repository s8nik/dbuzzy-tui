use crate::{
    editor::Workspace,
    event::{Event, Input},
    renderer::EventOutcome,
};

pub fn on_key(ws: &mut Workspace, input: Input) -> EventOutcome {
    match input {
        Input {
            event: Event::Esc, ..
        } => super::normal_mode(ws),
        Input {
            event: Event::Enter,
            ..
        } => apply_search(ws),
        Input {
            event: Event::Char(ch),
            ..
        } => todo!(),
        _ => todo!(),
    };

    todo!()
}

fn apply_search(ws: &mut Workspace) {
    todo!()
}

pub(super) fn search_next(ws: &mut Workspace) {
    todo!()
}

pub(super) fn search_prev(ws: &mut Workspace) {
    todo!()
}
