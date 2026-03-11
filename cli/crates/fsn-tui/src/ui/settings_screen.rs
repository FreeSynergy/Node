// Settings screen — tabbed preferences panel.
//
// Pattern: Composite — each tab is a self-contained render function.
// Adding a new settings section = add SettingsTab variant + match arm here.
//
// Current tabs:
//   Stores    — module store management (add/remove/enable URLs)
//   Languages — i18n language management (view/activate/remove)
//
// Layout:
//   ┌─────────────────────────────────────────────────────────────┐
//   │  ⚙ Settings                                                 │
//   ├── [Stores] [Languages] ─────────────────────────────────────┤
//   │  (tab content)                                              │
//   ├─────────────────────────────────────────────────────────────┤
//   │  (hint bar)                                                 │
//   └─────────────────────────────────────────────────────────────┘

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders},
};
use rat_widget::paragraph::{Paragraph, ParagraphState};

use crate::app::{AppState, SettingsTab};
use crate::i18n::{TRANSLATION_API_VERSION, t};
use crate::ui::render_ctx::RenderCtx;

pub fn render(f: &mut RenderCtx<'_>, state: &AppState, area: Rect) {
    let block = Block::default()
        .title(Span::styled(
            format!(" {} ", state.t("settings.title")),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Split: tab bar | content | hint
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // tab bar
            Constraint::Min(3),    // content
            Constraint::Length(1), // hint
        ])
        .split(inner);

    render_tab_bar(f, state, chunks[0]);

    match state.settings_tab {
        SettingsTab::Stores    => render_stores(f, state, chunks[1]),
        SettingsTab::Languages => render_languages(f, state, chunks[1]),
    }

    render_hint(f, state, chunks[2]);
}

// ── Tab bar ───────────────────────────────────────────────────────────────────

fn render_tab_bar(f: &mut RenderCtx<'_>, state: &AppState, area: Rect) {
    let lang = state.lang;
    let tabs: &[SettingsTab] = &[SettingsTab::Stores, SettingsTab::Languages];

    let spans: Vec<Span> = tabs.iter().flat_map(|&tab| {
        let label = t(lang, tab.label_key());
        let is_active = tab == state.settings_tab;
        let style = if is_active {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        [Span::styled(format!(" {label} "), style), Span::raw(" ")]
    }).collect();

    f.render_stateful_widget(
        Paragraph::new(Line::from(spans)),
        area,
        &mut ParagraphState::new(),
    );
}

// ── Stores tab ────────────────────────────────────────────────────────────────

fn render_stores(f: &mut RenderCtx<'_>, state: &AppState, area: Rect) {
    let stores = &state.settings.stores;

    if stores.is_empty() {
        f.render_stateful_widget(
            Paragraph::new(Line::from(Span::styled(
                state.t("settings.empty"),
                Style::default().fg(Color::DarkGray),
            ))),
            area,
            &mut ParagraphState::new(),
        );
        return;
    }

    let mut lines: Vec<Line> = Vec::new();
    for (i, store) in stores.iter().enumerate() {
        let is_sel = i == state.settings_cursor;
        let status_key = if store.enabled { "settings.store.enabled" } else { "settings.store.disabled" };
        let status     = state.t(status_key);
        let status_col = if store.enabled { Color::Green } else { Color::DarkGray };
        let marker     = if is_sel { "▶ " } else { "  " };
        let name_style = if is_sel {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        lines.push(Line::from(vec![
            Span::raw(marker),
            Span::styled(store.name.as_str(), name_style),
            Span::raw("  "),
            Span::styled(status, Style::default().fg(status_col)),
        ]));
        lines.push(Line::from(vec![
            Span::raw("    "),
            Span::styled(store.url.as_str(), Style::default().fg(Color::DarkGray)),
        ]));
        lines.push(Line::from(""));
    }

    f.render_stateful_widget(Paragraph::new(lines), area, &mut ParagraphState::new());
}

// ── Languages tab ─────────────────────────────────────────────────────────────

fn render_languages(f: &mut RenderCtx<'_>, state: &AppState, area: Rect) {
    let lang = state.lang;
    let mut lines: Vec<Line<'static>> = Vec::new();

    // English — built-in, always first
    {
        let is_active = matches!(state.lang, crate::app::Lang::En);
        let is_sel    = state.lang_cursor == 0;
        push_lang_row(&mut lines, "EN", "English", is_active, is_sel,
            t(lang, "settings.lang.builtin").to_string(), Color::DarkGray, lang);
    }

    if state.available_langs.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(t(lang, "settings.lang.none"), Style::default().fg(Color::DarkGray)),
        ]));
    } else {
        for (i, dl) in state.available_langs.iter().enumerate() {
            let cursor_idx = i + 1;
            let is_active  = matches!(state.lang, crate::app::Lang::Dynamic(d) if d.code == dl.code);
            let is_sel     = state.lang_cursor == cursor_idx;

            let (api_label, api_color) = if dl.api_version == TRANSLATION_API_VERSION {
                (t(lang, "settings.lang.api_ok"), Color::Green)
            } else {
                (t(lang, "settings.lang.api_warn"), Color::Yellow)
            };
            let info = format!("{}%  {}", dl.completeness, api_label);
            push_lang_row(&mut lines, dl.code_upper, dl.name, is_active, is_sel,
                info, api_color, lang);
        }
    }

    f.render_stateful_widget(Paragraph::new(lines), area, &mut ParagraphState::new());
}

fn push_lang_row(
    lines:     &mut Vec<Line<'static>>,
    code:      &'static str,
    name:      &'static str,
    is_active: bool,
    is_sel:    bool,
    info:      String,
    info_col:  Color,
    lang:      crate::app::Lang,
) {
    let marker     = if is_sel { "▶ " } else { "  " };
    let status_key = if is_active { "settings.lang.active" } else { "settings.lang.inactive" };
    let status     = t(lang, status_key);
    let status_col = if is_active { Color::Green } else { Color::DarkGray };
    let name_style = if is_sel {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    lines.push(Line::from(vec![
        Span::raw(marker),
        Span::styled(format!("[{code}] "), Style::default().fg(Color::Yellow)),
        Span::styled(name, name_style),
        Span::raw("  "),
        Span::styled(status, Style::default().fg(status_col)),
        Span::raw("  "),
        Span::styled(info, Style::default().fg(info_col)),
    ]));
}

// ── Hint bar ─────────────────────────────────────────────────────────────────

fn render_hint(f: &mut RenderCtx<'_>, state: &AppState, area: Rect) {
    let key = match state.settings_tab {
        SettingsTab::Stores    => "settings.stores.hint",
        SettingsTab::Languages => "settings.lang.hint",
    };
    f.render_stateful_widget(
        Paragraph::new(Line::from(Span::styled(
            state.t(key),
            Style::default().fg(Color::DarkGray),
        ))),
        area,
        &mut ParagraphState::new(),
    );
}
