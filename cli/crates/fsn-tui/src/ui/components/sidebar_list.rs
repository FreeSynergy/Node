// SidebarList component — project / host / service navigation list.
//
// Also contains impl SidebarItem { sidebar_line, render_center } — the OOP
// rendering methods that each sidebar variant carries with it.
// Design Pattern: Composite — each SidebarItem variant renders its own line.

use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders},
};
use rat_widget::paragraph::{Paragraph, ParagraphState};

use crate::app::{AppState, DashFocus, Lang, SidebarItem};
use crate::ui::{anim::Anim, detail, render_ctx::RenderCtx, widgets};
use super::Component;

pub struct SidebarList;

impl Component for SidebarList {
    fn render(&self, f: &mut RenderCtx<'_>, area: Rect, state: &mut AppState) {
        let focused = state.dash_focus == DashFocus::Sidebar;

        let border_style = if focused {
            Style::new().fg(Color::Cyan)
        } else {
            Style::new().fg(Color::DarkGray)
        };

        f.render_widget(
            Block::default().borders(Borders::RIGHT).border_style(border_style),
            area,
        );

        let inner = Rect {
            x:      area.x + 1,
            y:      area.y,
            width:  area.width.saturating_sub(2),
            height: area.height,
        };

        // Filter input row at top when active
        let (list_area, filter_row) = if let Some(ref query) = state.sidebar_filter {
            let rows = Layout::vertical([Constraint::Length(1), Constraint::Min(1)]).split(inner);
            (rows[1], Some((rows[0], query.as_str().to_owned())))
        } else {
            (inner, None)
        };

        if let Some((farea, query)) = filter_row {
            f.render_stateful_widget(
                Paragraph::new(Line::from(Span::styled(
                    format!("/{}_", query),
                    Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                ))),
                farea,
                &mut ParagraphState::new(),
            );
        }

        // Collect visible indices before mutation (borrow-checker: collect separately so we can
        // write state.sidebar_list_area afterwards).
        let visible_indices: Vec<usize> = if state.sidebar_filter.is_some() {
            state.visible_sidebar_items().into_iter().map(|(i, _)| i).collect()
        } else {
            (0..state.sidebar_items.len()).collect()
        };

        if visible_indices.is_empty() {
            let msg = if state.sidebar_filter.as_deref().is_some_and(|f| !f.is_empty()) {
                state.t("dash.filter.empty")
            } else {
                state.t("dash.no_projects")
            };
            f.render_stateful_widget(
                Paragraph::new(Line::from(Span::styled(msg, Style::new().fg(Color::DarkGray)))),
                list_area,
                &mut ParagraphState::new(),
            );
            return;
        }

        // Store area for mouse hit-testing (mouse.rs::sidebar_hit).
        state.sidebar_list_area = Some(list_area);

        let max_w  = list_area.width.saturating_sub(2) as usize;
        let cursor = state.sidebar_cursor;
        let lang   = state.lang;
        let lines: Vec<Line> = visible_indices
            .iter()
            .map(|&i| {
                let item = &state.sidebar_items[i];
                item.sidebar_line(i == cursor, focused, max_w, lang, &state.anim)
            })
            .collect();
        f.render_stateful_widget(Paragraph::new(lines), list_area, &mut ParagraphState::new());
    }
}

// ── SidebarItem rendering — each item renders itself ─────────────────────────
//
// Design Pattern: Composite — SidebarItem is the component interface.
// Each variant implements sidebar_line() and render_center() for itself.

impl SidebarItem {
    /// Produce the sidebar row line for this item.
    /// `anim` drives the pulsing Running indicator.
    pub(crate) fn sidebar_line(
        &self,
        is_cursor: bool,
        focused: bool,
        max_w: usize,
        lang: Lang,
        anim: &Anim,
    ) -> Line<'static> {
        let t = |key| crate::i18n::t(lang, key);
        match self {
            SidebarItem::Section(key) => Line::from(Span::styled(
                t(key),
                Style::new().fg(Color::DarkGray).add_modifier(Modifier::UNDERLINED),
            )),

            SidebarItem::Project { name, health, .. } => {
                let (prefix, name_style) = if is_cursor {
                    ("▶ ", Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                } else {
                    ("  ", Style::new().fg(Color::White))
                };
                let text = widgets::truncate(prefix, name, max_w.saturating_sub(2));
                Line::from(vec![
                    Span::styled(text, name_style),
                    Span::styled(format!(" {}", health.indicator()), widgets::health_color(*health)),
                ])
            }

            SidebarItem::Host { name, health, .. } => {
                let (prefix, name_style) = if is_cursor {
                    ("  ▶ ", Style::new().fg(Color::Cyan))
                } else {
                    ("  ⊡ ", Style::new().fg(Color::DarkGray))
                };
                let text = widgets::truncate(prefix, name, max_w.saturating_sub(2));
                Line::from(vec![
                    Span::styled(text, name_style),
                    Span::styled(format!(" {}", health.indicator()), widgets::health_color(*health)),
                ])
            }

            SidebarItem::Service { name, status, .. } => {
                let (status_char, status_color) = widgets::run_state_char_anim(*status, anim);
                let (prefix, name_style) = if is_cursor {
                    ("  ▶ ", Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                } else {
                    ("  ◆ ", Style::new().fg(Color::White))
                };
                let text = widgets::truncate(prefix, name, max_w.saturating_sub(2));
                Line::from(vec![
                    Span::styled(text, name_style),
                    Span::styled(format!(" {}", status_char), Style::new().fg(status_color)),
                ])
            }

            SidebarItem::Action { label_key, .. } => {
                let style = if is_cursor {
                    Style::new().fg(Color::Green).add_modifier(Modifier::BOLD)
                } else if focused {
                    Style::new().fg(Color::Green)
                } else {
                    Style::new().fg(Color::DarkGray)
                };
                Line::from(Span::styled(t(label_key), style))
            }
        }
    }

    /// Render the center detail panel for this item's type.
    pub(crate) fn render_center(&self, f: &mut RenderCtx<'_>, state: &mut AppState, area: Rect) {
        match self {
            SidebarItem::Project { slug, .. } => detail::render_project_detail(f, state, area, slug),
            SidebarItem::Host    { slug, .. } => detail::render_host_detail(f, state, area, slug),
            SidebarItem::Service { name, .. } => detail::render_service_detail(f, state, area, name),
            _                                 => crate::ui::components::detail_panel::render_services(f, state, area),
        }
    }
}
