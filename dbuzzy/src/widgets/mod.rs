mod conn_list;
mod db_tree;

pub use conn_list::ConnListWidget;
pub use db_tree::DbTreeWidget;

use duzzy_lib::EventOutcome;

use crate::db::PgPool;

// @todo:
#[allow(dead_code)]
pub enum AppEventOutcome {
    Outcome(EventOutcome),
    Focus(AppWidgetName),
    Apply(AppWidgetData),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppWidgetName {
    ConnectionList,
    DatabaseTree,
    Editor,
}

pub enum AppWidgetData {
    Connection(PgPool),
}

impl From<EventOutcome> for AppEventOutcome {
    fn from(value: EventOutcome) -> Self {
        Self::Outcome(value)
    }
}
