// Welcome screen — shown when no project exists.
//
// Layout:
//   ┌─────────────────────────────────────────────────────────┐
//   │  FreeSynergy.Node v0.1.0                      [DE]      │  ← header
//   ├─────────────────────────────────────────────────────────┤
//   │                                                         │
//   │     Willkommen bei FreeSynergy.Node                     │  ← title
//   │     Dezentrale Infrastruktur …                          │  ← subtitle
//   │                                                         │
//   │     Host : server   Podman: 5.2.1                       │  ← sysinfo
//   │     User : kal      Uptime: 3d 12h                      │
//   │     IP   : 1.2.3.4  Arch  : x86_64                      │
//   │     RAM  : 4.2/16GB CPU   : 8 Kerne                     │
//   │                                                         │
//   │       [ Neues Projekt ]    [ Vorhandenes Projekt ]       │  ← buttons
//   │                                                         │
//   ├─────────────────────────────────────────────────────────┤
//   │  Tab=Sprache  Enter=Auswahl  q=Beenden                   │  ← hint bar
//   └─────────────────────────────────────────────────────────┘

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::AppState;
use crate::ui::widgets;

pub fn render(f: &mut Frame, state: &AppState) {
    let area = f.area();

    // Outer layout: header / body / hint
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // header
            Constraint::Min(1),     // body
            Constraint::Length(1),  // hint bar
        ])
        .split(area);

    render_header(f, state, outer[0]);
    render_body(f, state, outer[1]);
    render_hint(f, state, outer[2]);
}

fn render_header(f: &mut Frame, state: &AppState, area: Rect) {
    let lang_btn = widgets::lang_button(state);

    let title = Line::from(vec![
        Span::styled(
            " FreeSynergy.Node ",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "v0.1.0",
            Style::default().fg(Color::DarkGray),
        ),
    ]);

    // Header block with lang button on the right
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
    let lang_p = Paragraph::new(Line::from(lang_btn));
    f.render_widget(lang_p, lang_area);
}

fn render_body(f: &mut Frame, state: &AppState, area: Rect) {
    // Body columns: padding / content / padding
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(15),
            Constraint::Percentage(70),
            Constraint::Percentage(15),
        ])
        .split(area);

    let inner = cols[1];

    // Vertical sections in the content column
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),   // spacer
            Constraint::Length(2),   // title + subtitle
            Constraint::Length(1),   // spacer
            Constraint::Length(4),   // sysinfo grid
            Constraint::Length(2),   // spacer
            Constraint::Length(3),   // buttons
            Constraint::Min(1),      // rest
        ])
        .split(inner);

    render_title(f, state, rows[1]);
    render_sysinfo(f, state, rows[3]);
    render_buttons(f, state, rows[5]);
}

fn render_title(f: &mut Frame, state: &AppState, area: Rect) {
    let text = Text::from(vec![
        Line::from(Span::styled(
            state.t("welcome.title"),
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            state.t("welcome.subtitle"),
            Style::default().fg(Color::DarkGray),
        )),
    ]);
    let p = Paragraph::new(text).alignment(Alignment::Center);
    f.render_widget(p, area);
}

fn render_sysinfo(f: &mut Frame, state: &AppState, area: Rect) {
    let s = &state.sysinfo;
    let label = Style::default().fg(Color::DarkGray);
    let value = Style::default().fg(Color::White);

    let lines = vec![
        info_line(state.t("sys.host"),   &s.hostname,        state.t("sys.podman"), &s.podman_version, label, value),
        info_line(state.t("sys.user"),   &s.user,            state.t("sys.uptime"), &s.uptime_str,     label, value),
        info_line(state.t("sys.ip"),     &s.ip,              state.t("sys.arch"),   &s.arch,           label, value),
        info_line(state.t("sys.ram"),    &s.ram_str(),       state.t("sys.cpu"),    &format!("{}", s.cpu_cores), label, value),
    ];

    let p = Paragraph::new(Text::from(lines)).alignment(Alignment::Center);
    f.render_widget(p, area);
}

fn info_line(
    l1: &str, v1: &str,
    l2: &str, v2: &str,
    label: Style, value: Style,
) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{:<12}", l1), label),
        Span::styled(format!("{:<18}", v1), value),
        Span::styled(format!("{:<12}", l2), label),
        Span::styled(v2.to_string(), value),
    ])
}

fn render_buttons(f: &mut Frame, state: &AppState, area: Rect) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(28),
            Constraint::Percentage(4),
            Constraint::Percentage(28),
            Constraint::Percentage(20),
        ])
        .split(area);

    // Button 1 — New Project (always active)
    let btn1_focused = state.welcome_focus == 0;
    let btn1 = Paragraph::new(widgets::button_line(state.t("welcome.new_project"), btn1_focused, false))
        .block(Block::default().borders(Borders::ALL).border_style(
            if btn1_focused {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::DarkGray)
            },
        ))
        .alignment(Alignment::Center);
    f.render_widget(btn1, cols[1]);

    // Button 2 — Open Project (grayed out)
    let btn2_focused = state.welcome_focus == 1;
    let btn2_label = format!("{} {}", state.t("welcome.open_project"), state.t("welcome.open_disabled"));
    let btn2 = Paragraph::new(widgets::button_line(&btn2_label, btn2_focused, true))
        .block(Block::default().borders(Borders::ALL).border_style(
            Style::default().fg(Color::DarkGray),
        ))
        .alignment(Alignment::Center);
    f.render_widget(btn2, cols[3]);
}

fn render_hint(f: &mut Frame, state: &AppState, area: Rect) {
    let hint = Paragraph::new(Line::from(Span::styled(
        state.t("welcome.hint"),
        Style::default().fg(Color::DarkGray),
    )))
    .alignment(Alignment::Center);
    f.render_widget(hint, area);
}
