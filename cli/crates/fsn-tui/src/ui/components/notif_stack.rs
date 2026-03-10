// NotifStack component — toast notifications in the top-right corner.
//
// Design: 2-row toast style.
//   Row 0: " ICON  Message text here               "  (fg=color, bold icon)
//   Row 1: " ▓▓▓▓▓▓▓▓▓░░░░░  (TTL bar, 4s max)    "  (fg=DarkGray)
//
// Slide-in: width grows from 0 to full_width over ~1s (via Anim::notif_width).
// To change the look: edit only this file.
// To change timing/characters: edit ui/anim.rs.

use std::time::Duration;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Clear,
};
use rat_widget::paragraph::{Paragraph, ParagraphState};

use crate::app::{AppState, NotifKind};
use crate::ui::{anim::Anim, render_ctx::RenderCtx};
use super::Component;

const NOTIF_TTL_SECS:   f32 = 4.0;
const NOTIF_FULL_WIDTH: u16 = 52;
const NOTIF_HEIGHT:     u16 = 2; // rows per notification

pub struct NotifStack;

impl Component for NotifStack {
    fn render(&self, f: &mut RenderCtx<'_>, area: Rect, state: &mut AppState) {
        if state.notifications.is_empty() { return; }

        let max_w = NOTIF_FULL_WIDTH.min(area.width.saturating_sub(2));

        for (i, notif) in state.notifications.iter().enumerate() {
            let base_y = area.y + (i as u16) * NOTIF_HEIGHT;
            if base_y + NOTIF_HEIGHT > area.bottom() { break; }

            let (color, icon) = match notif.kind {
                NotifKind::Success => (Color::Green,  "✓"),
                NotifKind::Warning => (Color::Yellow, "!"),
                NotifKind::Error   => (Color::Red,    "✗"),
                NotifKind::Info    => (Color::Cyan,   "i"),
            };

            // Slide-in width
            let full_w = (notif.message.chars().count() as u16 + 6).min(max_w);
            let width  = state.anim.notif_width(notif.born_tick, full_w).max(3);
            let x      = area.right().saturating_sub(width + 1);

            // Row 0: icon + message
            let msg_text = format!(" {}  {} ", icon, notif.message);
            let msg_line = Line::from(vec![
                Span::styled(
                    msg_text.chars().take(width as usize).collect::<String>(),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ),
            ]);
            let row0 = Rect { x, y: base_y, width, height: 1 };
            f.render_widget(Clear, row0);
            f.render_stateful_widget(Paragraph::new(msg_line), row0, &mut ParagraphState::new());

            // Row 1: TTL progress bar
            let bar_w    = (width as usize).saturating_sub(2);
            let bar      = Anim::ttl_bar(
                notif.born.elapsed(),
                Duration::from_secs_f32(NOTIF_TTL_SECS),
                bar_w,
            );
            let bar_text = format!(" {}", bar);
            let row1     = Rect { x, y: base_y + 1, width, height: 1 };
            f.render_widget(Clear, row1);
            f.render_stateful_widget(
                Paragraph::new(Line::from(Span::styled(bar_text, Style::default().fg(Color::DarkGray)))),
                row1,
                &mut ParagraphState::new(),
            );
        }
    }
}
