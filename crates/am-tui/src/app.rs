use crate::input::map_event;
use crate::model::{TuiMessage, TuiModel, MIN_HEIGHT, MIN_WIDTH};
use crate::update::update;
use crate::view::draw;
use ratatui::crossterm::event;
use ratatui::DefaultTerminal;

pub fn run() -> anyhow::Result<()> {
    check_terminal_size()?;

    let mut model = TuiModel::new()?;
    let mut terminal = ratatui::init();
    let result = run_loop(&mut terminal, &mut model);
    ratatui::restore();
    result
}

fn check_terminal_size() -> anyhow::Result<()> {
    let (w, h) = ratatui::crossterm::terminal::size()?;
    if w < MIN_WIDTH || h < MIN_HEIGHT {
        anyhow::bail!(
            "Terminal too small ({w}x{h}). Minimum size: {MIN_WIDTH}x{MIN_HEIGHT}. Please resize and try again."
        );
    }
    Ok(())
}

fn run_loop(terminal: &mut DefaultTerminal, model: &mut TuiModel) -> anyhow::Result<()> {
    let mut pending_use: Option<std::time::Instant> = None;

    loop {
        terminal.draw(|frame| draw(frame, model))?;

        let timeout = match pending_use {
            Some(instant) => {
                let elapsed = instant.elapsed();
                let deadline = std::time::Duration::from_millis(300);
                if elapsed >= deadline {
                    // Timeout expired — send plain toggle
                    update(model, TuiMessage::UseProfile);
                    pending_use = None;
                    let area = terminal.size()?;
                    model.adjust_scroll(area.height.saturating_sub(1) as usize);
                    continue;
                }
                deadline - elapsed
            }
            None => std::time::Duration::from_secs(60),
        };

        if event::poll(timeout)? {
            let event = event::read()?;
            if let Some(msg) = map_event(&event, &model.mode) {
                if msg == TuiMessage::Quit {
                    break;
                }
                if let TuiMessage::Resize(..) = msg {
                    check_terminal_size()?;
                    continue;
                }

                if pending_use.is_some() {
                    if let TuiMessage::UseProfileWithPriority(n) = msg {
                        // Digit key while pending — activate at priority
                        update(model, TuiMessage::UseProfileWithPriority(n));
                        pending_use = None;
                    } else {
                        // Non-digit key while pending — execute plain toggle first
                        update(model, TuiMessage::UseProfile);
                        pending_use = None;
                        // Then handle the actual key (unless it was another u press)
                        if msg == TuiMessage::UseProfile {
                            pending_use = Some(std::time::Instant::now());
                        } else {
                            update(model, msg);
                        }
                    }
                } else if msg == TuiMessage::UseProfile {
                    // Start pending — wait for digit
                    pending_use = Some(std::time::Instant::now());
                } else {
                    update(model, msg);
                }

                let area = terminal.size()?;
                let visible_height = area.height.saturating_sub(1) as usize;
                model.adjust_scroll(visible_height);
            }
        }
    }
    Ok(())
}
