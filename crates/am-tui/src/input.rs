use crate::model::{Mode, TuiMessage};
use ratatui::crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

pub fn map_event(event: &Event, mode: &Mode) -> Option<TuiMessage> {
    match event {
        Event::Key(key) => map_key(key, mode),
        Event::Resize(w, h) => Some(TuiMessage::Resize(*w, *h)),
        _ => None,
    }
}

fn map_key(key: &KeyEvent, mode: &Mode) -> Option<TuiMessage> {
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return Some(TuiMessage::Quit);
    }
    match mode {
        Mode::Normal | Mode::Moving => map_normal_key(key, mode),
        Mode::TextInput(_) => map_text_input_key(key),
        Mode::Confirm(_) => map_confirm_key(key),
    }
}

fn map_normal_key(key: &KeyEvent, mode: &Mode) -> Option<TuiMessage> {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => Some(TuiMessage::CursorDown),
        KeyCode::Char('k') | KeyCode::Up => Some(TuiMessage::CursorUp),
        KeyCode::Char('g') | KeyCode::Home => Some(TuiMessage::JumpTop),
        KeyCode::Char('G') | KeyCode::End => Some(TuiMessage::JumpBottom),
        KeyCode::Char(' ') => Some(TuiMessage::ToggleSelect),
        KeyCode::Char('m') => Some(TuiMessage::EnterMoveMode),
        KeyCode::Enter => Some(TuiMessage::ExecuteMove),
        KeyCode::Esc => Some(TuiMessage::CancelMove),
        KeyCode::Tab => Some(TuiMessage::SwitchColumn),
        KeyCode::Char('a') if *mode == Mode::Normal => Some(TuiMessage::StartAddAlias),
        KeyCode::Char('n') if *mode == Mode::Normal => Some(TuiMessage::StartCreateProfile),
        KeyCode::Char('x') if *mode == Mode::Normal => Some(TuiMessage::DeleteItem),
        KeyCode::Char('s') if *mode == Mode::Normal => Some(TuiMessage::SetActive),
        KeyCode::Char('q') => Some(TuiMessage::Quit),
        _ => None,
    }
}

fn map_text_input_key(key: &KeyEvent) -> Option<TuiMessage> {
    match key.code {
        KeyCode::Enter => Some(TuiMessage::TextInputConfirm),
        KeyCode::Esc => Some(TuiMessage::TextInputCancel),
        KeyCode::Tab => Some(TuiMessage::TextInputSwitchField),
        KeyCode::Backspace => Some(TuiMessage::TextInputBackspace),
        KeyCode::Char(c) => Some(TuiMessage::TextInputChar(c)),
        _ => None,
    }
}

fn map_confirm_key(key: &KeyEvent) -> Option<TuiMessage> {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => Some(TuiMessage::ConfirmYes),
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => Some(TuiMessage::ConfirmNo),
        _ => None,
    }
}
