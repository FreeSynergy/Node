// HeaderBar component — logo area (6 rows, no tab bar).
//
// Renders the top 6 rows of the application:
//   Row 0:   top padding (breathing room)
//   Row 1-4: BigText "FSN" logo (left) + title / subtitle / project info / separator (right)
//   Row 5:   bottom padding before nav bar
//
// The navigation tab bar has been extracted to NavBarMain composition
// (ui/compositions/navbar_main.rs) so it lives in its own layout slot
// (menubar_height: 1 in LayoutConfig).

use tui_big_text::{BigText, PixelSize};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
};
use rat_widget::paragraph::{Paragraph, ParagraphState};

use crate::app::AppState;
use crate::click_map::ClickTarget;
use crate::ui::{render_ctx::RenderCtx, widgets};
use super::Component;

pub struct HeaderBar;

impl Component for HeaderBar {
    fn render(&self, f: &mut RenderCtx<'_>, area: Rect, state: &mut AppState) {
        // 1 row top padding + 4 logo rows + 1 row bottom padding = 6 total
        let rows = Layout::vertical([
            Constraint::Length(1), // top padding
            Constraint::Length(4), // BigText logo (Quadrant pixel = 4 rows)
            Constraint::Length(1), // bottom padding
        ])
        .split(area);

        render_logo_row(f, state, rows[1]);
    }
}

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
        Constraint::Length(1), // separator line
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
