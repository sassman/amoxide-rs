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

    let tree_lines = render_tree_lines(model);
    let visible_height = chunks[1].height as usize;
    let start = model.scroll_offset;
    let end = (start + visible_height).min(tree_lines.len());
    let visible: Vec<Line> = if start < tree_lines.len() {
        tree_lines[start..end].to_vec()
    } else {
        Vec::new()
    };

    let tree_widget = Paragraph::new(Text::from(visible));
    frame.render_widget(tree_widget, chunks[1]);
}

fn render_tree_lines(model: &TuiModel) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    for (i, node) in model.tree.iter().enumerate() {
        let is_cursor = i == model.cursor && model.active_column == Column::Left;
        let is_selected = node.alias_id.as_ref().is_some_and(|id| model.selected.contains(id));

        let indent = "  ".repeat(node.depth as usize);

        match &node.kind {
            NodeKind::GlobalHeader => {
                let marker = if is_cursor { "▸ " } else { "  " };
                lines.push(Line::from(vec![
                    Span::raw(format!("{indent}{marker}")),
                    Span::raw("🌐 "),
                    Span::styled("global", Style::default().bold()),
                ]));
            }
            NodeKind::ProjectHeader => {
                let marker = if is_cursor { "▸ " } else { "  " };
                lines.push(Line::from(vec![
                    Span::raw(format!("{indent}{marker}")),
                    Span::raw("📁 "),
                    Span::styled("project (.aliases)", Style::default().bold()),
                ]));
            }
            NodeKind::ProfileHeader => {
                let icon = if node.is_active { "●" } else { "○" };
                let marker = if is_cursor { "▸ " } else { "  " };
                let active_tag = if node.is_active { " (active)" } else { "" };
                lines.push(Line::from(vec![
                    Span::raw(format!("{indent}{marker}")),
                    Span::styled(
                        format!("{icon} {}{active_tag}", node.label),
                        Style::default().bold(),
                    ),
                ]));
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

                lines.push(Line::from(vec![
                    Span::raw(format!("{indent}  {marker}")),
                    Span::styled(node.label.clone(), name_style),
                ]));

                if let Some(ref cmd) = node.alias_command {
                    lines.push(Line::from(vec![
                        Span::raw(format!("{indent}    ")),
                        Span::styled(cmd.clone(), Style::default().fg(Color::DarkGray)),
                    ]));
                }

                lines.push(Line::from(""));
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
