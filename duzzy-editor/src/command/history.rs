use crate::{
    buffer::Position,
    doc_mut,
    editor::Workspace,
    history::{Change, Commit, History},
    set_cursor, SmartString,
};

pub(super) fn undo(ws: &mut Workspace) {
    let (buf, history) = doc_mut!(ws);

    if let Some(pos) = history.undo(&mut buf.text) {
        set_cursor!(buf, pos);
    }
}

pub(super) fn redo(ws: &mut Workspace) {
    let (buf, history) = doc_mut!(ws);

    if let Some(pos) = history.redo(&mut buf.text) {
        set_cursor!(buf, pos);
    }
}

pub(super) fn insert_char(ch: char, history: &mut History, before: Position, after: Position) {
    let commit = match history.tx() {
        Some(commit) => commit,
        None => {
            let mut content = SmartString::new_const();
            content.push(ch);
            let commit = Commit::new(Change::Insert(content), before, after);
            return history.set_tx(commit);
        }
    };

    if let Change::Insert(content) = &mut commit.change {
        content.push(ch);
        commit.after = after;
    }
}

pub(super) fn delete_slice(
    s: SmartString,
    history: &mut History,
    before: Position,
    after: Position,
) {
    let commit = match history.tx() {
        Some(commit) => commit,
        None => {
            let mut content = SmartString::new_const();
            content.push_str(s.as_str());
            let commit = Commit::new(Change::Delete(content), before, after);
            return history.set_tx(commit);
        }
    };

    if let Change::Delete(content) = &mut commit.change {
        content.push_str(s.as_str());
        commit.after = after;
    }
}
