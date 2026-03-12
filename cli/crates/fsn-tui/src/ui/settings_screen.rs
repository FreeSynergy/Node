// Settings screen — sidebar-first layout consistent with the Dashboard.
//
// Design Pattern: Composite — left sidebar (section navigation) + right content panel.
// Each SettingsSection has its own content render function.
//
// Layout (reuses AppLayout + HeaderBar + FooterBar):
//   ┌────────────────────────────────────────────────────────────┐
//   │  HeaderBar (5 rows)                                        │
//   ├──────────────┬─────────────────────────────────────────────┤
//   │  Sidebar     │  Content panel                              │
//   │  ▶ Stores    │  (section-specific items)                   │
//   │    Languages │                                             │
//   │    General   │                                             │
//   │    About     │  hint bar at content bottom                 │
//   ├──────────────┴─────────────────────────────────────────────┤
//   │  FooterBar (1 row)                                         │
//   └────────────────────────────────────────────────────────────┘
//
// Mouse registration (ClickMap):
//   SettingsSidebar { idx } — each sidebar section row
//   SettingsCursor  { idx } — each Stores content row
//   LangCursor      { idx } — each Languages content row
//
// Adding a new section:
//   1. Add variant to SettingsSection in app/screen.rs.
//   2. Add label_key() arm in SettingsSection::label_key().
//   3. Add render_* function below.
//   4. Add match arm in render_content() and render_hint().
//   5. Add keyboard handler in events.rs::handle_settings_content().

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders},
};
use rat_widget::paragraph::{Paragraph, ParagraphState};

use crate::app::{AppState, SettingsFocus, SettingsSection};
use crate::click_map::{ClickMap, ClickTarget};
use crate::i18n::{TRANSLATION_API_VERSION, t};
use crate::ui::components::{Component, FooterBar, HeaderBar};
use crate::ui::layout::{AppLayout, LayoutConfig};
use crate::ui::render_ctx::RenderCtx;

/// Width of the settings sidebar column in characters.
const SIDEBAR_WIDTH: u16 = 22;

pub fn render(f: &mut RenderCtx<'_>, state: &mut AppState, area: Rect) {
    // Clear last frame's ClickMap — HeaderBar + content will rebuild it.
    state.click_map.clear();

    let layout = AppLayout::compute(area, &LayoutConfig {
        topbar_height: 5,
        left_width:    Some(SIDEBAR_WIDTH),
        ..LayoutConfig::default()
    });

    HeaderBar.render(f, layout.topbar, state);
    FooterBar.render(f, layout.footer_primary, state);

    let sidebar_area = layout.body.left.unwrap_or(layout.body.main);
    let content_area = layout.body.main;

    // Take ClickMap — avoids simultaneous borrow of state + state.click_map.
    let mut cmap = std::mem::take(&mut state.click_map);
    render_sidebar(f, state, sidebar_area, &mut cmap);
    render_content(f, state, content_area, &mut cmap);
    state.click_map = cmap;
}

// ── Settings sidebar ──────────────────────────────────────────────────────────

fn render_sidebar(f: &mut RenderCtx<'_>, state: &AppState, area: Rect, cmap: &mut ClickMap) {
    let focused = state.settings_focus == SettingsFocus::Sidebar;
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
        x:      area.x + 1,
        y:      area.y,
        width:  area.width.saturating_sub(2),
        height: area.height,
    };

    let lang  = state.lang;
    let lines: Vec<Line> = SettingsSection::ALL.iter().enumerate().map(|(i, &sec)| {
        let is_cursor = i == state.settings_sidebar_cursor;
        let is_active = sec == state.settings_section;

        // Register click target for this row.
        let row_y = inner.y + i as u16;
        if row_y < area.bottom() {
            cmap.push(
                Rect { x: inner.x, y: row_y, width: inner.width, height: 1 },
                ClickTarget::SettingsSidebar { idx: i },
            );
        }

        let marker = if is_cursor && focused { "▶ " } else { "  " };
        let label  = t(lang, sec.label_key());
        let style  = if is_active && focused {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else if is_active {
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
        } else if is_cursor && focused {
            Style::default().fg(Color::White)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        Line::from(vec![
            Span::raw(marker),
            Span::styled(label.to_string(), style),
        ])
    }).collect();

    f.render_stateful_widget(Paragraph::new(lines), inner, &mut ParagraphState::new());
}

// ── Content dispatcher ────────────────────────────────────────────────────────

fn render_content(f: &mut RenderCtx<'_>, state: &AppState, area: Rect, cmap: &mut ClickMap) {
    // Left border separates content from sidebar.
    let block = Block::default()
        .borders(Borders::LEFT)
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = block.inner(area);
    f.render_widget(block, area);

    // Split content into: main area + 1-row hint.
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(inner);

    match state.settings_section {
        SettingsSection::Stores    => render_stores(f, state, chunks[0], cmap),
        SettingsSection::Languages => render_languages(f, state, chunks[0], cmap),
        SettingsSection::General   => render_general(f, state, chunks[0]),
        SettingsSection::About     => render_about(f, state, chunks[0]),
    }

    render_hint(f, state, chunks[1]);
}

// ── Stores content ────────────────────────────────────────────────────────────

fn render_stores(f: &mut RenderCtx<'_>, state: &AppState, area: Rect, cmap: &mut ClickMap) {
    let stores  = &state.settings.stores;
    let focused = state.settings_focus == SettingsFocus::Content
        && state.settings_section == SettingsSection::Stores;

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

    let mut y = area.y;
    for (i, store) in stores.iter().enumerate() {
        if y >= area.bottom() { break; }

        // Height: name+status (1) + URL (1) + optional path + optional git + blank (1)
        let detail_lines = store.local_path.is_some() as u16 + store.git_url.is_some() as u16;
        let item_h = (2 + detail_lines + 1).min(area.bottom().saturating_sub(y));
        let item_rect = Rect { x: area.x, y, width: area.width, height: item_h };

        cmap.push(item_rect, ClickTarget::SettingsCursor { idx: i });

        let is_sel     = focused && i == state.settings_cursor;
        let status_key = if store.enabled { "settings.store.enabled" } else { "settings.store.disabled" };
        let status     = state.t(status_key);
        let status_col = if store.enabled { Color::Green } else { Color::DarkGray };
        let marker     = if is_sel { "▶ " } else { "  " };
        let name_style = if is_sel {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        let mut lines: Vec<Line<'_>> = vec![
            Line::from(vec![
                Span::raw(marker),
                Span::styled(store.name.as_str(), name_style),
                Span::raw("  "),
                Span::styled(status, Style::default().fg(status_col)),
            ]),
            Line::from(vec![
                Span::raw("    "),
                Span::styled("URL:  ", Style::default().fg(Color::DarkGray)),
                Span::styled(store.url.as_str(), Style::default().fg(Color::DarkGray)),
            ]),
        ];
        if let Some(ref lp) = store.local_path {
            lines.push(Line::from(vec![
                Span::raw("    "),
                Span::styled("Path: ", Style::default().fg(Color::DarkGray)),
                Span::styled(lp.as_str(), Style::default().fg(Color::Yellow)),
            ]));
        }
        if let Some(ref gu) = store.git_url {
            lines.push(Line::from(vec![
                Span::raw("    "),
                Span::styled("Git:  ", Style::default().fg(Color::DarkGray)),
                Span::styled(gu.as_str(), Style::default().fg(Color::DarkGray)),
            ]));
        }
        lines.push(Line::from(""));

        f.render_stateful_widget(Paragraph::new(lines), item_rect, &mut ParagraphState::new());
        y += item_h;
    }
}

// ── Languages content ─────────────────────────────────────────────────────────
//
// Design: Unified checkbox list — one row per language, installed = [x], not installed = [ ].
//
// Cursor layout:
//   0              → English (built-in, always [x])
//   1..store_langs → each store language in Store index order
//
// When store_langs is empty (still fetching): fall back to showing only installed langs.
//
// Interactions:
//   Enter  → activate language for UI (only if installed)
//   Space  → toggle: download if [ ], remove if [x]
//   Del/D  → remove installed language
//   ←/Esc  → back to sidebar

fn render_languages(f: &mut RenderCtx<'_>, state: &AppState, area: Rect, cmap: &mut ClickMap) {
    let ui_lang   = state.lang;
    let focused   = state.settings_focus == SettingsFocus::Content
        && state.settings_section == SettingsSection::Languages;
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut line_y = area.y;

    // ── English — built-in, always index 0 ────────────────────────────────────
    {
        let is_active = matches!(state.lang, crate::app::Lang::En);
        let is_sel    = focused && state.lang_cursor == 0;
        if line_y < area.bottom() {
            cmap.push(
                Rect { x: area.x, y: line_y, width: area.width, height: 1 },
                ClickTarget::LangCursor { idx: 0 },
            );
        }
        line_y += 1;
        lines.push(lang_checkbox_row(
            "[x]".to_string(), "EN".to_string(), "English".to_string(),
            is_active, is_sel, true,
            t(ui_lang, "settings.lang.builtin").to_string(),
        ));
    }

    if state.store_langs.is_empty() {
        // Store index not yet fetched — show installed languages only.
        lines.push(Line::from(""));
        line_y += 1;
        for (i, dl) in state.available_langs.iter().enumerate() {
            let cursor_idx = i + 1;
            let is_active  = matches!(state.lang, crate::app::Lang::Dynamic(d) if d.code == dl.code);
            let is_sel     = focused && state.lang_cursor == cursor_idx;
            if line_y < area.bottom() {
                cmap.push(
                    Rect { x: area.x, y: line_y, width: area.width, height: 1 },
                    ClickTarget::LangCursor { idx: cursor_idx },
                );
            }
            line_y += 1;
            let (api_label, _) = if dl.api_version == TRANSLATION_API_VERSION {
                (t(ui_lang, "settings.lang.api_ok"), Color::Green)
            } else {
                (t(ui_lang, "settings.lang.api_warn"), Color::Yellow)
            };
            let info = format!("{}%  {}", dl.completeness, api_label);
            lines.push(lang_checkbox_row(
                "[x]".to_string(), dl.code_upper.to_string(), dl.name.to_string(),
                is_active, is_sel, true, info,
            ));
        }
        if state.available_langs.is_empty() {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    "Loading Store index…",
                    Style::default().fg(Color::DarkGray),
                ),
            ]));
        }
    } else {
        // Store index loaded — show all languages as a unified checkbox list.
        lines.push(Line::from(""));
        line_y += 1;

        for (i, entry) in state.store_langs.iter().enumerate() {
            let cursor_idx  = i + 1;
            let is_installed = state.available_langs.iter().any(|d| d.code == entry.code);
            let is_active    = matches!(state.lang, crate::app::Lang::Dynamic(d) if d.code == entry.code);
            let is_sel       = focused && state.lang_cursor == cursor_idx;

            if line_y < area.bottom() {
                cmap.push(
                    Rect { x: area.x, y: line_y, width: area.width, height: 1 },
                    ClickTarget::LangCursor { idx: cursor_idx },
                );
            }
            line_y += 1;

            let checkbox  = if is_installed { "[x]" } else { "[ ]" };
            let code_up   = entry.code.to_uppercase();
            let info_str  = if is_installed {
                format!("{}%  ✓", entry.completeness)
            } else {
                format!("{}%  ↓ Space", entry.completeness)
            };
            lines.push(lang_checkbox_row(
                checkbox.to_string(), code_up, entry.name.clone(),
                is_active, is_sel, is_installed, info_str,
            ));
        }
    }

    let _ = line_y;
    f.render_stateful_widget(Paragraph::new(lines), area, &mut ParagraphState::new());
}

/// Build one language row for the checkbox list.
///
///   ▶ [x] [DE] Deutsch    ✓ Active   100%  ✓
///     [ ] [FR] Français              100%  ↓ Space
fn lang_checkbox_row(
    checkbox:     String,
    code_upper:   String,
    name:         String,
    is_active:    bool,
    is_sel:       bool,
    is_installed: bool,
    info:         String,
) -> Line<'static> {
    let marker     = if is_sel { "▶ " } else { "  " };
    let chk_col    = if is_installed { Color::Green  } else { Color::DarkGray };
    let name_style = if is_sel {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else if is_installed {
        Style::default().fg(Color::White)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let active_badge: Vec<Span<'static>> = if is_active {
        vec![
            Span::raw("  "),
            Span::styled("✓ Active", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        ]
    } else {
        vec![]
    };
    let mut spans = vec![
        Span::raw(marker),
        Span::styled(checkbox,             Style::default().fg(chk_col)),
        Span::raw(" "),
        Span::styled(format!("[{code_upper}] "), Style::default().fg(Color::Yellow)),
        Span::styled(name,                  name_style),
    ];
    spans.extend(active_badge);
    spans.push(Span::raw("  "));
    spans.push(Span::styled(info, Style::default().fg(Color::DarkGray)));
    Line::from(spans)
}

// ── General content ───────────────────────────────────────────────────────────

fn render_general(f: &mut RenderCtx<'_>, _state: &AppState, area: Rect) {
    // Placeholder — future: theme, log level, auto-update, etc.
    f.render_stateful_widget(
        Paragraph::new(vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("Coming soon", Style::default().fg(Color::DarkGray)),
            ]),
        ]),
        area,
        &mut ParagraphState::new(),
    );
}

// ── About content ─────────────────────────────────────────────────────────────

fn render_about(f: &mut RenderCtx<'_>, _state: &AppState, area: Rect) {
    let build_time = crate::BUILD_TIME;
    let git_hash   = crate::GIT_HASH;
    let version    = env!("CARGO_PKG_VERSION");

    let lines: Vec<Line<'static>> = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("FreeSynergy", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(".Node",       Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Version:    ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("v{version}"), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("Build:      ", Style::default().fg(Color::DarkGray)),
            Span::styled(build_time,    Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("Commit:     ", Style::default().fg(Color::DarkGray)),
            Span::styled(git_hash,      Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("License:    ", Style::default().fg(Color::DarkGray)),
            Span::styled("MIT",         Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("Website:    ", Style::default().fg(Color::DarkGray)),
            Span::styled("freesynergy.net", Style::default().fg(Color::Cyan)),
        ]),
    ];

    f.render_stateful_widget(Paragraph::new(lines), area, &mut ParagraphState::new());
}

// ── Hint bar ──────────────────────────────────────────────────────────────────

fn render_hint(f: &mut RenderCtx<'_>, state: &AppState, area: Rect) {
    let key = match state.settings_focus {
        SettingsFocus::Sidebar => "settings.hint.sidebar",
        SettingsFocus::Content => match state.settings_section {
            SettingsSection::Stores    => "settings.hint.stores",
            SettingsSection::Languages => "settings.hint.languages",
            SettingsSection::General   => "settings.hint.general",
            SettingsSection::About     => "settings.hint.about",
        },
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
