// Service slot node — categorized service picker with type-filter.
//
// Design Pattern: Composite — field display + popup logic combined via delegation.
//   ServiceSlotNode  — FormNode impl, closed-state render, entry ownership.
//   ServiceSlotPopup — all popup state, rendering, key/mouse handling.
//
// UX: focused field shows current value + "▼" hint (same as SelectInputNode).
//     ↓/↑/Enter opens a centered popup.
//     Inside popup: filter row (Left/Right/Enter cycle type), item list (↑↓ navigate).
//     Enter/→ on item = confirm. Esc = close without change. Mouse supported.

use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders},
};
use rat_widget::paragraph::{Paragraph, ParagraphState};

use crate::app::Lang;
use crate::ui::form_node::{handle_selection_nav, FormAction, FormNode};
use crate::ui::render_ctx::RenderCtx;
use super::service_slot_popup::ServiceSlotPopup;

// ── SlotCategory ──────────────────────────────────────────────────────────────

/// Which category a service entry belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotCategory {
    Configured,
    Available,
    Store,
}

// ── SlotEntry ─────────────────────────────────────────────────────────────────

/// One item in the service slot popup.
#[derive(Debug, Clone)]
pub struct SlotEntry {
    /// Human-readable label shown in the popup.
    pub display:      String,
    /// Encoded value stored in the form field.
    pub value:        String,
    pub category:     SlotCategory,
    /// Short service type tag, e.g. "iam", "proxy", "" for external.
    pub service_type: String,
}

impl SlotEntry {
    /// A locally configured service instance.
    pub fn configured(name: &str, svc_type: &str) -> Self {
        Self {
            display:      name.to_string(),
            value:        name.to_string(),
            category:     SlotCategory::Configured,
            service_type: svc_type.to_string(),
        }
    }

    /// A locally available (compiled-in) service class not yet deployed.
    /// `value` = `"new:{class}"`.
    pub fn available(class: &str, display: &str, svc_type: &str) -> Self {
        Self {
            display:      display.to_string(),
            value:        format!("new:{}", class),
            category:     SlotCategory::Available,
            service_type: svc_type.to_string(),
        }
    }

    /// A module available in the store (download required).
    /// `value` = `"store:{id}"`.
    pub fn store_module(id: &str, display: &str, svc_type: &str) -> Self {
        Self {
            display:      display.to_string(),
            value:        format!("store:{}", id),
            category:     SlotCategory::Store,
            service_type: svc_type.to_string(),
        }
    }

    /// An externally hosted service (no local deployment).
    #[allow(dead_code)]
    pub fn external() -> Self {
        Self {
            display:      "External service".to_string(),
            value:        "external".to_string(),
            category:     SlotCategory::Available,
            service_type: String::new(),
        }
    }
}

// ── ServiceSlotNode ───────────────────────────────────────────────────────────

/// Service slot form field — composite picker with type filter + categorized list.
///
/// The node owns the entry list and delegates all popup behavior to
/// [`ServiceSlotPopup`]. The FormNode impl wires the two together.
#[derive(Debug)]
pub struct ServiceSlotNode {
    pub key:       &'static str,
    pub label_key: &'static str,
    pub hint_key:  Option<&'static str>,
    pub tab:       usize,
    pub required:  bool,
    pub value:     String,
    pub col_span:  u8,
    pub min_width: u16,

    entries: Vec<SlotEntry>,
    popup:   ServiceSlotPopup,
}

impl ServiceSlotNode {
    /// Create a new service slot node.
    ///
    /// `default_type` pre-selects a type filter (e.g. `"iam"`).
    /// Pass `""` to start on "all".
    pub fn new(
        key:          &'static str,
        label_key:    &'static str,
        tab:          usize,
        required:     bool,
        entries:      Vec<SlotEntry>,
        default_type: &str,
    ) -> Self {
        // Build type_options: ["all"] + unique service_type values (non-empty, in order)
        let mut seen = std::collections::HashSet::new();
        let mut type_options = vec!["all".to_string()];
        for e in &entries {
            if !e.service_type.is_empty() && seen.insert(e.service_type.clone()) {
                type_options.push(e.service_type.clone());
            }
        }

        let type_filter_idx = if default_type.is_empty() {
            0
        } else {
            type_options.iter().position(|t| t == default_type).unwrap_or(0)
        };

        let show_filter = default_type.is_empty();

        Self {
            key, label_key, hint_key: None, tab, required,
            value: String::new(),
            col_span: 12, min_width: 0,
            entries,
            popup: ServiceSlotPopup::new(type_options, type_filter_idx, show_filter),
        }
    }

    // ── Builder helpers ────────────────────────────────────────────────────

    pub fn hint(mut self, k: &'static str) -> Self { self.hint_key = Some(k); self }
    pub fn col(mut self, n: u8)             -> Self { self.col_span = n.min(12).max(1); self }
    pub fn min_w(mut self, n: u16)          -> Self { self.min_width = n; self }

    pub fn with_value(mut self, v: &str) -> Self {
        self.value = v.to_string();
        self
    }

    // ── Display ────────────────────────────────────────────────────────────

    /// Human-readable label for the current value shown in the closed field.
    fn display_value(&self) -> String {
        if self.value.is_empty()      { return "—".to_string(); }
        if self.value == "external"   { return "External service".to_string(); }
        if let Some(class) = self.value.strip_prefix("new:") {
            return format!("+ {}", class.split('/').last().unwrap_or(class));
        }
        if let Some(id) = self.value.strip_prefix("store:") {
            return format!("↓ {}", id.split('/').last().unwrap_or(id));
        }
        self.value.clone()
    }
}

// ── FormNode impl ─────────────────────────────────────────────────────────────

impl FormNode for ServiceSlotNode {
    fn key(&self)       -> &'static str         { self.key }
    fn label_key(&self) -> &'static str         { self.label_key }
    fn hint_key(&self)  -> Option<&'static str> { self.hint_key }
    fn tab(&self)       -> usize                { self.tab }
    fn required(&self)  -> bool                 { self.required }
    fn col_span(&self)  -> u8                   { self.col_span }
    fn min_width(&self) -> u16                  { self.min_width }

    fn value(&self)           -> &str { &self.value }
    fn effective_value(&self) -> &str { &self.value }

    fn set_value(&mut self, v: &str)  { self.value = v.to_string(); }
    fn is_dirty(&self)  -> bool       { false }
    fn set_dirty(&mut self, _v: bool) {}

    fn is_focusable(&self)    -> bool { true }
    fn preferred_height(&self) -> u16 { 4 }
    fn is_filled(&self)       -> bool { !self.value.is_empty() }

    fn render(&mut self, f: &mut RenderCtx<'_>, area: Rect, focused: bool, lang: Lang) {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Length(1)])
            .split(area);

        let label_text   = crate::i18n::t(lang, self.label_key);
        let req_suffix   = if self.required { " *" } else { "" };
        let label_style  = if focused {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        let border_style = if focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let display    = self.display_value();
        let input_line = if focused {
            Line::from(vec![
                Span::styled(display, Style::default().fg(Color::White)),
                Span::styled(" ▼", Style::default().fg(Color::Cyan)),
            ])
        } else {
            Line::from(Span::styled(display, Style::default().fg(Color::White)))
        };

        f.render_stateful_widget(
            Paragraph::new(input_line)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .border_style(border_style)
                    .title(Line::from(Span::styled(
                        format!(" {}{} ", label_text, req_suffix),
                        label_style,
                    )))),
            rows[0],
            &mut ParagraphState::new(),
        );

        if let Some(hk) = self.hint_key {
            f.render_stateful_widget(
                Paragraph::new(Line::from(Span::styled(
                    crate::i18n::t(lang, hk),
                    Style::default().fg(Color::DarkGray),
                ))),
                rows[1],
                &mut ParagraphState::new(),
            );
        }
    }

    fn render_overlay(&mut self, f: &mut RenderCtx<'_>, _available: Rect, lang: Lang) {
        if self.popup.is_open {
            self.popup.render(f, f.area(), &self.entries, self.label_key, lang);
        }
    }

    fn has_open_popup(&self) -> bool { self.popup.is_open }

    fn handle_key(&mut self, key: KeyEvent) -> FormAction {
        if self.popup.is_open {
            let (action, new_value) = self.popup.handle_key(key, &self.entries);
            if let Some(v) = new_value { self.value = v; }
            return action;
        }

        if let Some(nav) = handle_selection_nav(key) { return nav; }

        match key.code {
            KeyCode::Down | KeyCode::Up | KeyCode::Enter => {
                self.popup.open(&self.entries, &self.value);
                FormAction::Consumed
            }
            _ => FormAction::Unhandled,
        }
    }

    fn handle_mouse(&mut self, event: MouseEvent, _area: Rect) -> FormAction {
        if event.kind == MouseEventKind::Down(MouseButton::Left) {
            self.popup.open(&self.entries, &self.value);
            return FormAction::Consumed;
        }
        FormAction::Unhandled
    }

    fn handle_popup_mouse(&mut self, event: MouseEvent) -> Option<FormAction> {
        let (action, new_value) = self.popup.handle_mouse(event, &self.entries)?;
        if let Some(v) = new_value { self.value = v; }
        Some(action)
    }
}
