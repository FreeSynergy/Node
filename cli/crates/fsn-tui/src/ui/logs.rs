// Logs overlay — modal panel over the current screen.

use ratatui::{
    layout::Alignment,
    style::{Color, Style},
    text::{Line, Span},
};
use rat_widget::paragraph::{Paragraph, ParagraphState};

use crate::ui::render_ctx::RenderCtx;

use crate::app::AppState;
use crate::ui::widgets;

pub fn render(f: &mut RenderCtx<'_>, state: &AppState) {
    let Some(ref logs) = state.logs_overlay() else { return };

    let area = f.area();
    let popup = widgets::popup_area(80, 70, area);

    let title = format!(" Logs: {} ", logs.service_name);
    widgets::clear_block(f, popup, &title);

    // Inner area (inside the border)
    use ratatui::layout::Rect;
    let inner = Rect {
        x: popup.x + 1,
        y: popup.y + 1,
        width: popup.width.saturating_sub(2),
        height: popup.height.saturating_sub(3), // leave room for hint
    };

    let visible_lines: Vec<Line> = logs
        .lines
        .iter()
        .skip(logs.scroll)
        .take(inner.height as usize)
        .map(|l| Line::from(Span::styled(l.as_str(), Style::default().fg(Color::White))))
        .collect();

    f.render_stateful_widget(Paragraph::new(visible_lines), inner, &mut ParagraphState::new());

    // Hint bar at bottom of popup
    let hint_area = Rect {
        x: popup.x + 1,
        y: popup.y + popup.height.saturating_sub(2),
        width: popup.width.saturating_sub(2),
        height: 1,
    };
    f.render_stateful_widget(
        Paragraph::new(Line::from(Span::styled(
            state.t("logs.hint"),
            Style::default().fg(Color::DarkGray),
        )))
        .alignment(Alignment::Right),
        hint_area,
        &mut ParagraphState::new(),
    );
}
