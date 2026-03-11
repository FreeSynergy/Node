// HeaderBar component — logo row + navigation tab bar.
//
// Renders the top 5 rows of the application:
//   Row 0-3: BigText "FSN" logo (left) + title / subtitle / project info (right)
//   Row 4:   Navigation tab bar

use tui_big_text::{BigText, PixelSize};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
};
use rat_widget::paragraph::{Paragraph, ParagraphState};

use crate::app::{AppState, Screen, SidebarAction, SidebarItem};
use crate::click_map::ClickTarget;
use crate::ui::{render_ctx::RenderCtx, widgets};
use super::Component;

const TAB_KEYS: &[&str] = &[
    "dash.tab.projects",
    "dash.tab.hosts",
    "dash.tab.services",
    "dash.tab.store",
    "dash.tab.settings",
];

pub struct HeaderBar;

impl Component for HeaderBar {
    fn render(&self, f: &mut RenderCtx<'_>, area: Rect, state: &mut AppState) {
        let rows = Layout::vertical([
            Constraint::Length(4), // logo row (BigText Quadrant = 4 rows)
            Constraint::Length(1), // tab bar
        ])
        .split(area);

        render_logo_row(f, state, rows[0]);
        render_tab_bar(f, state, rows[1]);
    }
}

// ClickMap is cleared and rebuilt every frame by the top-level render entry
// (ui/mod.rs). Both render_logo_row and render_tab_bar push their own
// clickable regions so mouse.rs can dispatch without screen-specific branches.

fn render_logo_row(f: &mut RenderCtx<'_>, state: &mut AppState, area: Rect) {
    let cols = Layout::horizontal([
        Constraint::Length(18), // "FSN" in Quadrant: 3 chars × 4 cols + padding
        Constraint::Min(1),
    ])
    .split(area);

    // BigText logo
    let big = BigText::builder()
        .pixel_size(PixelSize::Quadrant)
        .style(Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .lines(vec![Line::from("FSN")])
        .build();
    f.render_widget(big, cols[0]);

    // Right info column (4 rows)
    let info_rows = Layout::vertical([
        Constraint::Length(1), // title + lang
        Constraint::Length(1), // subtitle + version
        Constraint::Length(1), // project/domain
        Constraint::Length(1), // separator
    ])
    .split(cols[1]);

    // Row 0: "FreeSynergy.Node" (left) + [DE] button (right)
    f.render_stateful_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("FreeSynergy", Style::new().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(".Node",       Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ])),
        info_rows[0],
        &mut ParagraphState::new(),
    );
    let lang_area = Rect {
        x:      cols[1].right().saturating_sub(5),
        y:      info_rows[0].y,
        width:  5,
        height: 1,
    };
    // Register lang button as clickable — mouse.rs handles LangToggle globally.
    state.click_map.push(lang_area, ClickTarget::LangToggle);
    f.render_stateful_widget(
        Paragraph::new(Line::from(widgets::lang_button(state))),
        lang_area,
        &mut ParagraphState::new(),
    );

    // Row 1: subtitle (left) + version (right)
    f.render_stateful_widget(
        Paragraph::new(Line::from(Span::styled(
            "Modular Deployment System  —  by KalEl",
            Style::new().fg(Color::DarkGray),
        ))),
        info_rows[1],
        &mut ParagraphState::new(),
    );
    let ver_str  = format!("v{}  ", env!("CARGO_PKG_VERSION"));
    let ver_w    = ver_str.chars().count() as u16;
    let ver_area = Rect {
        x:      cols[1].right().saturating_sub(ver_w),
        y:      info_rows[1].y,
        width:  ver_w,
        height: 1,
    };
    f.render_stateful_widget(
        Paragraph::new(Line::from(Span::styled(ver_str, Style::new().fg(Color::DarkGray)))),
        ver_area,
        &mut ParagraphState::new(),
    );

    // Row 2: project name @ domain
    let domain_text = state
        .projects
        .get(state.selected_project)
        .map(|p| format!("{}  @  {}", p.name(), p.domain()))
        .unwrap_or_else(|| state.t("dash.no_project_selected").to_string());
    f.render_stateful_widget(
        Paragraph::new(Line::from(Span::styled(domain_text, Style::new().fg(Color::DarkGray)))),
        info_rows[2],
        &mut ParagraphState::new(),
    );

    // Row 3: separator line
    use ratatui::widgets::{Block, Borders};
    f.render_widget(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::new().fg(Color::DarkGray)),
        info_rows[3],
    );
}

fn render_tab_bar(f: &mut RenderCtx<'_>, state: &mut AppState, area: Rect) {
    let active     = active_tab_index(state);
    let mut spans: Vec<Span> = vec![Span::raw(" ")];
    // Track x position for click target registration.
    let mut x = area.x + 1u16; // +1 for the leading space

    for (i, &key) in TAB_KEYS.iter().enumerate() {
        // .to_string() releases the &self borrow before click_map is mutably borrowed.
        let label  = state.t(key).to_string();
        let tab_w  = label.chars().count() as u16 + 2; // " label "

        // Register every tab as clickable — mouse.rs dispatches NavTab to navigate_to_tab().
        state.click_map.push(
            Rect { x, y: area.y, width: tab_w, height: 1 },
            ClickTarget::NavTab { index: i },
        );
        x += tab_w;

        if i == active {
            spans.push(Span::styled(
                format!(" {} ", label),
                Style::new().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD),
            ));
        } else {
            spans.push(Span::styled(format!(" {} ", label), Style::new().fg(Color::DarkGray)));
        }
        if i < TAB_KEYS.len() - 1 {
            spans.push(Span::styled(" │ ", Style::new().fg(Color::DarkGray)));
            x += 3; // " │ "
        }
    }

    f.render_stateful_widget(Paragraph::new(Line::from(spans)), area, &mut ParagraphState::new());
}

fn active_tab_index(state: &AppState) -> usize {
    // Settings screen → always highlight the Settings tab (index 4).
    if state.screen == Screen::Settings {
        return 4;
    }
    match state.current_sidebar_item() {
        Some(SidebarItem::Project { .. }) => 0,
        Some(SidebarItem::Host    { .. }) => 1,
        Some(SidebarItem::Service { .. }) => 2,
        Some(SidebarItem::Action { kind, .. }) => match kind {
            SidebarAction::NewProject => 0,
            SidebarAction::NewHost    => 1,
            SidebarAction::NewService => 2,
        },
        _ => 0,
    }
}
