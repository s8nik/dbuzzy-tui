use crate::workspace::Workspace;

// @note: for development purposes. will be improved later
pub(super) fn open_close_logger(workspace: &mut Workspace) {
    if workspace.logger_active() {
        // at this point we do not care about the last active buffer
        let id = workspace.buffers.keys().last().unwrap();
        workspace.set_current_id(*id);

        return;
    }

    let id = workspace.logger_id();
    workspace.set_current_id(id);
}
