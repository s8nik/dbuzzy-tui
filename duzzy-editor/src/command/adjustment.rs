use crate::{
    editor::Workspace,
    set_cursor,
    transaction::{Transaction, TransactionResult},
};

pub(super) fn insert_char(ws: &mut Workspace, ch: char) {
    let doc = ws.curr_mut();

    doc.with_transaction(|insert_tx, buf| {
        let pos = buf.byte_pos();
        let mut tx = Transaction::new();

        tx.insert_char(pos, ch);
        tx.apply(&mut buf.text);

        insert_tx.merge(tx);
        set_cursor!(buf, offset += 1);
        TransactionResult::Keep
    });
}

pub(super) fn new_line(ws: &mut Workspace) {
    let doc = ws.curr_mut();

    doc.with_transaction(|insert_tx, buf| {
        let pos = buf.byte_pos();
        let mut tx = Transaction::new();

        tx.insert_char(pos, '\n');
        tx.apply(&mut buf.text);

        insert_tx.merge(tx);
        set_cursor!(buf, super::shift_down(1, buf));
        set_cursor!(buf, offset = 0);

        TransactionResult::Keep
    });
}

pub(super) fn delete_char_inplace(ws: &mut Workspace) {
    let doc = ws.curr_mut();

    doc.with_transaction(|tx, buf| {
        let pos = buf.byte_pos();

        if pos < buf.text.len_chars() {
            let ch = buf.text.char(pos);

            tx.delete_char(pos, ch);
            tx.apply(&mut buf.text);

            return TransactionResult::Commit;
        }

        TransactionResult::Abort
    });
}

pub(super) fn delete_char(ws: &mut Workspace) {
    let doc = ws.curr_mut();

    doc.with_transaction(|delete_tx, buf| {
        let pos = buf.byte_pos();

        if pos > 0 {
            let char_pos = pos - 1;

            let mut tx = Transaction::new();
            let ch = buf.text.char(char_pos);

            tx.delete_char(char_pos, ch);
            tx.apply(&mut buf.text);

            delete_tx.merge(tx);
            set_cursor!(buf, super::shift_left(buf));
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
        assert_eq!((0, 4), Into::into(&buf.pos));
        assert_eq!(&buf.text.to_string(), "test");

        delete_char(&mut ws);
        delete_char(&mut ws);

        let buf = ws.curr().buf();
        assert_eq!((0, 2), Into::into(&buf.pos));
        assert_eq!(buf.text.to_string().as_str(), "te");

        ws.curr_mut().buf_mut().pos = (0, 0).into();
        new_line(&mut ws);
        new_line(&mut ws);
        new_line(&mut ws);
        new_line(&mut ws);

        let buf = ws.curr().buf();
        assert_eq!((4, 0), Into::into(&buf.pos));
        assert_eq!(buf.text.to_string().as_str(), "\n\n\n\nte");

        ws.curr_mut().commit();

        delete_char_inplace(&mut ws);
        delete_char_inplace(&mut ws);
        delete_char_inplace(&mut ws);

        let buf = ws.curr().buf();
        assert_eq!((4, 0), Into::into(&buf.pos));
        assert_eq!(buf.text.to_string().as_str(), "\n\n\n\n");
    }
}
