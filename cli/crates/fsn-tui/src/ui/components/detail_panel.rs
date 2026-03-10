// DetailPanel component — stats cards (top) + center content (rest).
//
// The top 3 rows show system stats cards (RAM, system, running, alerts).
// The remaining area delegates to the selected SidebarItem or the services table.
//
// render_services() is pub(crate) so SidebarItem::render_center (in sidebar_list.rs)
// can call it for Section / Action items that have no detail view of their own.

use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Styled},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders},
};
use rat_widget::paragraph::{Paragraph, ParagraphState};
use rat_widget::table::{Table, TableState};
use rat_widget::table::textdata::{Cell, Row};

use crate::app::{AppState, DashFocus, RunState};
use crate::ui::{render_ctx::RenderCtx, widgets};
use super::Component;

pub struct DetailPanel;

impl Component for DetailPanel {
    fn render(&self, f: &mut RenderCtx<'_>, area: Rect, state: &mut AppState) {
        let rows = Layout::vertical([
            Constraint::Length(3), // stats cards
            Constraint::Min(1),    // content
        ])
        .split(area);

        render_stats_cards(f, state, rows[0]);
        render_center(f, state, rows[1]);
    }
}

// ── Stats cards ───────────────────────────────────────────────────────────────

fn render_stats_cards(f: &mut RenderCtx<'_>, state: &AppState, area: Rect) {
    let cards = Layout::horizontal([
        Constraint::Percentage(25),
        Constraint::Percentage(25),
        Constraint::Percentage(25),
        Constraint::Percentage(25),
    ])
    .split(area);

    render_stat_card(f, cards[0], "RAM", &state.sysinfo.ram_str(), Color::Cyan);

    let sys_label = format!("{}@{}", state.sysinfo.user, state.sysinfo.hostname);
    render_stat_card(f, cards[1], "System", &sys_label, Color::White);

    let total = state.services.len();
    let ok    = state.services.iter().filter(|s| s.status == RunState::Running).count();
    let running_color = if total == 0 {
        Color::DarkGray
    } else if ok == total {
        Color::Green
    } else {
        Color::Yellow
    };
    render_stat_card(f, cards[2], "Running", &format!("{} / {}", ok, total), running_color);

    let failed  = state.services.iter().filter(|s| s.status == RunState::Failed).count();
    let stopped = state.services.iter().filter(|s| s.status == RunState::Stopped).count();
    let alert_color = if failed > 0 { Color::Red } else if stopped > 0 { Color::Yellow } else { Color::Green };
    render_stat_card(f, cards[3], "Alerts", &format!("⚠ {}  ✗ {}", stopped, failed), alert_color);
}

fn render_stat_card(f: &mut RenderCtx<'_>, area: Rect, label: &str, value: &str, color: Color) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(Color::DarkGray))
        .title(Span::styled(format!(" {} ", label), Style::new().fg(Color::DarkGray)));

    let inner = block.inner(area);
    f.render_widget(block, area);
    f.render_stateful_widget(
        Paragraph::new(Line::from(Span::styled(
            value.to_string(),
            Style::new().fg(color).add_modifier(Modifier::BOLD),
        )))
        .alignment(Alignment::Center),
        inner,
        &mut ParagraphState::new(),
    );
}

// ── Center panel ──────────────────────────────────────────────────────────────

fn render_center(f: &mut RenderCtx<'_>, state: &mut AppState, area: Rect) {
    match state.current_sidebar_item().cloned() {
        Some(item) => item.render_center(f, state, area),
        None       => render_services(f, state, area),
    }
}

// ── Services table ────────────────────────────────────────────────────────────

pub(crate) fn render_services(f: &mut RenderCtx<'_>, state: &mut AppState, area: Rect) {
    let services_focused = state.dash_focus == DashFocus::Services;

    let block = Block::default()
        .borders(Borders::NONE)
        .title(Span::styled(
            format!(" {} ", state.t("dash.services")),
            Style::new()
                .fg(if services_focused { Color::Cyan } else { Color::White })
                .add_modifier(Modifier::BOLD),
        ));

    if state.services.is_empty() {
        f.render_stateful_widget(
            Paragraph::new(Line::from(Span::styled(
                state.t("dash.no_services"),
                Style::new().fg(Color::DarkGray),
            )))
            .block(block),
            area,
            &mut ParagraphState::new(),
        );
        return;
    }

    let header = Row::new(vec![
        Cell::from(state.t("dash.col.name"))
            .set_style(Style::new().fg(Color::DarkGray).add_modifier(Modifier::UNDERLINED)),
        Cell::from(state.t("dash.col.type"))
            .set_style(Style::new().fg(Color::DarkGray).add_modifier(Modifier::UNDERLINED)),
        Cell::from(state.t("dash.col.domain"))
            .set_style(Style::new().fg(Color::DarkGray).add_modifier(Modifier::UNDERLINED)),
        Cell::from(state.t("dash.col.status"))
            .set_style(Style::new().fg(Color::DarkGray).add_modifier(Modifier::UNDERLINED)),
    ])
    .height(1);

    let multi_select = !state.selected_services.is_empty();

    let rows: Vec<Row> = state
        .services
        .iter()
        .enumerate()
        .map(|(i, svc)| {
            let is_cursor  = i == state.selected && services_focused;
            let is_checked = state.selected_services.contains(&i);

            let name_style = if is_cursor {
                Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else if is_checked {
                Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::new().fg(Color::White)
            };

            let prefix = if multi_select {
                if is_checked { "[✓] " } else { "[ ] " }
            } else if is_cursor {
                "▶ "
            } else {
                "  "
            };

            Row::new(vec![
                Cell::from(format!("{}{}", prefix, svc.name)).set_style(name_style),
                Cell::from(svc.service_type.as_str()).set_style(Style::new().fg(Color::DarkGray)),
                Cell::from(svc.domain.as_str()).set_style(Style::new().fg(Color::Blue)),
                Cell::from(Line::from(widgets::status_span(svc.status, state))),
            ])
            .height(1)
        })
        .collect();

    let table = Table::new_ratatui(
        rows,
        [
            Constraint::Length(20),
            Constraint::Length(10),
            Constraint::Min(25),
            Constraint::Length(14),
        ],
    )
    .header(header)
    .block(block)
    .select_row_style(Some(Style::new().bg(Color::DarkGray)));

    // Store area for mouse hit-testing (mouse.rs::services_hit).
    state.services_table_area = Some(area);

    let mut table_state = TableState::default();
    if services_focused {
        table_state.select(Some(state.selected));
    }
    f.render_stateful_widget(table, area, &mut table_state);
}
