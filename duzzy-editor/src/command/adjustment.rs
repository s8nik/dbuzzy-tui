use std::borrow::Cow;

use crate::{
    editor::Workspace,
    transaction::{Transaction, TransactionResult},
};

pub(super) fn insert_char(ws: &mut Workspace, ch: char) {
    let doc = ws.curr_mut();

    doc.with_transaction(|insert_tx, buf| {
        let pos = buf.byte_pos();
        let mut tx = Transaction::new();

        tx.insert_char(pos, ch);
        tx.apply(buf.text_mut());

        insert_tx.merge(tx);
        let ofs = buf.offset() + 1;
        buf.set_offset(ofs);

        TransactionResult::Keep
    });
}

pub(super) fn new_line(ws: &mut Workspace) {
    let doc = ws.curr_mut();

    doc.with_transaction(|insert_tx, buf| {
        let pos = buf.byte_pos();
        let mut tx = Transaction::new();

        tx.insert_char(pos, '\n');
        tx.apply(buf.text_mut());

        insert_tx.merge(tx);

        let new_pos = super::shift_down(1, buf);

        buf.set_pos(new_pos);
        buf.set_offset(0);

        TransactionResult::Keep
    });
}

pub(super) fn delete(ws: &mut Workspace) {
    let doc = ws.curr_mut();

    doc.with_transaction(|tx, buf| {
        let pos = buf.byte_pos();

        if let Some(selection) = buf.selection() {
            let (start, end) = selection.range();
            let len_chars = buf.len_chars();

            let slice = buf.text().slice(start..(end + 1).min(len_chars));
            let slice = match slice.as_str() {
                Some(s) => Cow::from(s),
                None => Cow::from(slice.to_string()),
            };

            tx.delete_str(start, &slice);
            if let Some(pos) = tx.apply(buf.text_mut()) {
                buf.set_pos(buf.curs_pos(pos));
            }

            super::switch_mode::visual_to_normal_impl(buf);
            return TransactionResult::Commit;
        }

        if pos < buf.len_chars() {
            let ch = buf.char(pos);

            tx.delete_char(pos, ch);
            tx.apply(buf.text_mut());

            return TransactionResult::Commit;
        }

        TransactionResult::Abort
    });
}

pub(super) fn delete_backspace(ws: &mut Workspace) {
    let doc = ws.curr_mut();

    doc.with_transaction(|delete_tx, buf| {
        let pos = buf.byte_pos();

        if pos > 0 {
            let char_pos = pos - 1;

            let mut tx = Transaction::new();
            let ch = buf.char(char_pos);

            tx.delete_char(char_pos, ch);
            tx.apply(buf.text_mut());

            delete_tx.merge(tx);

            let new_pos = super::shift_left(buf);
            buf.set_pos(new_pos);
        }

        TransactionResult::Keep
    });
}

#[cfg(test)]
mod tests {
    use crate::document::Document;

    use super::*;

    #[test]
    fn test_adjustment() {
        let mut ws = Workspace::default();
        ws.add_doc(Document::default());

        insert_char(&mut ws, 't');
        insert_char(&mut ws, 'e');
        insert_char(&mut ws, 's');
        insert_char(&mut ws, 't');

        let buf = ws.curr().buf();
        assert_eq!((0, 4), buf.pos());
        assert_eq!(&buf.text().to_string(), "test");

        delete_backspace(&mut ws);
        delete_backspace(&mut ws);

        let buf = ws.curr().buf();
        assert_eq!((0, 2), buf.pos());
        assert_eq!(&buf.text().to_string(), "te");

        ws.curr_mut().buf_mut().set_pos((0, 0));
        new_line(&mut ws);
        new_line(&mut ws);
        new_line(&mut ws);
        new_line(&mut ws);

        let buf = ws.curr().buf();
        assert_eq!((4, 0), buf.pos());
        assert_eq!(&buf.text().to_string(), "\n\n\n\nte");

        ws.curr_mut().commit();

        delete(&mut ws);
        delete(&mut ws);
        delete(&mut ws);

        let buf = ws.curr().buf();
        assert_eq!((4, 0), buf.pos());
        assert_eq!(&buf.text().to_string(), "\n\n\n\n");
    }

    #[test]
    fn test_delete_selection() {
        let mut ws = Workspace::default();
        ws.add_doc(Document::default());

        let text = ropey::Rope::from_str("test \ntest");
        let doc = ws.curr_mut();
        let buf = doc.buf_mut();
        buf.set_text(text);
        buf.new_selection(buf.len_chars());
        buf.update_selection(4);

        delete(&mut ws);

        assert_eq!(&ws.curr().buf().text().to_string(), "test");
    }
}
