#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventOutcome {
    Render,
    Ignore,
    Exit,
}

pub trait OnEvent {
    fn on_event(&mut self, event: crossterm::event::Event) -> EventOutcome;
}
