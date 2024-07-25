mod connections;
mod databases;

pub use connections::ConnectionsWidget;
pub use databases::DatabaseTreeWidget;

use duzzy_lib::EventOutcome;

use crate::db::connection::PgPool;

// @todo:
#[allow(dead_code)]
pub enum AppEventOutcome {
    Outcome(EventOutcome),
    Focus(AppWidgetName),
    Apply(AppWidgetData),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppWidgetName {
    Connections,
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
