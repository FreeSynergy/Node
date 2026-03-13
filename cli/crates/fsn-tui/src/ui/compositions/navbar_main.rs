// NavBarMain composition — navigation tab bar.
//
// Renders all NavTab variants in order. Active tab is highlighted (Cyan bg).
// Coming-soon tabs are rendered dimmed and are not clickable.
//
// Extracted from ui/components/header_bar.rs to give it its own layout slot
// (menubar_height: 1 in LayoutConfig), keeping the header and nav bar
// independently configurable.

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
};
use rat_widget::paragraph::{Paragraph, ParagraphState};

use crate::app::{AppState, NavTab};
use crate::click_map::ClickTarget;
use crate::ui::render_ctx::RenderCtx;
use super::Composition;

pub struct NavBarMain;

impl Composition for NavBarMain {
    fn render(&self, f: &mut RenderCtx<'_>, area: Rect, state: &mut AppState) {
        let active = state.active_tab;
        let mut spans: Vec<Span> = vec![Span::raw(" ")];
        let mut x = area.x + 1u16; // +1 for the leading space

        for (i, &tab) in NavTab::ALL.iter().enumerate() {
            let label   = state.t(tab.label_key()).to_string();
            let tab_w   = label.chars().count() as u16 + 2; // " label "
            let is_soon = tab.is_coming_soon();

            // Register clickable area — coming-soon tabs still register so
            // mouse.rs can ignore them (ClickTarget carries the NavTab).
            if !is_soon {
                state.click_map.push(
                    Rect { x, y: area.y, width: tab_w, height: 1 },
                    ClickTarget::NavTab { index: i },
                );
            }
            x += tab_w;

            let span = if tab == active {
                Span::styled(
                    format!(" {} ", label),
                    Style::new().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD),
                )
            } else if is_soon {
                Span::styled(format!(" {} ", label), Style::new().fg(Color::DarkGray).add_modifier(Modifier::DIM))
            } else {
                Span::styled(format!(" {} ", label), Style::new().fg(Color::DarkGray))
            };
            spans.push(span);

            if i < NavTab::ALL.len() - 1 {
                spans.push(Span::styled(" │ ", Style::new().fg(Color::DarkGray)));
                x += 3;
            }
        }

        f.render_stateful_widget(
            Paragraph::new(Line::from(spans)),
            area,
            &mut ParagraphState::new(),
        );
    }
}
