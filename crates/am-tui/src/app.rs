use ratatui::crossterm::event;
use ratatui::DefaultTerminal;
use crate::input::map_event;
use crate::model::{TuiModel, TuiMessage, MIN_WIDTH, MIN_HEIGHT};
use crate::update::update;
use crate::view::draw;

pub fn run() -> anyhow::Result<()> {
    let (w, h) = ratatui::crossterm::terminal::size()?;
    if w < MIN_WIDTH || h < MIN_HEIGHT {
        anyhow::bail!(
            "Terminal too small ({w}x{h}). Minimum size: {MIN_WIDTH}x{MIN_HEIGHT}. Please resize and try again."
        );
    }

    let mut model = TuiModel::new()?;
    let mut terminal = ratatui::init();
    let result = run_loop(&mut terminal, &mut model);
    ratatui::restore();
    result
}

fn run_loop(terminal: &mut DefaultTerminal, model: &mut TuiModel) -> anyhow::Result<()> {
    loop {
        terminal.draw(|frame| draw(frame, model))?;
        let event = event::read()?;
        if let Some(msg) = map_event(&event, &model.mode) {
            if msg == TuiMessage::Quit {
                break;
            }
            if let TuiMessage::Resize(w, h) = msg {
                if w < MIN_WIDTH || h < MIN_HEIGHT {
                    return Err(anyhow::anyhow!(
                        "Terminal resized too small ({w}x{h}). Minimum size: {MIN_WIDTH}x{MIN_HEIGHT}."
                    ));
                }
                continue;
            }
            update(model, msg);
            let area = terminal.size()?;
            let visible_height = area.height.saturating_sub(1) as usize; // minus help bar
            model.adjust_scroll(visible_height);
        }
    }
    Ok(())
}
