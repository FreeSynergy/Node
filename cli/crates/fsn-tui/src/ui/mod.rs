// UI rendering — dispatches to screen-specific renderers.
//
// Render takes `&mut AppState` because FormNode::render(&mut self, ...) needs
// to store the last rendered Rect for mouse hit-testing (layout cache).

pub mod dashboard;
pub mod form_node;
pub mod logs;
pub mod new_project;
pub mod nodes;
pub mod welcome;
pub mod widgets;

use ratatui::Frame;
use crate::app::{AppState, OverlayLayer, Screen};

pub fn render(f: &mut Frame, state: &mut AppState) {
    match state.screen {
        Screen::Welcome    => welcome::render(f, state),
        Screen::Dashboard  => dashboard::render(f, state),
        Screen::NewProject => new_project::render(f, state),
    }

    // Overlay layers drawn on top (Ebene system)
    // We need to check each layer type and render accordingly.
    // `logs::render` and `confirm::render` peek at the stack non-destructively.
    for layer in &state.overlay_stack {
        match layer {
            OverlayLayer::Logs(_)     => logs::render(f, state),
            OverlayLayer::Confirm { .. } => render_confirm(f, state),
        }
    }
}

fn render_confirm(f: &mut Frame, state: &AppState) {
    use ratatui::{
        layout::{Alignment, Rect},
        style::{Color, Modifier, Style},
        text::{Line, Span},
        widgets::{Block, Borders, Clear, Paragraph},
    };

    let Some((msg_key, _)) = state.confirm_overlay() else { return };
    let area = f.area();
    let popup = Rect {
        x: area.width / 4,
        y: area.height / 2 - 2,
        width: area.width / 2,
        height: 3,
    };

    f.render_widget(Clear, popup);
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            state.t(msg_key),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )))
        .block(Block::default().borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow)))
        .alignment(Alignment::Center),
        popup,
    );
}
