use crossterm::event::{self, Event, KeyEvent};
use std::time::Duration;

pub enum AppEvent {
    Key(KeyEvent),
    Tick,
}

pub fn poll() -> std::io::Result<Option<AppEvent>> {
    if event::poll(Duration::from_millis(200))? {
        if let Event::Key(key) = event::read()? {
            return Ok(Some(AppEvent::Key(key)));
        }
    }
    Ok(Some(AppEvent::Tick))
}
