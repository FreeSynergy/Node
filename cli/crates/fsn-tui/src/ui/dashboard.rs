// Dashboard screen — sidebar + services table.
//
// Layout:
//   ┌──────────────┬──────────────────────────────────────────┐
//   │ header (full width)                                      │
//   ├──────────────┼──────────────────────────────────────────┤
//   │ sidebar      │  main panel (services table)             │
//   ├──────────────┴──────────────────────────────────────────┤
//   │ hint bar (full width)                                    │
//   └─────────────────────────────────────────────────────────┘

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
    Frame,
};

use crate::app::AppState;
use crate::ui::widgets;

pub fn render(f: &mut Frame, state: &AppState) {
    let area = f.area();

    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(area);

    render_header(f, state, outer[0]);
    render_body(f, state, outer[1]);
    render_hint(f, state, outer[2]);
}

fn render_header(f: &mut Frame, state: &AppState, area: Rect) {
    // "FSN · myproject @ example.com"  — project info left, lang button right
    let title = Line::from(vec![
        Span::styled(" FSN ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled("· ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            "FreeSynergy.Node",
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        ),
    ]);

    let header = Paragraph::new(title)
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .alignment(Alignment::Left);
    f.render_widget(header, area);

    // Lang button top-right
    let lang_area = Rect {
        x: area.right().saturating_sub(6),
        y: area.y + 1,
        width: 4,
        height: 1,
    };
    let lang_p = Paragraph::new(Line::from(widgets::lang_button(state)));
    f.render_widget(lang_p, lang_area);
}

fn render_body(f: &mut Frame, state: &AppState, area: Rect) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(18),  // sidebar
            Constraint::Min(1),      // main panel
        ])
        .split(area);

    render_sidebar(f, state, cols[0]);
    render_services(f, state, cols[1]);
}

fn render_sidebar(f: &mut Frame, state: &AppState, area: Rect) {
    let block = Block::default()
        .borders(Borders::RIGHT)
        .border_style(Style::default().fg(Color::DarkGray));
    f.render_widget(block, area);

    let inner = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(1),
    };

    let items = vec![
        Line::from(Span::styled(
            "▶ FreeSynergy.Node",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::raw("")),
        Line::from(Span::styled(
            state.t("sidebar.system"),
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let para = Paragraph::new(items);
    f.render_widget(para, inner);
}

fn render_services(f: &mut Frame, state: &AppState, area: Rect) {
    let block = Block::default()
        .borders(Borders::NONE)
        .title(format!(" {} ", state.t("dash.services")))
        .title_style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD));

    let header = Row::new(vec![
        Cell::from(state.t("dash.col.name"))  .style(Style::default().fg(Color::DarkGray).add_modifier(Modifier::UNDERLINED)),
        Cell::from(state.t("dash.col.type"))  .style(Style::default().fg(Color::DarkGray).add_modifier(Modifier::UNDERLINED)),
        Cell::from(state.t("dash.col.domain")).style(Style::default().fg(Color::DarkGray).add_modifier(Modifier::UNDERLINED)),
        Cell::from(state.t("dash.col.status")).style(Style::default().fg(Color::DarkGray).add_modifier(Modifier::UNDERLINED)),
    ])
    .height(1);

    let rows: Vec<Row> = state
        .services
        .iter()
        .enumerate()
        .map(|(i, svc)| {
            let selected = i == state.selected;
            let name_style = if selected {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            Row::new(vec![
                Cell::from(if selected {
                    format!("▶ {}", svc.name)
                } else {
                    format!("  {}", svc.name)
                })
                .style(name_style),
                Cell::from(svc.service_type.as_str())
                    .style(Style::default().fg(Color::DarkGray)),
                Cell::from(svc.domain.as_str())
                    .style(Style::default().fg(Color::Blue)),
                Cell::from(Line::from(widgets::status_span(svc.status, state))),
            ])
            .height(1)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(20),  // name
            Constraint::Length(10),  // type
            Constraint::Min(25),     // domain
            Constraint::Length(14),  // status
        ],
    )
    .header(header)
    .block(block)
    .row_highlight_style(Style::default().bg(Color::DarkGray));

    let mut table_state = TableState::default().with_selected(Some(state.selected));
    f.render_stateful_widget(table, area, &mut table_state);
}

fn render_hint(f: &mut Frame, state: &AppState, area: Rect) {
    let hint = Paragraph::new(Line::from(Span::styled(
        state.t("dash.hint"),
        Style::default().fg(Color::DarkGray),
    )))
    .alignment(Alignment::Center);
    f.render_widget(hint, area);
}
