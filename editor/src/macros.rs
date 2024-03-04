#[macro_export]
macro_rules! current {
    ($workspace: expr) => {{
        $workspace.buffers.get(&$workspace.current)
    }};
    ($workspace: expr, logger) => {{
        $workspace.buffers.get(&$workspace.logger)
    }};
}

#[macro_export]
macro_rules! current_mut {
    ($workspace:expr) => {{
        $workspace.buffers.get_mut(&$workspace.current)
    }};
    ($workspace: expr, logger) => {{
        $workspace.buffers.get_mut(&$workspace.logger)
    }};
}

#[macro_export]
macro_rules! command {
    ($fun: ident) => {{
        let name = stringify!($fun);
        let command = Command::new(name.to_string(), |workspace: &mut Workspace| {
            match current_mut!(workspace) {
                Some(buffer) => $fun(buffer),
                None => log::warn!("buffer is None, skipping command execution."),
            }
        });
        command
    }};
    ($fun: ident, with_workspace) => {{
        let name = stringify!($fun);
        let command = Command::new(name.to_string(), $fun(buffer));
        command
    }};
}
