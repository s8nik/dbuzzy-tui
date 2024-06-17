use ratatui::widgets::ListState;

use crate::db::connection::ConnCfg;

pub struct ConnWidget {
    items: Vec<ConnCfg>,
    state: ListState,
}

impl ConnWidget {
    pub fn new(conns: Vec<ConnCfg>) -> Self {
        Self {
            items: conns,
            state: ListState::default(),
        }
    }

    pub fn next_conn(&mut self) {
        let i = self
            .state
            .selected()
            .map(|i| if i > self.items.len() - 1 { 0 } else { i + 1 });

        self.state.select(i);
    }

    pub fn prev_conn(&mut self) {
        let i = self
            .state
            .selected()
            .map(|i| if i == 0 { self.items.len() - 1 } else { i - 1 });

        self.state.select(i);
    }

    pub fn selected_conn(&self) -> Option<&ConnCfg> {
        self.state.selected().and_then(|i| self.items.get(i))
    }
}
