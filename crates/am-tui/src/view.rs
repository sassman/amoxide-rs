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
const SELECTED_ACCENT: Color = Color::Rgb(208, 136, 74); // #d0884a — warm orange for selected marker/connectors
const SELECTED_ACCENT_MUTED: Color = Color::Rgb(154, 101, 53); // #9a6535 — muted orange for selected commands
const SELECTED_TEXT: Color = Color::Rgb(232, 232, 234); // #e8e8ea — bright white for selected alias names
const ERROR_RED: Color = Color::Rgb(220, 80, 80); // #dc5050 — error / validation feedback
const TRUST_WARN: Color = Color::Rgb(200, 160, 60); // amber — unknown/untrusted project
const TRUST_TAMPERED: Color = Color::Rgb(220, 80, 80); // red — tampered project
const SUBCOMMAND_COLOR: Color = Color::Rgb(80, 180, 160); // teal for subcommand nodes

pub fn draw(frame: &mut Frame, model: &TuiModel) {
    let area = frame.area();

    let help = Paragraph::new(help_bar(&model.mode, model));

    let has_status = model.status_line.is_some();
    let in_editor = matches!(model.mode, Mode::TextInput(_));

    // Build vertical layout dynamically:
    //   - status line only appears when there is a message
    //   - in editor mode, status (if any) sits above the editor line, both full-width
    let mut constraints = vec![
        Constraint::Length(1), // help bar
        Constraint::Length(1), // separator
        Constraint::Min(0),    // tree content
    ];
    if in_editor && has_status {
        constraints.push(Constraint::Length(1)); // status above editor
        constraints.push(Constraint::Length(1)); // editor
    } else if in_editor || has_status {
        constraints.push(Constraint::Length(1)); // editor or status (one row)
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    // chunk indices
    let content_chunk = 2;
    let (status_chunk, editor_chunk) = match (in_editor, has_status) {
        (true, true) => (Some(3), Some(4)),
        (true, false) => (None, Some(3)),
        (false, true) => (Some(3), None),
        (false, false) => (None, None),
    };

    frame.render_widget(help, chunks[0]);

    // Add 1-column padding on left and right
    let padded = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(chunks[content_chunk]);
    let content_area = padded[1];

    match &model.mode {
        Mode::Transfer(_) => {
            let columns = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
                .split(content_area);

            render_left_column(frame, model, columns[0]);
            render_right_column(frame, model, columns[1]);
            if let Some(i) = status_chunk {
                render_status(frame, model, chunks[i]);
            }
        }
        Mode::TextInput(state) => {
            render_left_column(frame, model, content_area);
            if let Some(i) = status_chunk {
                render_status(frame, model, chunks[i]);
            }
            if let Some(i) = editor_chunk {
                render_text_input(frame, state, chunks[i]);
            }
        }
        Mode::Confirm(action) => {
            render_left_column(frame, model, content_area);
            render_confirm(frame, action, content_area);
            if let Some(i) = status_chunk {
                render_status(frame, model, chunks[i]);
            }
        }
        Mode::Normal => {
            render_left_column(frame, model, content_area);
            if let Some(i) = status_chunk {
                render_status(frame, model, chunks[i]);
            }
        }
    }
}

fn render_status(frame: &mut Frame, model: &TuiModel, area: Rect) {
    if let Some(ref msg) = model.status_line {
        let status = Paragraph::new(ratatui::text::Span::styled(
            msg.clone(),
            Style::default().fg(TRUST_WARN),
        ));
        frame.render_widget(status, area);
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

/// Returns (icon, label) for a tree header node.
fn header_content(node: &TreeNode, activation_order: Option<usize>) -> (String, String) {
    match &node.kind {
        NodeKind::GlobalHeader => (ICON_GLOBAL.to_string(), "global".to_string()),
        NodeKind::ProjectHeader => (ICON_PROJECT.to_string(), node.label.clone()),
        NodeKind::ProfileHeader => {
            let icon = if node.is_active {
                ICON_ACTIVE
            } else {
                ICON_INACTIVE
            };
            let tag = match activation_order {
                Some(n) => format!(" (active: {n})"),
                None => String::new(),
            };
            (format!("{icon} "), format!("{}{tag}", node.label))
        }
        NodeKind::AliasItem
        | NodeKind::SubcommandProgramHeader
        | NodeKind::SubcommandGroupNode
        | NodeKind::SubcommandItem => unreachable!(),
    }
}

/// Returns (label_color, icon_color) for a tree header node.
fn header_colors(node: &TreeNode, is_cursor: bool) -> (Color, Color) {
    if is_cursor {
        return (GOLD, GOLD);
    }
    match &node.project_trust {
        Some(ProjectTrustState::Unknown) | Some(ProjectTrustState::Untrusted) => {
            return (TRUST_WARN, TRUST_WARN);
        }
        Some(ProjectTrustState::Tampered) => {
            return (TRUST_TAMPERED, TRUST_TAMPERED);
        }
        _ => {}
    }
    let highlight = node.kind == NodeKind::ProfileHeader && node.is_active;
    let label_color = if highlight { GOLD } else { HEADER_DEFAULT };
    let icon_color = match &node.kind {
        NodeKind::ProfileHeader if !highlight => TEXT_MUTED,
        _ => label_color,
    };
    (label_color, icon_color)
}

fn render_right_column(frame: &mut Frame, model: &TuiModel, area: Rect) {
    let title = match &model.mode {
        Mode::Transfer(TransferMode::Copy) => "-> Copy to",
        _ => "-> Move to",
    };
    let mut lines: Vec<Line<'static>> = Vec::new();
    lines.push(Line::from(Span::styled(
        title,
        Style::default().fg(HEADER_DEFAULT).bold(),
    )));
    lines.push(Line::from(""));

    for (i, node) in model.dest_tree.iter().enumerate() {
        if node.kind == NodeKind::AliasItem {
            continue;
        }
        let is_cursor = i == model.dest_cursor && model.active_column == Column::Right;
        let marker = if is_cursor {
            MARKER_CURSOR
        } else {
            MARKER_NONE
        };
        let conn = if is_cursor {
            TREE_CONNECTOR_ACTIVE
        } else {
            TREE_CONNECTOR
        };

        let (icon, label) =
            header_content(node, model.app_model.session.activation_order(&node.label));
        let (label_color, icon_color) = header_colors(node, is_cursor);

        lines.push(Line::from(vec![
            Span::styled(
                format!("{}{marker}", node.prefix),
                Style::default().fg(conn),
            ),
            Span::styled(icon, Style::default().fg(icon_color)),
            Span::styled(label, Style::default().fg(label_color).bold()),
        ]));
    }

    frame.render_widget(Paragraph::new(Text::from(lines)), area);
}

fn render_text_input(frame: &mut Frame, state: &TextInputState, area: Rect) {
    let input_area = area;
    let prompt = match state {
        TextInputState::NewProfile(text) => Line::from(vec![
            Span::styled("  New profile: ", Style::default().fg(GOLD)),
            Span::styled(text.as_str(), Style::default().fg(TEXT_PRIMARY)),
            Span::styled("_", Style::default().fg(TEXT_PRIMARY)),
        ]),
        TextInputState::NewAlias {
            name,
            command,
            active_field,
            cursor,
            target,
        } => {
            let target_label = match target {
                AliasTarget::Global => "global",
                AliasTarget::Project => "project",
                AliasTarget::Profile(p) => p.as_str(),
            };
            let name_active = *active_field == AliasField::Name;
            let cmd_active = *active_field == AliasField::Command;
            let name_style = if name_active {
                Style::default().fg(TEXT_PRIMARY)
            } else {
                Style::default().fg(TEXT_MUTED)
            };
            let cmd_style = if cmd_active {
                Style::default().fg(TEXT_PRIMARY)
            } else {
                Style::default().fg(TEXT_MUTED)
            };
            let pos = (*cursor).min(if name_active {
                name.len()
            } else {
                command.len()
            });
            // Show a placeholder hint when the name field is still empty so users
            // discover the "prog: → Tab" subcommand shortcut on first use.
            let hint = if name.is_empty() && name_active {
                Span::styled(
                    "  · e.g. ll, or git: for subcommand",
                    Style::default().fg(TEXT_MUTED),
                )
            } else {
                Span::raw("")
            };
            let mut spans = vec![Span::styled(
                format!("  [{target_label}] "),
                Style::default().fg(GOLD),
            )];
            if name_active {
                spans.push(Span::styled(name[..pos].to_string(), name_style));
                spans.push(Span::styled("_", Style::default().fg(TEXT_PRIMARY)));
                spans.push(Span::styled(name[pos..].to_string(), name_style));
            } else {
                spans.push(Span::styled(name.as_str(), name_style));
            }
            spans.push(Span::styled(" = ", Style::default().fg(TEXT_MUTED)));
            if cmd_active {
                spans.push(Span::styled(command[..pos].to_string(), cmd_style));
                spans.push(Span::styled("_", Style::default().fg(TEXT_PRIMARY)));
                spans.push(Span::styled(command[pos..].to_string(), cmd_style));
            } else {
                spans.push(Span::styled(command.as_str(), cmd_style));
            }
            spans.push(hint);
            Line::from(spans)
        }
        TextInputState::EditProfile { name, error, .. } => {
            let err_span = error
                .as_ref()
                .map(|e| Span::styled(format!("  ({e})"), Style::default().fg(ERROR_RED)))
                .unwrap_or_else(|| Span::raw(""));
            Line::from(vec![
                Span::styled("  Rename profile: ", Style::default().fg(GOLD)),
                Span::styled(name.as_str(), Style::default().fg(TEXT_PRIMARY)),
                Span::styled("_", Style::default().fg(TEXT_PRIMARY)),
                err_span,
            ])
        }
        TextInputState::SubcommandInput {
            program,
            pairs,
            active_pair,
            active_field,
            cursor,
            ..
        } => {
            let mut spans: Vec<Span<'_>> = vec![Span::styled(
                format!("  {program}: "),
                Style::default().fg(GOLD),
            )];
            for (i, (short, long)) in pairs.iter().enumerate() {
                if i > 0 {
                    spans.push(Span::styled(" › ", Style::default().fg(TEXT_MUTED)));
                }
                let is_active_pair = i == *active_pair;

                // Render short field
                let short_active = is_active_pair && *active_field == SubcommandField::Short;
                let short_style = if short_active {
                    Style::default().fg(TEXT_PRIMARY)
                } else {
                    Style::default().fg(TEXT_MUTED)
                };
                if short_active {
                    let pos = (*cursor).min(short.len());
                    spans.push(Span::styled(short[..pos].to_string(), short_style));
                    spans.push(Span::styled("_", Style::default().fg(TEXT_PRIMARY)));
                    spans.push(Span::styled(short[pos..].to_string(), short_style));
                } else {
                    spans.push(Span::styled(short.clone(), short_style));
                }

                spans.push(Span::styled(" → ", Style::default().fg(TEXT_MUTED)));

                // Render long field
                let long_active = is_active_pair && *active_field == SubcommandField::Long;
                let long_style = if long_active {
                    Style::default().fg(TEXT_PRIMARY)
                } else {
                    Style::default().fg(TEXT_MUTED)
                };
                if long_active {
                    let pos = (*cursor).min(long.len());
                    spans.push(Span::styled(long[..pos].to_string(), long_style));
                    spans.push(Span::styled("_", Style::default().fg(TEXT_PRIMARY)));
                    spans.push(Span::styled(long[pos..].to_string(), long_style));
                } else {
                    spans.push(Span::styled(long.clone(), long_style));
                }
            }
            spans.push(Span::styled(
                "   (Tab: next, Shift+Tab: back, ↵: confirm, Esc: cancel)",
                Style::default().fg(TEXT_MUTED),
            ));
            Line::from(spans)
        }
        TextInputState::EditAlias {
            alias_id,
            name,
            command,
            active_field,
            cursor,
            error,
        } => {
            let scope_label = match alias_id {
                AliasId::Global { .. } => "global",
                AliasId::Profile { profile_name, .. } => profile_name.as_str(),
                AliasId::Project { .. } => "project",
                AliasId::Subcommand { .. } => "subcmd",
            };
            let name_active = *active_field == AliasField::Name;
            let cmd_active = *active_field == AliasField::Command;
            let name_style = if name_active {
                Style::default().fg(TEXT_PRIMARY)
            } else {
                Style::default().fg(TEXT_MUTED)
            };
            let cmd_style = if cmd_active {
                Style::default().fg(TEXT_PRIMARY)
            } else {
                Style::default().fg(TEXT_MUTED)
            };
            let pos = (*cursor).min(if name_active {
                name.len()
            } else {
                command.len()
            });
            let err_span = error
                .as_ref()
                .map(|e| Span::styled(format!("  ({e})"), Style::default().fg(ERROR_RED)))
                .unwrap_or_else(|| Span::raw(""));
            let mut spans = vec![Span::styled(
                format!("  [{scope_label}] "),
                Style::default().fg(GOLD),
            )];
            if name_active {
                spans.push(Span::styled(name[..pos].to_string(), name_style));
                spans.push(Span::styled("_", Style::default().fg(TEXT_PRIMARY)));
                spans.push(Span::styled(name[pos..].to_string(), name_style));
            } else {
                spans.push(Span::styled(name.as_str(), name_style));
            }
            spans.push(Span::styled(" = ", Style::default().fg(TEXT_MUTED)));
            if cmd_active {
                spans.push(Span::styled(command[..pos].to_string(), cmd_style));
                spans.push(Span::styled("_", Style::default().fg(TEXT_PRIMARY)));
                spans.push(Span::styled(command[pos..].to_string(), cmd_style));
            } else {
                spans.push(Span::styled(command.as_str(), cmd_style));
            }
            spans.push(err_span);
            Line::from(spans)
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
            transfer_mode,
        } => {
            let count = aliases.len();
            let verb = match transfer_mode {
                TransferMode::Move => "Move",
                TransferMode::Copy => "Copy",
            };
            let dest = match destination {
                MoveDestination::Global => "global".to_string(),
                MoveDestination::Project => "project".to_string(),
                MoveDestination::Profile(name) => format!("profile \"{name}\""),
            };
            format!("  {verb} {count} alias(es) to {dest}, overwriting duplicates? [y/n]")
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
            NodeKind::GlobalHeader | NodeKind::ProjectHeader | NodeKind::ProfileHeader => {
                let marker = if is_cursor {
                    MARKER_CURSOR
                } else {
                    MARKER_NONE
                };
                let conn = if is_cursor {
                    TREE_CONNECTOR_ACTIVE
                } else {
                    TREE_CONNECTOR
                };
                let (icon, label) =
                    header_content(node, model.app_model.session.activation_order(&node.label));
                let (label_color, icon_color) = header_colors(node, is_cursor);

                lines.push(Line::from(vec![
                    Span::styled(
                        format!("{}{marker}", node.prefix),
                        Style::default().fg(conn),
                    ),
                    Span::styled(icon, Style::default().fg(icon_color)),
                    Span::styled(label, Style::default().fg(label_color).bold()),
                ]));

                // Breathing room after an empty section (no alias items follow before the next header)
                if node.kind == NodeKind::ProfileHeader {
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
            NodeKind::SubcommandProgramHeader => {
                let marker = if is_cursor {
                    MARKER_CURSOR
                } else {
                    MARKER_NONE
                };
                let conn = if is_cursor {
                    TREE_CONNECTOR_ACTIVE
                } else {
                    SUBCOMMAND_COLOR
                };
                let label_color = if is_cursor { GOLD } else { SUBCOMMAND_COLOR };
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("{}{marker}", node.prefix),
                        Style::default().fg(conn),
                    ),
                    Span::styled(
                        format!("{ICON_SUBCOMMAND} "),
                        Style::default().fg(label_color),
                    ),
                    Span::styled(node.label.clone(), Style::default().fg(label_color).bold()),
                ]));
            }
            NodeKind::SubcommandGroupNode => {
                let marker = if is_cursor {
                    MARKER_CURSOR
                } else if is_selected {
                    MARKER_SELECTED
                } else {
                    MARKER_NONE
                };
                let conn = if is_cursor {
                    TREE_CONNECTOR_ACTIVE
                } else {
                    SUBCOMMAND_COLOR
                };
                let marker_style = if is_selected {
                    Style::default().fg(SELECTED_ACCENT)
                } else {
                    Style::default().fg(conn)
                };
                let label_color = if is_cursor {
                    GOLD
                } else if is_selected {
                    SELECTED_TEXT
                } else {
                    SUBCOMMAND_COLOR
                };
                lines.push(Line::from(vec![
                    Span::styled(node.prefix.clone(), Style::default().fg(conn)),
                    Span::styled(marker.to_string(), marker_style),
                    Span::styled(node.label.clone(), Style::default().fg(label_color)),
                ]));
            }
            NodeKind::SubcommandItem => {
                let marker = if is_cursor {
                    MARKER_CURSOR
                } else if is_selected {
                    MARKER_SELECTED
                } else {
                    MARKER_NONE
                };
                let conn = if is_cursor {
                    TREE_CONNECTOR_ACTIVE
                } else if is_selected {
                    SELECTED_ACCENT_MUTED
                } else {
                    SUBCOMMAND_COLOR
                };
                let marker_style = if is_selected {
                    Style::default().fg(SELECTED_ACCENT)
                } else {
                    Style::default().fg(conn)
                };
                let key_color = if is_cursor {
                    GOLD
                } else if is_selected {
                    SELECTED_TEXT
                } else {
                    SUBCOMMAND_COLOR
                };
                let exp_color = if is_cursor {
                    HEADER_DEFAULT
                } else if is_selected {
                    SELECTED_ACCENT_MUTED
                } else {
                    TEXT_MUTED
                };
                let (key_span, arrow_span, exp_span) =
                    if let Some((key, exp)) = node.label.split_once(" \u{2192} ") {
                        (
                            Span::styled(key.to_string(), Style::default().fg(key_color)),
                            Span::styled(" \u{2192} ", Style::default().fg(TEXT_MUTED)),
                            Span::styled(exp.to_string(), Style::default().fg(exp_color)),
                        )
                    } else {
                        (
                            Span::styled(node.label.clone(), Style::default().fg(key_color)),
                            Span::raw(""),
                            Span::raw(""),
                        )
                    };
                lines.push(Line::from(vec![
                    Span::styled(node.prefix.clone(), Style::default().fg(conn)),
                    Span::styled(marker.to_string(), marker_style),
                    key_span,
                    arrow_span,
                    exp_span,
                ]));

                // Breathing room between sections: when the last subcommand item in a group
                // is immediately followed by a section header, emit a blank separator line.
                let next_is_section_header = model.tree.get(i + 1).is_some_and(|n| {
                    matches!(
                        n.kind,
                        NodeKind::GlobalHeader | NodeKind::ProjectHeader | NodeKind::ProfileHeader
                    )
                });
                if next_is_section_header {
                    lines.push(Line::from(Span::styled(
                        node.prefix
                            .chars()
                            .map(|c| if c == '│' { '│' } else { ' ' })
                            .collect::<String>(),
                        Style::default().fg(TREE_CONNECTOR),
                    )));
                }
            }
            NodeKind::AliasItem => {
                let is_last_alias = model.tree.get(i + 1).is_none_or(|next| {
                    !matches!(
                        next.kind,
                        NodeKind::AliasItem | NodeKind::SubcommandProgramHeader
                    )
                });

                let arm = if is_last_alias {
                    TREE_LAST
                } else {
                    TREE_BRANCH
                };

                let marker = if is_cursor {
                    MARKER_CURSOR
                } else if is_selected {
                    MARKER_SELECTED
                } else {
                    MARKER_NONE
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

                // Single line: prefix arm marker name -> command
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
                        format!("{}{arm}", node.content_prefix),
                        Style::default().fg(conn),
                    ),
                    Span::styled(marker.to_string(), marker_style),
                    Span::styled(node.label.clone(), name_style),
                    Span::styled(" -> ", Style::default().fg(TEXT_MUTED)),
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
                                | NodeKind::SubcommandProgramHeader
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

fn help_bar(mode: &Mode, model: &TuiModel) -> Line<'static> {
    match mode {
        Mode::Normal => {
            let cursor_node = model.tree.get(model.cursor);
            let on_project = cursor_node.is_some_and(|n| n.kind == NodeKind::ProjectHeader);
            let on_profile = cursor_node.is_some_and(|n| n.kind == NodeKind::ProfileHeader);
            let profile_is_active = on_profile && cursor_node.is_some_and(|n| n.is_active);

            let mut spans = vec![
                Span::raw("  "),
                Span::styled("q", Style::default().fg(GOLD)),
                Span::styled(" quit  ", Style::default().fg(TEXT_MUTED)),
                Span::styled("a", Style::default().fg(GOLD)),
                Span::styled(" add  ", Style::default().fg(TEXT_MUTED)),
                Span::styled("Space", Style::default().fg(GOLD)),
                Span::styled(" select  ", Style::default().fg(TEXT_MUTED)),
                Span::styled("m", Style::default().fg(GOLD)),
                Span::styled(" move  ", Style::default().fg(TEXT_MUTED)),
                Span::styled("c", Style::default().fg(GOLD)),
                Span::styled(" copy  ", Style::default().fg(TEXT_MUTED)),
                Span::styled("n", Style::default().fg(GOLD)),
                Span::styled(" new profile  ", Style::default().fg(TEXT_MUTED)),
                Span::styled("x", Style::default().fg(GOLD)),
                Span::styled(" delete  ", Style::default().fg(TEXT_MUTED)),
                Span::styled("e", Style::default().fg(GOLD)),
                Span::styled(" edit", Style::default().fg(TEXT_MUTED)),
            ];
            if on_profile {
                let use_label = if profile_is_active { " unuse" } else { " use" };
                spans.push(Span::styled("  u", Style::default().fg(GOLD)));
                spans.push(Span::styled(use_label, Style::default().fg(TEXT_MUTED)));
            }
            if on_project {
                let project_is_trusted = cursor_node
                    .and_then(|n| n.project_trust.as_ref())
                    .is_some_and(|t| matches!(t, ProjectTrustState::Trusted));
                let trust_label = if project_is_trusted {
                    " untrust"
                } else {
                    " trust"
                };
                spans.push(Span::styled("  t", Style::default().fg(GOLD)));
                spans.push(Span::styled(trust_label, Style::default().fg(TEXT_MUTED)));
            }
            Line::from(spans)
        }
        Mode::Transfer(TransferMode::Move) => Line::from(vec![
            Span::raw("  "),
            Span::styled("Esc", Style::default().fg(GOLD)),
            Span::styled(" cancel  ", Style::default().fg(TEXT_MUTED)),
            Span::styled("jk/↑↓", Style::default().fg(GOLD)),
            Span::styled(" navigate  ", Style::default().fg(TEXT_MUTED)),
            Span::styled("Enter", Style::default().fg(GOLD)),
            Span::styled(" move here  ", Style::default().fg(TEXT_MUTED)),
            Span::styled("Tab", Style::default().fg(GOLD)),
            Span::styled(" switch column", Style::default().fg(TEXT_MUTED)),
        ]),
        Mode::Transfer(TransferMode::Copy) => Line::from(vec![
            Span::raw("  "),
            Span::styled("Esc", Style::default().fg(GOLD)),
            Span::styled(" cancel  ", Style::default().fg(TEXT_MUTED)),
            Span::styled("jk/↑↓", Style::default().fg(GOLD)),
            Span::styled(" navigate  ", Style::default().fg(TEXT_MUTED)),
            Span::styled("Enter", Style::default().fg(GOLD)),
            Span::styled(" copy here  ", Style::default().fg(TEXT_MUTED)),
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
        Mode::TextInput(TextInputState::NewAlias { name, .. }) => {
            let tab_hint = if name.contains(':') {
                " → subcommand mode  "
            } else {
                " switch field  "
            };
            Line::from(vec![
                Span::raw("  "),
                Span::styled("Tab", Style::default().fg(GOLD)),
                Span::styled(tab_hint, Style::default().fg(TEXT_MUTED)),
                Span::styled("Esc", Style::default().fg(GOLD)),
                Span::styled(" cancel  ", Style::default().fg(TEXT_MUTED)),
                Span::styled("Enter", Style::default().fg(GOLD)),
                Span::styled(" confirm", Style::default().fg(TEXT_MUTED)),
            ])
        }
        Mode::TextInput(TextInputState::EditProfile { .. }) => Line::from(vec![
            Span::raw("  "),
            Span::styled("Esc", Style::default().fg(GOLD)),
            Span::styled(" cancel  ", Style::default().fg(TEXT_MUTED)),
            Span::styled("Enter", Style::default().fg(GOLD)),
            Span::styled(" confirm", Style::default().fg(TEXT_MUTED)),
        ]),
        Mode::TextInput(TextInputState::EditAlias { .. }) => Line::from(vec![
            Span::raw("  "),
            Span::styled("Tab", Style::default().fg(GOLD)),
            Span::styled(" switch field  ", Style::default().fg(TEXT_MUTED)),
            Span::styled("Esc", Style::default().fg(GOLD)),
            Span::styled(" cancel  ", Style::default().fg(TEXT_MUTED)),
            Span::styled("Enter", Style::default().fg(GOLD)),
            Span::styled(" confirm", Style::default().fg(TEXT_MUTED)),
        ]),
        Mode::TextInput(TextInputState::SubcommandInput { .. }) => Line::from(vec![
            Span::raw("  "),
            Span::styled("Tab/←→", Style::default().fg(GOLD)),
            Span::styled(" switch field/pair  ", Style::default().fg(TEXT_MUTED)),
            Span::styled("a", Style::default().fg(GOLD)),
            Span::styled(" add pair  ", Style::default().fg(TEXT_MUTED)),
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

#[cfg(test)]
mod subcommand_render {
    use super::*;
    use crate::model::TuiModel;
    use amoxide::{Config, ProfileConfig};

    fn make_model_with_subcommand() -> TuiModel {
        let mut config = Config::default();
        config
            .subcommands
            .as_mut()
            .insert("jj:ab".into(), vec!["abandon".into()]);
        let app = amoxide::update::AppModel::new(config, ProfileConfig::default());
        let mut model = TuiModel::new().unwrap();
        model.app_model = app;
        model.rebuild_tree();
        model
    }

    #[test]
    fn subcommand_program_header_renders_with_diamond() {
        let model = make_model_with_subcommand();
        let lines = render_tree_lines(&model);
        let rendered: String = lines
            .iter()
            .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref()))
            .collect();
        assert!(rendered.contains("◆"), "expected ◆ diamond marker");
        assert!(rendered.contains("jj (subcommands)"));
    }

    #[test]
    fn subcommand_item_renders_arrow() {
        let model = make_model_with_subcommand();
        let lines = render_tree_lines(&model);
        let rendered: String = lines
            .iter()
            .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref()))
            .collect();
        assert!(rendered.contains("ab"));
        assert!(rendered.contains("abandon"));
    }
}
