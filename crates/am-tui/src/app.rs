use crate::input::InputResolver;
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
    let mut input = InputResolver::default();

    loop {
        terminal.draw(|frame| draw(frame, model))?;

        let messages = if event::poll(input.poll_timeout())? {
            input.feed(&event::read()?, &model.mode)
        } else {
            input.flush()
        };

        for msg in messages {
            match msg {
                TuiMessage::Quit => return Ok(()),
                TuiMessage::Resize(..) => {
                    check_terminal_size()?;
                    continue;
                }
                msg => update(model, msg),
            }
            model.adjust_scroll(terminal.size()?.height.saturating_sub(1) as usize);
        }
    }
}
