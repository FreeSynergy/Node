// Dashboard screen — sidebar (projects + hosts + services) + context center panel.
//
// ┌──────────────────────────────────────────────────────────────────┐
// │  FSN · myproject @ example.com                          [DE]    │
// ├────────────────────┬─────────────────────────────────────────────┤
// │ PROJEKTE           │  Services                                   │
// │ ▶ myproject        │  Name      Typ    Domain    Status          │
// │   testprojekt      │▶ kanidm    iam    auth.ex   ● Aktiv        │
// │ + Neues Projekt    │  forgejo   git    git.ex    ○ Stopp        │
// │ HOSTS              │                                             │
// │   ⊡ srv1           │  (center shows details of selected item)   │
// │ + Neuer Host       │                                             │
// │ SERVICES           │                                             │
// │   ◆ kanidm         │                                             │
// │   ◆ forgejo        │                                             │
// │ + Neuer Service    │                                             │
// ├────────────────────┴─────────────────────────────────────────────┤
// │  ↑↓=Nav  n=Neu  e=Bearbeiten  x=Löschen  Tab=Detail  q=Quit     │
// └──────────────────────────────────────────────────────────────────┘

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
    Frame,
};

use crate::app::{AppState, DashFocus, Lang, RunState, SidebarItem};
use crate::ui::widgets;

pub fn render(f: &mut Frame, state: &mut AppState, area: ratatui::layout::Rect) {

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

// ── Header ────────────────────────────────────────────────────────────────────

fn render_header(f: &mut Frame, state: &AppState, area: Rect) {
    let (name, domain) = state.projects.get(state.selected_project)
        .map(|p| (p.name(), p.domain()))
        .unwrap_or(("FreeSynergy.Node", ""));

    let title = if domain.is_empty() {
        Line::from(vec![
            Span::styled(" FSN ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled("· ", Style::default().fg(Color::DarkGray)),
            Span::styled(name.to_string(), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        ])
    } else {
        Line::from(vec![
            Span::styled(" FSN ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled("· ", Style::default().fg(Color::DarkGray)),
            Span::styled(name.to_string(), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(" @ ", Style::default().fg(Color::DarkGray)),
            Span::styled(domain.to_string(), Style::default().fg(Color::DarkGray)),
        ])
    };

    let header = Paragraph::new(title)
        .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(Color::DarkGray)))
        .alignment(Alignment::Left);
    f.render_widget(header, area);

    let build_str = format!("v{} {} ({})  ", env!("CARGO_PKG_VERSION"), crate::BUILD_TIME, crate::GIT_HASH);
    let build_w   = build_str.chars().count() as u16;
    let build_x   = area.right().saturating_sub(build_w + 5);
    let build_area = Rect { x: build_x, y: area.y + 1, width: build_w, height: 1 };
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(build_str, Style::default().fg(Color::DarkGray)))),
        build_area,
    );

    let lang_area = Rect { x: area.right().saturating_sub(6), y: area.y + 1, width: 4, height: 1 };
    f.render_widget(Paragraph::new(Line::from(widgets::lang_button(state))), lang_area);
}

// ── Body ──────────────────────────────────────────────────────────────────────

fn render_body(f: &mut Frame, state: &AppState, area: Rect) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(28),
            Constraint::Min(1),
        ])
        .split(area);

    render_sidebar(f, state, cols[0]);
    render_center(f, state, cols[1]);
}

// ── Sidebar ───────────────────────────────────────────────────────────────────

fn render_sidebar(f: &mut Frame, state: &AppState, area: Rect) {
    let focused = state.dash_focus == DashFocus::Sidebar;

    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    f.render_widget(
        Block::default().borders(Borders::RIGHT).border_style(border_style),
        area,
    );

    let inner = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    };

    if state.sidebar_items.is_empty() {
        f.render_widget(
            Paragraph::new(Line::from(Span::styled(
                state.t("dash.no_projects"),
                Style::default().fg(Color::DarkGray),
            ))),
            inner,
        );
        return;
    }

    let max_w = inner.width.saturating_sub(4) as usize;

    // Each SidebarItem renders its own sidebar line — no external dispatch.
    let lines: Vec<Line> = state.sidebar_items.iter().enumerate()
        .map(|(i, item)| item.sidebar_line(i == state.sidebar_cursor, focused, max_w, state.lang))
        .collect();

    f.render_widget(Paragraph::new(lines), inner);
}

// ── Center panel ──────────────────────────────────────────────────────────────

/// Dispatches to the center panel appropriate for the currently focused sidebar item.
/// Each SidebarItem knows how to render its own center view.
fn render_center(f: &mut Frame, state: &AppState, area: Rect) {
    match state.current_sidebar_item() {
        Some(item) => item.render_center(f, state, area),
        None       => render_services(f, state, area),
    }
}

// ── Host detail panel ─────────────────────────────────────────────────────────

fn render_host_detail(f: &mut Frame, state: &AppState, area: Rect, slug: &str) {
    let Some(host) = state.hosts.iter().find(|h| h.slug == slug) else {
        f.render_widget(Paragraph::new("—"), area);
        return;
    };

    let display = host.config.host.alias.as_deref().unwrap_or(&host.config.host.name);
    let block = Block::default()
        .borders(Borders::NONE)
        .title(Span::styled(
            format!(" {} ", display),
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let addr     = host.config.host.addr();
    let ssh_user = &host.config.host.ssh_user;
    let ssh_port = host.config.host.ssh_port;
    let external = if host.config.host.external { "extern" } else { "lokal" };
    let alias    = host.config.host.alias.as_deref().unwrap_or("—");

    let lines = vec![
        Line::from(vec![
            Span::styled("Adresse:   ", Style::default().fg(Color::DarkGray)),
            Span::styled(addr.to_string(), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("SSH:       ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{}@{}:{}", ssh_user, addr, ssh_port), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("Alias:     ", Style::default().fg(Color::DarkGray)),
            Span::styled(alias.to_string(), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("Typ:       ", Style::default().fg(Color::DarkGray)),
            Span::styled(external.to_string(), Style::default().fg(Color::White)),
        ]),
    ];
    f.render_widget(Paragraph::new(lines), inner);
}

// ── Service detail panel ──────────────────────────────────────────────────────

fn render_service_detail(f: &mut Frame, state: &AppState, area: Rect, svc_name: &str) {
    let Some(proj) = state.projects.get(state.selected_project) else {
        f.render_widget(Paragraph::new("—"), area);
        return;
    };
    let Some(entry) = proj.config.load.services.get(svc_name) else {
        f.render_widget(Paragraph::new("—"), area);
        return;
    };

    let status = state.last_podman_statuses.get(svc_name).copied().unwrap_or(RunState::Missing);
    let status_label = match status {
        RunState::Running => "● Running",
        RunState::Stopped => "○ Stopped",
        RunState::Failed  => "✕ Failed",
        RunState::Missing => "· Missing",
    };
    let status_color = match status {
        RunState::Running => Color::Green,
        RunState::Stopped => Color::DarkGray,
        RunState::Failed  => Color::Red,
        RunState::Missing => Color::DarkGray,
    };

    let block = Block::default()
        .borders(Borders::NONE)
        .title(Span::styled(
            format!(" {} ", svc_name),
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let subdomain = entry.subdomain.as_deref().unwrap_or("—");
    let port      = entry.port.map(|p| p.to_string()).unwrap_or_else(|| "—".to_string());
    let env_count = entry.env.len();
    let domain    = format!("{}.{}", svc_name, proj.domain());

    let lines = vec![
        Line::from(vec![
            Span::styled("Klasse:    ", Style::default().fg(Color::DarkGray)),
            Span::styled(entry.service_class.clone(), Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("Projekt:   ", Style::default().fg(Color::DarkGray)),
            Span::styled(proj.slug.clone(), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("Domain:    ", Style::default().fg(Color::DarkGray)),
            Span::styled(domain, Style::default().fg(Color::Blue)),
        ]),
        Line::from(vec![
            Span::styled("Subdomain: ", Style::default().fg(Color::DarkGray)),
            Span::styled(subdomain.to_string(), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("Port:      ", Style::default().fg(Color::DarkGray)),
            Span::styled(port, Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("Status:    ", Style::default().fg(Color::DarkGray)),
            Span::styled(status_label.to_string(), Style::default().fg(status_color)),
        ]),
        Line::from(vec![
            Span::styled("Env-Vars:  ", Style::default().fg(Color::DarkGray)),
            Span::styled(env_count.to_string(), Style::default().fg(Color::White)),
        ]),
    ];
    f.render_widget(Paragraph::new(lines), inner);
}

// ── Services table ────────────────────────────────────────────────────────────

fn render_services(f: &mut Frame, state: &AppState, area: Rect) {
    let services_focused = state.dash_focus == DashFocus::Services;

    let block = Block::default()
        .borders(Borders::NONE)
        .title(Span::styled(
            format!(" {} ", state.t("dash.services")),
            Style::default()
                .fg(if services_focused { Color::Cyan } else { Color::White })
                .add_modifier(Modifier::BOLD),
        ));

    if state.services.is_empty() {
        let msg = Paragraph::new(Line::from(Span::styled(
            "(keine Services)",
            Style::default().fg(Color::DarkGray),
        )))
        .block(block);
        f.render_widget(msg, area);
        return;
    }

    let header = Row::new(vec![
        Cell::from(state.t("dash.col.name"))  .style(Style::default().fg(Color::DarkGray).add_modifier(Modifier::UNDERLINED)),
        Cell::from(state.t("dash.col.type"))  .style(Style::default().fg(Color::DarkGray).add_modifier(Modifier::UNDERLINED)),
        Cell::from(state.t("dash.col.domain")).style(Style::default().fg(Color::DarkGray).add_modifier(Modifier::UNDERLINED)),
        Cell::from(state.t("dash.col.status")).style(Style::default().fg(Color::DarkGray).add_modifier(Modifier::UNDERLINED)),
    ])
    .height(1);

    let rows: Vec<Row> = state.services.iter().enumerate().map(|(i, svc)| {
        let selected = i == state.selected && services_focused;
        let name_style = if selected {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        Row::new(vec![
            Cell::from(if selected { format!("▶ {}", svc.name) } else { format!("  {}", svc.name) })
                .style(name_style),
            Cell::from(svc.service_type.as_str()).style(Style::default().fg(Color::DarkGray)),
            Cell::from(svc.domain.as_str()).style(Style::default().fg(Color::Blue)),
            Cell::from(Line::from(widgets::status_span(svc.status, state))),
        ])
        .height(1)
    }).collect();

    let table = Table::new(rows, [
        Constraint::Length(20),
        Constraint::Length(10),
        Constraint::Min(25),
        Constraint::Length(14),
    ])
    .header(header)
    .block(block)
    .row_highlight_style(Style::default().bg(Color::DarkGray));

    let mut table_state = TableState::default().with_selected(
        if services_focused { Some(state.selected) } else { None }
    );
    f.render_stateful_widget(table, area, &mut table_state);
}

// ── Hint bar ──────────────────────────────────────────────────────────────────

fn render_hint(f: &mut Frame, state: &AppState, area: Rect) {
    let has_confirm = state.confirm_overlay().is_some();

    let key: &'static str = if has_confirm {
        "dash.hint.confirm"
    } else {
        match state.dash_focus {
            DashFocus::Services => "dash.hint.services",
            // Each SidebarItem knows its own hint key — no external dispatch needed.
            DashFocus::Sidebar  => state.current_sidebar_item()
                .map(|item| item.hint_key())
                .unwrap_or("dash.hint"),
        }
    };

    let style = if has_confirm {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    f.render_widget(
        Paragraph::new(Line::from(Span::styled(state.t(key), style)))
            .alignment(Alignment::Center),
        area,
    );
}

// ── SidebarItem rendering — each item renders itself ─────────────────────────

fn truncate(prefix: &str, name: &str, max_w: usize) -> String {
    let total = prefix.len() + name.len();
    if total > max_w && max_w > prefix.len() + 1 {
        format!("{}{}…", prefix, &name[..max_w - prefix.len() - 1])
    } else {
        format!("{}{}", prefix, name)
    }
}

impl SidebarItem {
    /// Produce the sidebar row line for this item.
    ///
    /// Analogous to an element rendering its own `<li>` — the caller just
    /// collects lines; no variant-specific logic leaks into the sidebar renderer.
    fn sidebar_line(&self, is_cursor: bool, focused: bool, max_w: usize, lang: Lang) -> Line<'static> {
        let t = |key| crate::i18n::t(lang, key);
        match self {
            SidebarItem::Section(key) => Line::from(Span::styled(
                t(key),
                Style::default().fg(Color::DarkGray).add_modifier(Modifier::UNDERLINED),
            )),

            SidebarItem::Project { name, .. } => {
                let (prefix, style) = if is_cursor {
                    ("▶ ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                } else {
                    ("  ", Style::default().fg(Color::White))
                };
                Line::from(Span::styled(truncate(prefix, name, max_w), style))
            }

            SidebarItem::Host { name, .. } => {
                let (prefix, style) = if is_cursor {
                    ("  ▶ ", Style::default().fg(Color::Cyan))
                } else {
                    ("  ⊡ ", Style::default().fg(Color::DarkGray))
                };
                Line::from(Span::styled(truncate(prefix, name, max_w), style))
            }

            SidebarItem::Service { name, status, .. } => {
                let (status_char, status_color) = match status {
                    RunState::Running => ("●", Color::Green),
                    RunState::Stopped => ("○", Color::DarkGray),
                    RunState::Failed  => ("✕", Color::Red),
                    RunState::Missing => ("·", Color::DarkGray),
                };
                let (prefix, name_style) = if is_cursor {
                    ("  ▶ ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                } else {
                    ("  ◆ ", Style::default().fg(Color::White))
                };
                let text = truncate(prefix, name, max_w.saturating_sub(2));
                Line::from(vec![
                    Span::styled(text, name_style),
                    Span::styled(format!(" {}", status_char), Style::default().fg(status_color)),
                ])
            }

            SidebarItem::Action { label_key, .. } => {
                let style = if is_cursor {
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                } else if focused {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                Line::from(Span::styled(t(label_key), style))
            }
        }
    }

    /// Render the center detail panel appropriate for this item's type.
    ///
    /// Analogous to a component rendering its own detail view — the caller
    /// only knows "show the center panel for the selected item".
    fn render_center(&self, f: &mut Frame, state: &AppState, area: Rect) {
        match self {
            SidebarItem::Project { slug, .. } => render_project_detail(f, state, area, slug),
            SidebarItem::Host    { slug, .. } => render_host_detail(f, state, area, slug),
            SidebarItem::Service { name, .. } => render_service_detail(f, state, area, name),
            // Action, Section → show service table
            _                                 => render_services(f, state, area),
        }
    }
}

// ── Project detail panel ──────────────────────────────────────────────────────

fn render_project_detail(f: &mut Frame, state: &AppState, area: Rect, slug: &str) {
    let Some(proj) = state.projects.iter().find(|p| p.slug == slug) else {
        f.render_widget(Paragraph::new("—"), area);
        return;
    };

    let name       = proj.config.project.name.as_str();
    let domain     = proj.config.project.domain.as_str();
    let email      = proj.email();
    let install    = proj.install_dir();
    let svc_count  = proj.config.load.services.len();
    let host_count = state.hosts.len();
    let langs      = proj.config.project.languages.join(", ");

    let svc_ok  = state.services.iter().filter(|s| s.status == RunState::Running).count();
    let svc_err = state.services.iter().filter(|s| s.status == RunState::Failed).count();

    let block = Block::default()
        .borders(Borders::NONE)
        .title(Span::styled(
            format!(" {} ", name),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let mut lines = vec![
        Line::from(vec![
            Span::styled("Domain:    ", Style::default().fg(Color::DarkGray)),
            Span::styled(domain.to_string(), Style::default().fg(Color::Blue)),
        ]),
    ];
    if !email.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("E-Mail:    ", Style::default().fg(Color::DarkGray)),
            Span::styled(email.to_string(), Style::default().fg(Color::White)),
        ]));
    }
    if !langs.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("Sprachen:  ", Style::default().fg(Color::DarkGray)),
            Span::styled(langs, Style::default().fg(Color::White)),
        ]));
    }
    if !install.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("Install:   ", Style::default().fg(Color::DarkGray)),
            Span::styled(install.to_string(), Style::default().fg(Color::White)),
        ]));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("Services:  ", Style::default().fg(Color::DarkGray)),
        Span::styled(svc_count.to_string(), Style::default().fg(Color::White)),
        Span::styled("  (", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("● {}", svc_ok), Style::default().fg(Color::Green)),
        if svc_err > 0 {
            Span::styled(format!("  ✕ {}", svc_err), Style::default().fg(Color::Red))
        } else {
            Span::styled("", Style::default())
        },
        Span::styled(")", Style::default().fg(Color::DarkGray)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Hosts:     ", Style::default().fg(Color::DarkGray)),
        Span::styled(host_count.to_string(), Style::default().fg(Color::White)),
    ]));

    if let Some(desc) = proj.config.project.description.as_deref() {
        if !desc.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(desc.to_string(), Style::default().fg(Color::DarkGray))));
        }
    }

    f.render_widget(Paragraph::new(lines), inner);
}
