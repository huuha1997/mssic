use crossterm::event::KeyCode;

use crate::types::{AppAction, InputMode};

pub fn map_key(mode: &InputMode, code: KeyCode, eq_panel_open: bool) -> Option<AppAction> {
    // EQ panel has its own key bindings
    if eq_panel_open {
        return match code {
            KeyCode::Char('e') | KeyCode::Esc => Some(AppAction::ToggleEQ),
            KeyCode::Left | KeyCode::Char('h') => Some(AppAction::EqPrevBand),
            KeyCode::Right | KeyCode::Char('l') => Some(AppAction::EqNextBand),
            KeyCode::Up | KeyCode::Char('k') => Some(AppAction::EqBandUp),
            KeyCode::Down | KeyCode::Char('j') => Some(AppAction::EqBandDown),
            KeyCode::Char('p') => Some(AppAction::EqNextPreset),
            KeyCode::Char('P') => Some(AppAction::EqPrevPreset),
            KeyCode::Char('t') => Some(AppAction::EqToggleEnabled),
            KeyCode::Char('q') => Some(AppAction::Quit),
            KeyCode::Char(' ') => Some(AppAction::TogglePause),
            _ => None,
        };
    }

    match mode {
        InputMode::Normal => match code {
            KeyCode::Char('q') => Some(AppAction::Quit),
            KeyCode::Char('i') => Some(AppAction::EnterEdit),
            KeyCode::Tab => Some(AppAction::NextTab),
            KeyCode::Enter => Some(AppAction::PlaySelected),
            KeyCode::Char(' ') => Some(AppAction::TogglePause),
            KeyCode::Char('f') => Some(AppAction::ToggleFavorite),
            KeyCode::Char('a') => Some(AppAction::AddToQueue),
            KeyCode::Char('+') | KeyCode::Char('=') => Some(AppAction::VolumeUp),
            KeyCode::Char('-') => Some(AppAction::VolumeDown),
            KeyCode::Char('e') => Some(AppAction::ToggleEQ),
            KeyCode::Char('?') => Some(AppAction::ShowHelp),
            KeyCode::Char('s') => Some(AppAction::Stop),
            KeyCode::Left | KeyCode::Char('h') => Some(AppAction::SeekBackward),
            KeyCode::Right | KeyCode::Char('l') => Some(AppAction::SeekForward),
            KeyCode::Down | KeyCode::Char('j') => Some(AppAction::NextItem),
            KeyCode::Up | KeyCode::Char('k') => Some(AppAction::PrevItem),
            _ => None,
        },
        InputMode::Editing => match code {
            KeyCode::Enter => Some(AppAction::Search),
            KeyCode::Esc => Some(AppAction::ExitEdit),
            KeyCode::Char(c) => Some(AppAction::TypeChar(c)),
            KeyCode::Backspace => Some(AppAction::Backspace),
            _ => None,
        },
    }
}
