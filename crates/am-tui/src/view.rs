use ratatui::prelude::*;
use ratatui::widgets::Paragraph;
use ratatui::Frame;
use crate::model::*;

pub fn draw(frame: &mut Frame, model: &TuiModel) {
    let area = frame.area();

    let help_text = help_bar_text(&model.mode);
    let help = Paragraph::new(help_text).style(Style::default().fg(Color::DarkGray));

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
        Style::default().fg(Color::DarkGray),
    )));
    lines.push(Line::from(""));

    for (i, node) in model.dest_tree.iter().enumerate() {
        let is_cursor = i == model.dest_cursor && model.active_column == Column::Right;
        let marker = if is_cursor { "▸ " } else { "  " };

        match &node.kind {
            NodeKind::GlobalHeader => {
                lines.push(Line::from(vec![
                    Span::raw(format!("{}{marker}", node.prefix)),
                    Span::raw("🌐 "),
                    Span::styled("global", Style::default().bold()),
                ]));
                // Blank connector line
                lines.push(Line::from(Span::raw(node.content_prefix.clone())));
            }
            NodeKind::ProjectHeader => {
                lines.push(Line::from(vec![
                    Span::raw(format!("{}{marker}", node.prefix)),
                    Span::raw("📁 "),
                    Span::styled("project (.aliases)", Style::default().bold()),
                ]));
                // Blank connector line
                lines.push(Line::from(Span::raw(node.content_prefix.clone())));
            }
            NodeKind::ProfileHeader => {
                let icon = if node.is_active { "●" } else { "○" };
                let active_tag = if node.is_active { " (active)" } else { "" };

                // If this is a child profile (non-empty prefix with connector),
                // add a connector line before to visually connect to parent.
                if !node.prefix.is_empty() {
                    // The connector line uses the parent's content_prefix area
                    // plus "│" to keep the vertical line going.
                    let connector_line = if node.prefix.ends_with("├─") {
                        // parent_content_prefix is everything before the connector
                        let parent_cp = &node.prefix[..node.prefix.len() - "├─".len()];
                        format!("{parent_cp}│")
                    } else if node.prefix.ends_with("╰─") {
                        let parent_cp = &node.prefix[..node.prefix.len() - "╰─".len()];
                        format!("{parent_cp}│")
                    } else {
                        node.content_prefix.clone()
                    };
                    lines.push(Line::from(Span::raw(connector_line)));
                }

                lines.push(Line::from(vec![
                    Span::raw(format!("{}{marker}", node.prefix)),
                    Span::styled(
                        format!("{icon} {}{active_tag}", node.label),
                        Style::default().bold(),
                    ),
                ]));
                // Blank connector line after header
                lines.push(Line::from(Span::raw(node.content_prefix.clone())));
            }
            NodeKind::AliasItem => {
                // AliasItem nodes are skipped in the dest tree
            }
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
    let display = format!("New profile: {text}█");
    let widget = Paragraph::new(display).style(Style::default().fg(Color::White));
    frame.render_widget(widget, input_area);
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
            format!("Delete profile \"{name}\"? [y/n]")
        }
        ConfirmAction::OverwriteAliases { aliases, destination } => {
            let count = aliases.len();
            let dest = match destination {
                MoveDestination::Global => "global".to_string(),
                MoveDestination::Project => "project".to_string(),
                MoveDestination::Profile(name) => format!("profile \"{name}\""),
            };
            format!("Move {count} alias(es) to {dest}, overwriting duplicates? [y/n]")
        }
    };
    let widget = Paragraph::new(message).style(Style::default().fg(Color::Yellow));
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
                    Span::raw(format!("{}{marker}", node.prefix)),
                    Span::raw("🌐 "),
                    Span::styled("global", Style::default().bold()),
                ]));
                // Blank connector line after header
                lines.push(Line::from(Span::raw(node.content_prefix.clone())));
            }
            NodeKind::ProjectHeader => {
                let marker = if is_cursor { "▸ " } else { "  " };
                lines.push(Line::from(vec![
                    Span::raw(format!("{}{marker}", node.prefix)),
                    Span::raw("📁 "),
                    Span::styled("project (.aliases)", Style::default().bold()),
                ]));
                // Blank connector line after header
                lines.push(Line::from(Span::raw(node.content_prefix.clone())));
            }
            NodeKind::ProfileHeader => {
                let icon = if node.is_active { "●" } else { "○" };
                let marker = if is_cursor { "▸ " } else { "  " };
                let active_tag = if node.is_active { " (active)" } else { "" };

                // If this is a child profile (non-empty prefix with connector),
                // add a connector line before to visually connect to parent.
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
                    lines.push(Line::from(Span::raw(connector_line)));
                }

                lines.push(Line::from(vec![
                    Span::raw(format!("{}{marker}", node.prefix)),
                    Span::styled(
                        format!("{icon} {}{active_tag}", node.label),
                        Style::default().bold(),
                    ),
                ]));
                // Blank connector line after header
                lines.push(Line::from(Span::raw(node.content_prefix.clone())));
            }
            NodeKind::AliasItem => {
                let marker = if is_cursor {
                    "▸ "
                } else if is_selected {
                    "■ "
                } else {
                    "  "
                };

                let name_style = if is_selected {
                    Style::default().fg(Color::Yellow)
                } else if is_cursor {
                    Style::default().fg(Color::White)
                } else {
                    Style::default()
                };

                // Check if this is the last alias before a non-alias node
                let is_last_alias = model.tree.get(i + 1)
                    .map_or(true, |next| next.kind != NodeKind::AliasItem);

                let arm = if is_last_alias { "╰─" } else { "├─" };
                let arm_continuation = if is_last_alias { "  " } else { "│ " };

                // Name line: content_prefix + arm + marker + name
                lines.push(Line::from(vec![
                    Span::raw(format!("{}{arm}{marker}", node.content_prefix)),
                    Span::styled(node.label.clone(), name_style),
                ]));

                // Command line: content_prefix + arm_continuation + command (dimmed)
                if let Some(ref cmd) = node.alias_command {
                    lines.push(Line::from(vec![
                        Span::raw(format!("{}{arm_continuation}  ", node.content_prefix)),
                        Span::styled(cmd.clone(), Style::default().fg(Color::DarkGray)),
                    ]));
                }

                // Blank separator line
                if !is_last_alias {
                    lines.push(Line::from(Span::raw(format!("{}│", node.content_prefix))));
                } else {
                    lines.push(Line::from(Span::raw(node.content_prefix.clone())));
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
