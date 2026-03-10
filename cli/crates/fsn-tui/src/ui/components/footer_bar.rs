// FooterBar component — license label (left) + context-sensitive hints (right).

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
};
use rat_widget::paragraph::{Paragraph, ParagraphState};

use crate::app::{AppState, DashFocus};
use crate::ui::render_ctx::RenderCtx;
use super::Component;

pub struct FooterBar;

impl Component for FooterBar {
    fn render(&self, f: &mut RenderCtx<'_>, area: Rect, state: &mut AppState) {
        // Left: MIT license label
        let mit   = "  MIT © FreeSynergy.Net";
        let mit_w = mit.chars().count() as u16;
        let mit_area = Rect {
            x:      area.x,
            y:      area.y,
            width:  mit_w.min(area.width / 2),
            height: 1,
        };
        f.render_stateful_widget(
            Paragraph::new(Line::from(Span::styled(mit, Style::new().fg(Color::DarkGray)))),
            mit_area,
            &mut ParagraphState::new(),
        );

        // Right: context-sensitive hints
        let has_confirm = state.confirm_overlay().is_some();

        let hint_text: String = if has_confirm {
            state.t("dash.hint.confirm").to_string()
        } else if state.dash_focus == DashFocus::Services && !state.selected_services.is_empty() {
            state.t("dash.hint.multiselect").to_string()
        } else if state.dash_focus == DashFocus::Sidebar && state.sidebar_filter.is_some() {
            state.t("dash.hint.filter").to_string()
        } else {
            let key = match state.dash_focus {
                DashFocus::Services => "dash.hint.services",
                DashFocus::Sidebar  => state
                    .current_sidebar_item()
                    .map(|i| i.hint_key())
                    .unwrap_or("dash.hint"),
            };
            format!("{}  {}  {}  ", state.t(key), state.t("dash.hint.f1"), state.t("dash.hint.quit"))
        };

        let hint_style = if has_confirm {
            Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::new().fg(Color::DarkGray)
        };

        let hints_w = hint_text.chars().count() as u16;
        if hints_w < area.width {
            let hints_area = Rect {
                x:      area.right().saturating_sub(hints_w),
                y:      area.y,
                width:  hints_w,
                height: 1,
            };
            f.render_stateful_widget(
                Paragraph::new(Line::from(Span::styled(hint_text, hint_style))),
                hints_area,
                &mut ParagraphState::new(),
            );
        }
    }
}
