mod connections;

pub use connections::Connections;
use duzzy_lib::EventOutcome;

// @todo:
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppEventOutcome {
    Outcome(EventOutcome),
    Focus(AppWidget),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppWidget {
    Connections,
    Editor,
}

impl From<EventOutcome> for AppEventOutcome {
    fn from(value: EventOutcome) -> Self {
        Self::Outcome(value)
    }
}
