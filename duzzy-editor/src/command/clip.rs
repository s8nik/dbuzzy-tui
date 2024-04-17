use crate::editor::Workspace;

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
    todo!()
}
