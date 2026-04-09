//! Event handling for TUI

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

/// Application events
pub enum AppEvent {
    /// Key press
    Key(KeyEvent),
    /// Tick (for animations)
    Tick,
    /// Quit
    Quit,
}

/// Handle a key event and return an action
pub fn handle_key(key: KeyEvent) -> Option<Action> {
    match (key.modifiers, key.code) {
        // Quit
        (KeyModifiers::CONTROL, KeyCode::Char('c')) => Some(Action::Quit),
        (KeyModifiers::NONE, KeyCode::Esc) => Some(Action::Quit),

        // Submit input
        (KeyModifiers::NONE, KeyCode::Enter) => Some(Action::Submit),
        (KeyModifiers::CONTROL, KeyCode::Enter) => Some(Action::SubmitMultiline),

        // Navigation
        (KeyModifiers::NONE, KeyCode::Up) => Some(Action::HistoryPrev),
        (KeyModifiers::NONE, KeyCode::Down) => Some(Action::HistoryNext),
        (KeyModifiers::NONE, KeyCode::PageUp) => Some(Action::ScrollUp),
        (KeyModifiers::NONE, KeyCode::PageDown) => Some(Action::ScrollDown),

        // Function keys
        (KeyModifiers::NONE, KeyCode::F(1)) => Some(Action::ShowHelp),
        (KeyModifiers::NONE, KeyCode::F(2)) => Some(Action::ChangeProvider),
        (KeyModifiers::NONE, KeyCode::F(3)) => Some(Action::ChangeModel),
        (KeyModifiers::NONE, KeyCode::F(4)) => Some(Action::ChangeProject),

        // Clear
        (KeyModifiers::CONTROL, KeyCode::Char('l')) => Some(Action::Clear),

        // Attach file
        (KeyModifiers::CONTROL, KeyCode::Char('o')) => Some(Action::AttachFile),

        // Text input
        (KeyModifiers::NONE, KeyCode::Char(c)) => Some(Action::Input(c)),
        (KeyModifiers::SHIFT, KeyCode::Char(c)) => Some(Action::Input(c.to_ascii_uppercase())),
        (KeyModifiers::NONE, KeyCode::Backspace) => Some(Action::Backspace),
        (KeyModifiers::CONTROL, KeyCode::Backspace) => Some(Action::DeleteWord),

        _ => None,
    }
}

/// Actions that can be performed
#[derive(Debug, Clone)]
pub enum Action {
    Quit,
    Submit,
    SubmitMultiline,
    HistoryPrev,
    HistoryNext,
    ScrollUp,
    ScrollDown,
    ShowHelp,
    ChangeProvider,
    ChangeModel,
    ChangeProject,
    Clear,
    AttachFile,
    Input(char),
    Backspace,
    DeleteWord,
}
