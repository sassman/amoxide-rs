use crate::model::*;
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

// Noctavox-inspired warm color palette
const TEXT_PRIMARY: Color = Color::Rgb(210, 210, 213); // #d2d2d5
const TEXT_MUTED: Color = Color::Rgb(100, 100, 103); // #646467
const GOLD: Color = Color::Rgb(220, 220, 100); // #dcdc64
const HEADER_DEFAULT: Color = Color::Rgb(190, 185, 170); // warm beige for inactive headers
const TREE_CONNECTOR: Color = Color::Rgb(70, 70, 73); // dim connector lines
const TREE_CONNECTOR_ACTIVE: Color = Color::Rgb(150, 150, 80); // brighter connectors for cursor row
const SELECTED_ACCENT: Color = Color::Rgb(208, 136, 74); // #d0884a — warm orange for selected ■ marker/connectors
const SELECTED_ACCENT_MUTED: Color = Color::Rgb(154, 101, 53); // #9a6535 — muted orange for selected commands
const SELECTED_TEXT: Color = Color::Rgb(232, 232, 234); // #e8e8ea — bright white for selected alias names

pub fn draw(frame: &mut Frame, model: &TuiModel) {
    let area = frame.area();

    let help = Paragraph::new(help_bar(&model.mode));

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(area);

    frame.render_widget(help, chunks[0]);

    // Add 1-column padding on left and right
    let padded = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(chunks[2]);
    let content_area = padded[1];

    match &model.mode {
        Mode::Moving => {
            let columns = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
                .split(content_area);

            render_left_column(frame, model, columns[0]);
            render_right_column(frame, model, columns[1]);
        }
        Mode::TextInput(state) => {
            render_left_column(frame, model, content_area);
            render_text_input(frame, state, content_area);
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
        "→ Move to",
        Style::default().fg(HEADER_DEFAULT).bold(),
    )));
    lines.push(Line::from(""));

    for (i, node) in model.dest_tree.iter().enumerate() {
        let is_cursor = i == model.dest_cursor && model.active_column == Column::Right;
        let marker = if is_cursor { "▸ " } else { "  " };
        let conn = if is_cursor {
            TREE_CONNECTOR_ACTIVE
        } else {
            TREE_CONNECTOR
        };

        match &node.kind {
            NodeKind::GlobalHeader => {
                let label_color = if is_cursor { GOLD } else { HEADER_DEFAULT };
                lines.push(Line::from(vec![
                    Span::raw("🌐 "),
                    Span::styled("global", Style::default().fg(label_color).bold()),
                ]));
            }
            NodeKind::ProjectHeader => {
                let label_color = if is_cursor { GOLD } else { HEADER_DEFAULT };
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("{}{marker}", node.prefix),
                        Style::default().fg(conn),
                    ),
                    Span::raw("📁 "),
                    Span::styled(
                        "project (.aliases)",
                        Style::default().fg(label_color).bold(),
                    ),
                ]));
            }
            NodeKind::ProfileHeader => {
                let icon = if node.is_active { "●" } else { "○" };
                let active_tag = if node.is_active { " (active)" } else { "" };
                let color = if is_cursor || node.is_active {
                    GOLD
                } else {
                    HEADER_DEFAULT
                };
                let icon_color = if is_cursor || node.is_active {
                    GOLD
                } else {
                    TEXT_MUTED
                };

                lines.push(Line::from(vec![
                    Span::styled(
                        format!("{}{marker}", node.prefix),
                        Style::default().fg(conn),
                    ),
                    Span::styled(format!("{icon} "), Style::default().fg(icon_color)),
                    Span::styled(
                        format!("{}{active_tag}", node.label),
                        Style::default().fg(color).bold(),
                    ),
                ]));
            }
            NodeKind::AliasItem => {}
        }
    }

    frame.render_widget(Paragraph::new(Text::from(lines)), area);
}

fn render_text_input(frame: &mut Frame, state: &TextInputState, area: Rect) {
    let input_area = Rect {
        x: area.x,
        y: area.y + area.height.saturating_sub(1),
        width: area.width,
        height: 1,
    };
    let prompt = match state {
        TextInputState::NewProfile(text) => Line::from(vec![
            Span::styled("  New profile: ", Style::default().fg(GOLD)),
            Span::styled(text.as_str(), Style::default().fg(TEXT_PRIMARY)),
            Span::styled("█", Style::default().fg(TEXT_PRIMARY)),
        ]),
        TextInputState::NewAlias {
            name,
            command,
            active_field,
            target,
        } => {
            let target_label = match target {
                AliasTarget::Global => "global",
                AliasTarget::Project => "project",
                AliasTarget::Profile(p) => p.as_str(),
            };
            let name_style = if *active_field == AliasField::Name {
                Style::default().fg(TEXT_PRIMARY)
            } else {
                Style::default().fg(TEXT_MUTED)
            };
            let cmd_style = if *active_field == AliasField::Command {
                Style::default().fg(TEXT_PRIMARY)
            } else {
                Style::default().fg(TEXT_MUTED)
            };
            let cursor_after_name = *active_field == AliasField::Name;
            let cursor_after_cmd = *active_field == AliasField::Command;
            Line::from(vec![
                Span::styled(format!("  [{target_label}] "), Style::default().fg(GOLD)),
                Span::styled(name.as_str(), name_style),
                if cursor_after_name {
                    Span::styled("█", Style::default().fg(TEXT_PRIMARY))
                } else {
                    Span::raw("")
                },
                Span::styled(" = ", Style::default().fg(TEXT_MUTED)),
                Span::styled(command.as_str(), cmd_style),
                if cursor_after_cmd {
                    Span::styled("█", Style::default().fg(TEXT_PRIMARY))
                } else {
                    Span::raw("")
                },
            ])
        }
    };
    frame.render_widget(ratatui::widgets::Clear, input_area);
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
        ConfirmAction::OverwriteAliases {
            aliases,
            destination,
        } => {
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
    frame.render_widget(ratatui::widgets::Clear, input_area);
    frame.render_widget(widget, input_area);
}

fn render_tree_lines(model: &TuiModel) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    for (i, node) in model.tree.iter().enumerate() {
        let is_cursor = i == model.cursor && model.active_column == Column::Left;
        let is_selected = node
            .alias_id
            .as_ref()
            .is_some_and(|id| model.selected.contains(id));

        match &node.kind {
            NodeKind::GlobalHeader => {
                let label_color = if is_cursor { GOLD } else { HEADER_DEFAULT };
                lines.push(Line::from(vec![
                    Span::raw("🌐 "),
                    Span::styled("global", Style::default().fg(label_color).bold()),
                ]));
            }
            NodeKind::ProjectHeader => {
                let marker = if is_cursor { "▸ " } else { "  " };
                let conn = if is_cursor {
                    TREE_CONNECTOR_ACTIVE
                } else {
                    TREE_CONNECTOR
                };
                let label_color = if is_cursor { GOLD } else { HEADER_DEFAULT };
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("{}{marker}", node.prefix),
                        Style::default().fg(conn),
                    ),
                    Span::raw("📁 "),
                    Span::styled(
                        "project (.aliases)",
                        Style::default().fg(label_color).bold(),
                    ),
                ]));
            }
            NodeKind::ProfileHeader => {
                let icon = if node.is_active { "●" } else { "○" };
                let marker = if is_cursor { "▸ " } else { "  " };
                let active_tag = if node.is_active { " (active)" } else { "" };

                let conn = if is_cursor {
                    TREE_CONNECTOR_ACTIVE
                } else {
                    TREE_CONNECTOR
                };
                let color = if is_cursor || node.is_active {
                    GOLD
                } else {
                    HEADER_DEFAULT
                };
                let icon_color = if is_cursor || node.is_active {
                    GOLD
                } else {
                    TEXT_MUTED
                };
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("{}{marker}", node.prefix),
                        Style::default().fg(conn),
                    ),
                    Span::styled(format!("{icon} "), Style::default().fg(icon_color)),
                    Span::styled(
                        format!("{}{active_tag}", node.label),
                        Style::default().fg(color).bold(),
                    ),
                ]));
            }
            NodeKind::AliasItem => {
                let is_last_alias = model
                    .tree
                    .get(i + 1)
                    .is_none_or(|next| next.kind != NodeKind::AliasItem);

                let arm = if is_last_alias { "╰─" } else { "├─" };

                let marker = if is_cursor {
                    "▸ "
                } else if is_selected {
                    "■ "
                } else {
                    "  "
                };

                let conn = if is_cursor {
                    TREE_CONNECTOR_ACTIVE
                } else if is_selected {
                    SELECTED_ACCENT_MUTED
                } else {
                    TREE_CONNECTOR
                };
                let name_style = if is_cursor {
                    Style::default().fg(GOLD).bold()
                } else if is_selected {
                    Style::default().fg(SELECTED_TEXT).bold()
                } else {
                    Style::default().fg(TEXT_PRIMARY)
                };

                let marker_style = if is_selected {
                    Style::default().fg(SELECTED_ACCENT)
                } else {
                    Style::default().fg(conn)
                };

                // Single line: prefix arm marker name → command
                let cmd_text = node.alias_command.as_deref().unwrap_or("");
                let cmd_style = if is_cursor {
                    Style::default().fg(HEADER_DEFAULT)
                } else if is_selected {
                    Style::default().fg(SELECTED_ACCENT_MUTED)
                } else {
                    Style::default().fg(TEXT_MUTED)
                };
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("{}  {arm}", node.content_prefix),
                        Style::default().fg(conn),
                    ),
                    Span::styled(marker.to_string(), marker_style),
                    Span::styled(node.label.clone(), name_style),
                    Span::styled(" → ", Style::default().fg(TEXT_MUTED)),
                    Span::styled(cmd_text.to_string(), cmd_style),
                ]));

                // Breathing room between sections
                if is_last_alias {
                    let next_is_header = model.tree.get(i + 1).is_some_and(|n| {
                        matches!(
                            n.kind,
                            NodeKind::GlobalHeader
                                | NodeKind::ProjectHeader
                                | NodeKind::ProfileHeader
                        )
                    });
                    if next_is_header {
                        lines.push(Line::from(Span::styled(
                            node.content_prefix.clone(),
                            Style::default().fg(TREE_CONNECTOR),
                        )));
                    }
                }
            }
        }
    }

    lines
}

fn help_bar(mode: &Mode) -> Line<'static> {
    match mode {
        Mode::Normal => Line::from(vec![
            Span::raw("  "),
            Span::styled("q", Style::default().fg(GOLD)),
            Span::styled(" quit  ", Style::default().fg(TEXT_MUTED)),
            Span::styled("a", Style::default().fg(GOLD)),
            Span::styled(" add  ", Style::default().fg(TEXT_MUTED)),
            Span::styled("␣", Style::default().fg(GOLD)),
            Span::styled(" select  ", Style::default().fg(TEXT_MUTED)),
            Span::styled("m", Style::default().fg(GOLD)),
            Span::styled(" move  ", Style::default().fg(TEXT_MUTED)),
            Span::styled("n", Style::default().fg(GOLD)),
            Span::styled(" new profile  ", Style::default().fg(TEXT_MUTED)),
            Span::styled("x", Style::default().fg(GOLD)),
            Span::styled(" delete  ", Style::default().fg(TEXT_MUTED)),
            Span::styled("s", Style::default().fg(GOLD)),
            Span::styled(" activate", Style::default().fg(TEXT_MUTED)),
        ]),
        Mode::Moving => Line::from(vec![
            Span::raw("  "),
            Span::styled("Esc", Style::default().fg(GOLD)),
            Span::styled(" cancel  ", Style::default().fg(TEXT_MUTED)),
            Span::styled("↑↓", Style::default().fg(GOLD)),
            Span::styled(" navigate  ", Style::default().fg(TEXT_MUTED)),
            Span::styled("Enter", Style::default().fg(GOLD)),
            Span::styled(" move here  ", Style::default().fg(TEXT_MUTED)),
            Span::styled("Tab", Style::default().fg(GOLD)),
            Span::styled(" switch column", Style::default().fg(TEXT_MUTED)),
        ]),
        Mode::TextInput(TextInputState::NewProfile(_)) => Line::from(vec![
            Span::raw("  "),
            Span::styled("Esc", Style::default().fg(GOLD)),
            Span::styled(" cancel  ", Style::default().fg(TEXT_MUTED)),
            Span::styled("Enter", Style::default().fg(GOLD)),
            Span::styled(" confirm", Style::default().fg(TEXT_MUTED)),
        ]),
        Mode::TextInput(TextInputState::NewAlias { .. }) => Line::from(vec![
            Span::raw("  "),
            Span::styled("Tab", Style::default().fg(GOLD)),
            Span::styled(" switch field  ", Style::default().fg(TEXT_MUTED)),
            Span::styled("Esc", Style::default().fg(GOLD)),
            Span::styled(" cancel  ", Style::default().fg(TEXT_MUTED)),
            Span::styled("Enter", Style::default().fg(GOLD)),
            Span::styled(" confirm", Style::default().fg(TEXT_MUTED)),
        ]),
        Mode::Confirm(_) => Line::from(vec![
            Span::raw("  "),
            Span::styled("y", Style::default().fg(GOLD)),
            Span::styled(" confirm  ", Style::default().fg(TEXT_MUTED)),
            Span::styled("n", Style::default().fg(GOLD)),
            Span::styled(" cancel", Style::default().fg(TEXT_MUTED)),
        ]),
    }
}
