use crate::{editor::Workspace, transaction::TransactionResult};

enum ClipboardType {
    Local,
    Global,
}

pub(super) fn copy_local(ws: &mut Workspace) {
    copy_clipboard_impl(ws, ClipboardType::Local);
}

pub(super) fn copy_global(ws: &mut Workspace) {
    copy_clipboard_impl(ws, ClipboardType::Global);
}

pub(super) fn paste_local(ws: &mut Workspace) {
    paste_clipboard_impl(ws, ClipboardType::Local);
}

pub(super) fn paste_global(ws: &mut Workspace) {
    paste_clipboard_impl(ws, ClipboardType::Global);
}

fn copy_clipboard_impl(ws: &mut Workspace, clipboard_type: ClipboardType) {
    let buf = ws.cur().buf();
    let selected_text = super::selected_text(buf).map(|x| x.to_string());

    if let Some(text) = selected_text {
        let clipboard = ws.clipboard();

        match clipboard_type {
            ClipboardType::Local => clipboard.set_local(text),
            ClipboardType::Global => clipboard.set_global(text),
        }
    }
}

fn paste_clipboard_impl(ws: &mut Workspace, clipboard_type: ClipboardType) {
    let clipboard = ws.clipboard();

    let text = match clipboard_type {
        ClipboardType::Local => clipboard.get_local(),
        ClipboardType::Global => clipboard.get_global(),
    };

    if text.is_empty() {
        return;
    }

    let doc = ws.cur_mut();
    doc.with_transaction(|tx, buf| {
        let pos = buf.byte_pos();

        if buf.is_visual() && super::delete_selection(buf, tx) {
            tx.insert_str(pos, &text);
        } else {
            let shift = (pos + 1).min(buf.len_chars());
            tx.shift(pos);
            tx.insert_str(shift, &text);
        };

        tx.apply(buf.text_mut());
        TransactionResult::Commit
    });
}

#[cfg(test)]
mod tests {
    use crate::{buffer::Mode, document::Document, editor::Workspace};

    #[test]
    fn test_past_in_visual() {
        let mut ws = Workspace::default();
        ws.add_doc(Document::default());
        ws.clipboard().set_local("hello".to_owned());

        let text = ropey::Rope::from_str("test test\ntest");
        let buf = ws.cur_mut().buf_mut();

        buf.set_mode(Mode::Visual);
        buf.set_text(text);
        buf.new_selection(buf.len_chars() - 1);
        buf.update_selection(5);
        buf.set_pos((0, 5));

        super::paste_local(&mut ws);

        assert_eq!(&ws.cur().buf().text().to_string(), "test hello");
    }
}
