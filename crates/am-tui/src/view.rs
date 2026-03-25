use ratatui::prelude::*;
use ratatui::widgets::Paragraph;
use ratatui::Frame;
use crate::model::*;

// Noctavox-inspired warm color palette
const TEXT_PRIMARY: Color = Color::Rgb(210, 210, 213);   // #d2d2d5
const TEXT_MUTED: Color = Color::Rgb(100, 100, 103);     // #646467
const GOLD: Color = Color::Rgb(220, 220, 100);           // #dcdc64
const GOLD_FADED: Color = Color::Rgb(130, 130, 60);      // #82823c
const TREE_CONNECTOR: Color = Color::Rgb(70, 70, 73);    // dim connector lines

pub fn draw(frame: &mut Frame, model: &TuiModel) {
    let area = frame.area();

    let help_text = help_bar_text(&model.mode);
    let help = Paragraph::new(help_text).style(Style::default().fg(TEXT_MUTED));

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(area);

    frame.render_widget(help, chunks[0]);

    let content_area = chunks[1];

    match &model.mode {
        Mode::Moving => {
            let columns = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
                .split(content_area);

            render_left_column(frame, model, columns[0]);
            render_right_column(frame, model, columns[1]);
        }
        Mode::TextInput(text) => {
            render_left_column(frame, model, content_area);
            render_text_input(frame, text, content_area);
        }
        Mode::Confirm(action) => {
            render_left_column(frame, model, content_area);
            render_confirm(frame, action, content_area);
        }
        Mode::Normal => {
            render_left_column(frame, model, content_area);
        }
    }
}

fn render_left_column(frame: &mut Frame, model: &TuiModel, area: Rect) {
    let tree_lines = render_tree_lines(model);
    let visible_height = area.height as usize;
    let start = model.scroll_offset;
    let end = (start + visible_height).min(tree_lines.len());
    let visible: Vec<Line> = if start < tree_lines.len() {
        tree_lines[start..end].to_vec()
    } else {
        Vec::new()
    };

    let tree_widget = Paragraph::new(Text::from(visible));
    frame.render_widget(tree_widget, area);
}

fn render_right_column(frame: &mut Frame, model: &TuiModel, area: Rect) {
    let mut lines: Vec<Line<'static>> = Vec::new();
    lines.push(Line::from(Span::styled(
        "  ──────────────────►",
        Style::default().fg(TREE_CONNECTOR),
    )));
    lines.push(Line::from(""));

    for (i, node) in model.dest_tree.iter().enumerate() {
        let is_cursor = i == model.dest_cursor && model.active_column == Column::Right;
        let marker = if is_cursor { "▸ " } else { "  " };

        match &node.kind {
            NodeKind::GlobalHeader => {
                lines.push(Line::from(vec![
                    Span::styled(format!("{}{marker}", node.prefix), Style::default().fg(TREE_CONNECTOR)),
                    Span::raw("🌐 "),
                    Span::styled("global", Style::default().fg(GOLD).bold()),
                ]));
            }
            NodeKind::ProjectHeader => {
                lines.push(Line::from(vec![
                    Span::styled(format!("{}{marker}", node.prefix), Style::default().fg(TREE_CONNECTOR)),
                    Span::raw("📁 "),
                    Span::styled("project (.aliases)", Style::default().fg(GOLD).bold()),
                ]));
            }
            NodeKind::ProfileHeader => {
                let icon = if node.is_active { "●" } else { "○" };
                let active_tag = if node.is_active { " (active)" } else { "" };

                if !node.prefix.is_empty() {
                    let connector_line = if node.prefix.ends_with("├─") {
                        let parent_cp = &node.prefix[..node.prefix.len() - "├─".len()];
                        format!("{parent_cp}│")
                    } else if node.prefix.ends_with("╰─") {
                        let parent_cp = &node.prefix[..node.prefix.len() - "╰─".len()];
                        format!("{parent_cp}│")
                    } else {
                        node.content_prefix.clone()
                    };
                    lines.push(Line::from(Span::styled(connector_line, Style::default().fg(TREE_CONNECTOR))));
                }

                lines.push(Line::from(vec![
                    Span::styled(format!("{}{marker}", node.prefix), Style::default().fg(TREE_CONNECTOR)),
                    Span::styled(
                        format!("{icon} {}{active_tag}", node.label),
                        Style::default().fg(GOLD).bold(),
                    ),
                ]));
            }
            NodeKind::AliasItem => {}
        }
    }

    frame.render_widget(Paragraph::new(Text::from(lines)), area);
}

fn render_text_input(frame: &mut Frame, text: &str, area: Rect) {
    let input_area = Rect {
        x: area.x,
        y: area.y + area.height.saturating_sub(1),
        width: area.width,
        height: 1,
    };
    let prompt = Line::from(vec![
        Span::styled("  New profile: ", Style::default().fg(GOLD)),
        Span::styled(text, Style::default().fg(TEXT_PRIMARY)),
        Span::styled("█", Style::default().fg(TEXT_PRIMARY)),
    ]);
    frame.render_widget(Paragraph::new(prompt), input_area);
}

fn render_confirm(frame: &mut Frame, action: &ConfirmAction, area: Rect) {
    let input_area = Rect {
        x: area.x,
        y: area.y + area.height.saturating_sub(1),
        width: area.width,
        height: 1,
    };
    let message = match action {
        ConfirmAction::DeleteProfile(name) => {
            format!("  Delete profile \"{name}\"? [y/n]")
        }
        ConfirmAction::OverwriteAliases { aliases, destination } => {
            let count = aliases.len();
            let dest = match destination {
                MoveDestination::Global => "global".to_string(),
                MoveDestination::Project => "project".to_string(),
                MoveDestination::Profile(name) => format!("profile \"{name}\""),
            };
            format!("  Move {count} alias(es) to {dest}, overwriting duplicates? [y/n]")
        }
    };
    let widget = Paragraph::new(message).style(Style::default().fg(GOLD));
    frame.render_widget(widget, input_area);
}

fn render_tree_lines(model: &TuiModel) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    for (i, node) in model.tree.iter().enumerate() {
        let is_cursor = i == model.cursor && model.active_column == Column::Left;
        let is_selected = node.alias_id.as_ref().is_some_and(|id| model.selected.contains(id));

        match &node.kind {
            NodeKind::GlobalHeader => {
                let marker = if is_cursor { "▸ " } else { "  " };
                lines.push(Line::from(vec![
                    Span::styled(format!("{}{marker}", node.prefix), Style::default().fg(TREE_CONNECTOR)),
                    Span::raw("🌐 "),
                    Span::styled("global", Style::default().fg(GOLD).bold()),
                ]));
            }
            NodeKind::ProjectHeader => {
                let marker = if is_cursor { "▸ " } else { "  " };
                lines.push(Line::from(vec![
                    Span::styled(format!("{}{marker}", node.prefix), Style::default().fg(TREE_CONNECTOR)),
                    Span::raw("📁 "),
                    Span::styled("project (.aliases)", Style::default().fg(GOLD).bold()),
                ]));
            }
            NodeKind::ProfileHeader => {
                let icon = if node.is_active { "●" } else { "○" };
                let marker = if is_cursor { "▸ " } else { "  " };
                let active_tag = if node.is_active { " (active)" } else { "" };

                // Connector line before child profiles
                if !node.prefix.is_empty() {
                    let connector_line = if node.prefix.ends_with("├─") {
                        let parent_cp = &node.prefix[..node.prefix.len() - "├─".len()];
                        format!("{parent_cp}│")
                    } else if node.prefix.ends_with("╰─") {
                        let parent_cp = &node.prefix[..node.prefix.len() - "╰─".len()];
                        format!("{parent_cp}│")
                    } else {
                        node.content_prefix.clone()
                    };
                    lines.push(Line::from(Span::styled(connector_line, Style::default().fg(TREE_CONNECTOR))));
                }

                let icon_color = if node.is_active { GOLD } else { TEXT_MUTED };
                lines.push(Line::from(vec![
                    Span::styled(format!("{}{marker}", node.prefix), Style::default().fg(TREE_CONNECTOR)),
                    Span::styled(format!("{icon} "), Style::default().fg(icon_color)),
                    Span::styled(
                        format!("{}{active_tag}", node.label),
                        Style::default().fg(if node.is_active { GOLD } else { TEXT_PRIMARY }).bold(),
                    ),
                ]));
            }
            NodeKind::AliasItem => {
                // Aliases branch off the parent's vertical line: ├─ or ╰─
                let is_last_alias = model.tree.get(i + 1)
                    .map_or(true, |next| next.kind != NodeKind::AliasItem);

                let arm = if is_last_alias { "╰─" } else { "├─" };
                let continuation = if is_last_alias { "  " } else { "│ " };

                let marker = if is_cursor {
                    "▸ "
                } else if is_selected {
                    "■ "
                } else {
                    "  "
                };

                let name_style = if is_selected {
                    Style::default().fg(GOLD)
                } else if is_cursor {
                    Style::default().fg(TEXT_PRIMARY)
                } else {
                    Style::default().fg(TEXT_PRIMARY)
                };

                // Name line: parent_prefix │ arm marker name
                // The content_prefix already ends with the parent's vertical connector.
                // We replace the trailing "│ " with the arm character.
                let alias_prefix = if node.content_prefix.ends_with("│ ") {
                    format!("{}{arm}", &node.content_prefix[..node.content_prefix.len() - "│ ".len()])
                } else {
                    format!("{}{arm}", node.content_prefix)
                };

                let cmd_prefix = if node.content_prefix.ends_with("│ ") {
                    format!("{}{continuation}", &node.content_prefix[..node.content_prefix.len() - "│ ".len()])
                } else {
                    format!("{}{continuation}", node.content_prefix)
                };

                lines.push(Line::from(vec![
                    Span::styled(format!("{alias_prefix}{marker}"), Style::default().fg(TREE_CONNECTOR)),
                    Span::styled(node.label.clone(), name_style),
                ]));

                // Command line (dimmed)
                if let Some(ref cmd) = node.alias_command {
                    lines.push(Line::from(vec![
                        Span::styled(format!("{cmd_prefix}  "), Style::default().fg(TREE_CONNECTOR)),
                        Span::styled(cmd.clone(), Style::default().fg(TEXT_MUTED)),
                    ]));
                }

                // Separator — keep the vertical going if not last
                if !is_last_alias {
                    lines.push(Line::from(Span::styled(
                        format!("{}│", &cmd_prefix[..cmd_prefix.len().saturating_sub(continuation.len())]),
                        Style::default().fg(TREE_CONNECTOR),
                    )));
                }
            }
        }
    }

    lines
}

fn help_bar_text(mode: &Mode) -> String {
    match mode {
        Mode::Normal => "  q quit  ␣ select  m move  n new  x delete  s activate".into(),
        Mode::Moving => "  Esc cancel  ↑↓ navigate  Enter move here  Tab switch column".into(),
        Mode::TextInput(_) => "  Esc cancel  Enter confirm".into(),
        Mode::Confirm(_) => "  y confirm  n cancel".into(),
    }
}
