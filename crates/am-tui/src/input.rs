use crate::model::{Mode, TuiMessage};
use ratatui::crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::time::{Duration, Instant};

pub fn map_event(event: &Event, mode: &Mode) -> Option<TuiMessage> {
    match event {
        Event::Key(key) if key.kind == KeyEventKind::Press => map_key(key, mode),
        Event::Resize(w, h) => Some(TuiMessage::Resize(*w, *h)),
        _ => None,
    }
}

fn map_key(key: &KeyEvent, mode: &Mode) -> Option<TuiMessage> {
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return Some(TuiMessage::Quit);
    }
    match mode {
        Mode::Normal | Mode::Transfer(_) => map_normal_key(key, mode),
        Mode::TextInput(_) => map_text_input_key(key),
        Mode::Confirm(_) => map_confirm_key(key),
    }
}

fn map_normal_key(key: &KeyEvent, _mode: &Mode) -> Option<TuiMessage> {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => Some(TuiMessage::CursorDown),
        KeyCode::Char('k') | KeyCode::Up => Some(TuiMessage::CursorUp),
        KeyCode::Char('g') | KeyCode::Home => Some(TuiMessage::JumpTop),
        KeyCode::Char('G') | KeyCode::End => Some(TuiMessage::JumpBottom),
        KeyCode::Char(' ') => Some(TuiMessage::ToggleSelect),
        KeyCode::Char('m') => Some(TuiMessage::EnterMoveMode),
        KeyCode::Char('c') => Some(TuiMessage::EnterCopyMode),
        KeyCode::Enter => Some(TuiMessage::ExecuteTransfer),
        KeyCode::Esc => Some(TuiMessage::CancelTransfer),
        KeyCode::Tab => Some(TuiMessage::SwitchColumn),
        KeyCode::Char('u') => Some(TuiMessage::UseProfile),
        KeyCode::Char('a') => Some(TuiMessage::StartAddAlias),
        KeyCode::Char('n') => Some(TuiMessage::StartCreateProfile),
        KeyCode::Char('e') => Some(TuiMessage::EditItem),
        KeyCode::Char('x') => Some(TuiMessage::DeleteItem),
        KeyCode::Char(c @ '1'..='9') => Some(TuiMessage::UseProfileWithPriority(
            c as usize - '0' as usize,
        )),
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

const SEQUENCE_DEADLINE: Duration = Duration::from_millis(300);
const IDLE_TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Default)]
pub struct InputResolver {
    pending: Option<(TuiMessage, Instant)>,
}

impl InputResolver {
    pub fn poll_timeout(&self) -> Duration {
        match &self.pending {
            Some((_, t)) => SEQUENCE_DEADLINE.saturating_sub(t.elapsed()),
            None => IDLE_TIMEOUT,
        }
    }

    /// Feed a raw terminal event. Returns 0–2 resolved messages.
    pub fn feed(&mut self, event: &Event, mode: &Mode) -> Vec<TuiMessage> {
        let Some(msg) = map_event(event, mode) else {
            return vec![];
        };

        if let Some((pending, _)) = self.pending.take() {
            match (&pending, &msg) {
                // u + digit → resolve as priority activation
                (TuiMessage::UseProfile, TuiMessage::UseProfileWithPriority(_)) => vec![msg],
                // u + u → flush first pending, start new sequence
                (TuiMessage::UseProfile, TuiMessage::UseProfile) => {
                    self.pending = Some((msg, Instant::now()));
                    vec![pending]
                }
                // u + anything else → flush pending, then the new key
                _ => vec![pending, msg],
            }
        } else if matches!(msg, TuiMessage::UseProfile) {
            self.pending = Some((msg, Instant::now()));
            vec![]
        } else {
            vec![msg]
        }
    }

    /// Call when `event::poll` times out. Flushes any expired pending message.
    pub fn flush(&mut self) -> Vec<TuiMessage> {
        match self.pending.take() {
            Some((msg, _)) => vec![msg],
            None => vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn press(c: char) -> Event {
        Event::Key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE))
    }

    fn normal() -> Mode {
        Mode::Normal
    }

    // ── feed: passthrough ──────────────────────────────────────────

    #[test]
    fn feed_normal_key_passes_through() {
        let mut r = InputResolver::default();
        let msgs = r.feed(&press('j'), &normal());
        assert_eq!(msgs, vec![TuiMessage::CursorDown]);
    }

    #[test]
    fn feed_unrecognised_key_returns_empty() {
        let mut r = InputResolver::default();
        let msgs = r.feed(&press('z'), &normal());
        assert!(msgs.is_empty());
    }

    // ── feed: sequence start ───────────────────────────────────────

    #[test]
    fn feed_use_profile_starts_sequence_returns_empty() {
        let mut r = InputResolver::default();
        let msgs = r.feed(&press('u'), &normal());
        assert!(msgs.is_empty(), "u should be held pending");
    }

    // ── feed: sequence resolution ──────────────────────────────────

    #[test]
    fn feed_u_then_digit_resolves_to_priority() {
        let mut r = InputResolver::default();
        r.feed(&press('u'), &normal());
        let msgs = r.feed(&press('3'), &normal());
        assert_eq!(msgs, vec![TuiMessage::UseProfileWithPriority(3)]);
    }

    #[test]
    fn feed_u_then_non_digit_flushes_pending_and_new() {
        let mut r = InputResolver::default();
        r.feed(&press('u'), &normal());
        let msgs = r.feed(&press('j'), &normal());
        assert_eq!(msgs, vec![TuiMessage::UseProfile, TuiMessage::CursorDown]);
    }

    #[test]
    fn feed_u_then_u_flushes_first_starts_new_sequence() {
        let mut r = InputResolver::default();
        r.feed(&press('u'), &normal());
        let msgs = r.feed(&press('u'), &normal());
        assert_eq!(msgs, vec![TuiMessage::UseProfile], "first u flushed");

        // second u is still pending — flush it
        let msgs = r.flush();
        assert_eq!(
            msgs,
            vec![TuiMessage::UseProfile],
            "second u flushed on timeout"
        );
    }

    #[test]
    fn feed_u_then_digit_clears_pending() {
        let mut r = InputResolver::default();
        r.feed(&press('u'), &normal());
        r.feed(&press('3'), &normal());

        // nothing pending any more
        let msgs = r.flush();
        assert!(msgs.is_empty());
    }

    // ── flush ──────────────────────────────────────────────────────

    #[test]
    fn flush_with_pending_returns_message() {
        let mut r = InputResolver::default();
        r.feed(&press('u'), &normal());
        let msgs = r.flush();
        assert_eq!(msgs, vec![TuiMessage::UseProfile]);
    }

    #[test]
    fn flush_without_pending_returns_empty() {
        let mut r = InputResolver::default();
        let msgs = r.flush();
        assert!(msgs.is_empty());
    }

    #[test]
    fn flush_drains_pending() {
        let mut r = InputResolver::default();
        r.feed(&press('u'), &normal());
        r.flush();
        let msgs = r.flush();
        assert!(msgs.is_empty(), "second flush should be empty");
    }

    // ── poll_timeout ───────────────────────────────────────────────

    #[test]
    fn poll_timeout_idle_returns_long_duration() {
        let r = InputResolver::default();
        assert_eq!(r.poll_timeout(), IDLE_TIMEOUT);
    }

    #[test]
    fn poll_timeout_with_pending_returns_short_duration() {
        let mut r = InputResolver::default();
        r.feed(&press('u'), &normal());
        let t = r.poll_timeout();
        assert!(
            t <= SEQUENCE_DEADLINE,
            "should be at most {SEQUENCE_DEADLINE:?}, got {t:?}"
        );
        assert!(
            t > Duration::ZERO,
            "should be non-zero immediately after feed"
        );
    }
}
