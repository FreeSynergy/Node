// Service slot popup — floating picker overlay for ServiceSlotNode.
//
// Design Pattern: Strategy — renders a categorized service list with a type-filter.
// Extracted from ServiceSlotNode to isolate all popup state and rendering logic.
//
// The popup does NOT own the entry list. Entries are passed in on each method call
// to avoid data duplication and lifetime complexity. ServiceSlotNode owns the entries.
//
// API contract:
//   handle_key / handle_mouse → return (FormAction, Option<String>)
//     Some(value) = a selection was confirmed; caller must update its own value field.
//     None        = no selection change (navigation, filter cycle, cancel, etc.)

use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear},
};
use rat_widget::paragraph::{Paragraph, ParagraphState};

use crate::ui::form_node::FormAction;
use crate::ui::render_ctx::RenderCtx;
use super::service_slot::{SlotCategory, SlotEntry};

// ── ServiceSlotPopup ──────────────────────────────────────────────────────────

/// Popup state and rendering for the service slot picker.
///
/// Owns all floating-UI state: filter row, cursor, rendered rects for mouse
/// hit-testing. The entry list stays on `ServiceSlotNode` and is passed in
/// by reference so there is no data duplication.
#[derive(Debug)]
pub struct ServiceSlotPopup {
    pub is_open:         bool,
    pub type_filter_idx: usize,
    pub show_filter:     bool,
    pub type_options:    Vec<String>,
    on_filter_row:       bool,
    cursor:              usize,
    // Populated during render(), used for mouse hit-testing.
    rendered_rect:       Option<Rect>,
    filter_row_rect:     Option<Rect>,
    item_rects:          Vec<(usize, Rect)>,
}

impl ServiceSlotPopup {
    pub fn new(type_options: Vec<String>, type_filter_idx: usize, show_filter: bool) -> Self {
        Self {
            is_open: false,
            type_filter_idx,
            show_filter,
            type_options,
            on_filter_row: false,
            cursor: 0,
            rendered_rect: None,
            filter_row_rect: None,
            item_rects: Vec::new(),
        }
    }

    // ── Entry filtering ────────────────────────────────────────────────────

    /// Entries visible under the current type filter.
    pub fn visible_entries<'a>(&self, entries: &'a [SlotEntry]) -> Vec<&'a SlotEntry> {
        let filter = &self.type_options[self.type_filter_idx];
        if filter == "all" {
            entries.iter().collect()
        } else {
            entries.iter()
                .filter(|e| &e.service_type == filter || e.service_type.is_empty())
                .collect()
        }
    }

    // ── Filter cycling ─────────────────────────────────────────────────────

    fn cycle_filter_right(&mut self) {
        let n = self.type_options.len();
        self.type_filter_idx = (self.type_filter_idx + 1) % n;
        self.cursor = 0;
    }

    fn cycle_filter_left(&mut self) {
        let n = self.type_options.len();
        self.type_filter_idx = (self.type_filter_idx + n - 1) % n;
        self.cursor = 0;
    }

    // ── Open / confirm ─────────────────────────────────────────────────────

    /// Open the popup, positioning the cursor at the currently selected entry.
    pub fn open(&mut self, entries: &[SlotEntry], current_value: &str) {
        let visible = self.visible_entries(entries);
        self.cursor = visible.iter()
            .position(|e| e.value == current_value)
            .unwrap_or(0);
        self.on_filter_row = false;
        self.is_open = true;
    }

    /// Confirm the entry at the current cursor. Returns (action, new_value).
    fn confirm_current(&mut self, entries: &[SlotEntry]) -> (FormAction, String) {
        let visible = self.visible_entries(entries);
        let value = visible.get(self.cursor)
            .map(|e| e.value.clone())
            .unwrap_or_default();
        self.is_open = false;
        (FormAction::AcceptAndNext, value)
    }

    // ── Key handling ───────────────────────────────────────────────────────

    /// Handle a key event while the popup is open.
    ///
    /// Returns `(action, Some(new_value))` when a selection is confirmed,
    /// `(action, None)` for all other outcomes (navigation, cancel, filter cycle).
    pub fn handle_key(&mut self, key: KeyEvent, entries: &[SlotEntry]) -> (FormAction, Option<String>) {
        let vis_len = self.visible_entries(entries).len();

        match key.code {
            KeyCode::Esc => {
                self.is_open = false;
                (FormAction::Consumed, None)
            }
            KeyCode::Up => {
                if self.on_filter_row {
                    // already at top — no-op
                } else if self.cursor == 0 {
                    self.on_filter_row = true;
                } else {
                    self.cursor -= 1;
                }
                (FormAction::Consumed, None)
            }
            KeyCode::Down => {
                if self.on_filter_row {
                    self.on_filter_row = false;
                } else if vis_len > 0 {
                    self.cursor = (self.cursor + 1).min(vis_len - 1);
                }
                (FormAction::Consumed, None)
            }
            KeyCode::Left => {
                if self.on_filter_row {
                    self.cycle_filter_left();
                } else {
                    // Left outside filter = close (cancel)
                    self.is_open = false;
                }
                (FormAction::Consumed, None)
            }
            KeyCode::Right => {
                if self.on_filter_row {
                    self.cycle_filter_right();
                    (FormAction::Consumed, None)
                } else {
                    let (action, value) = self.confirm_current(entries);
                    (action, Some(value))
                }
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                if self.on_filter_row {
                    self.cycle_filter_right();
                    (FormAction::Consumed, None)
                } else {
                    let (action, value) = self.confirm_current(entries);
                    (action, Some(value))
                }
            }
            _ => (FormAction::Consumed, None), // swallow all other keys while open
        }
    }

    // ── Mouse handling ─────────────────────────────────────────────────────

    /// Handle a mouse event directed at the open popup.
    ///
    /// Returns `Some((action, Some(new_value)))` when a selection is confirmed,
    /// `Some((action, None))` for other handled events (scroll, filter, close),
    /// `None` when the event should fall through.
    pub fn handle_mouse(&mut self, event: MouseEvent, entries: &[SlotEntry]) -> Option<(FormAction, Option<String>)> {
        let popup_rect = self.rendered_rect?;
        let col = event.column;
        let row = event.row;

        match event.kind {
            MouseEventKind::ScrollUp => {
                if !self.on_filter_row && self.cursor > 0 {
                    self.cursor -= 1;
                }
                return Some((FormAction::Consumed, None));
            }
            MouseEventKind::ScrollDown => {
                if !self.on_filter_row {
                    let vis_len = self.visible_entries(entries).len();
                    if vis_len > 0 {
                        self.cursor = (self.cursor + 1).min(vis_len - 1);
                    }
                }
                return Some((FormAction::Consumed, None));
            }
            MouseEventKind::Down(MouseButton::Left) => {}
            _ => return None,
        }

        // Click outside popup — close (preserve selection, no cancel)
        let outside = col < popup_rect.x || col >= popup_rect.right()
            || row < popup_rect.y || row >= popup_rect.bottom();
        if outside {
            self.is_open = false;
            return Some((FormAction::Consumed, None));
        }

        // Click on filter row
        if let Some(fr) = self.filter_row_rect {
            if row == fr.y {
                self.cycle_filter_right();
                return Some((FormAction::Consumed, None));
            }
        }

        // Click on item
        let item_rects = self.item_rects.clone();
        for (vis_idx, item_rect) in &item_rects {
            if row == item_rect.y && col >= item_rect.x && col < item_rect.right() {
                self.cursor = *vis_idx;
                let (action, value) = self.confirm_current(entries);
                return Some((action, Some(value)));
            }
        }

        Some((FormAction::Consumed, None))
    }

    // ── Rendering ──────────────────────────────────────────────────────────

    /// Render the popup overlay onto the full screen area.
    pub fn render(
        &mut self,
        f:         &mut RenderCtx<'_>,
        screen:    Rect,
        entries:   &[SlotEntry],
        label_key: &'static str,
    ) {
        let visible: Vec<&SlotEntry> = self.visible_entries(entries);
        let n_items = visible.len();

        let popup_w   = (52_u16).min(screen.width);
        let content_h = (n_items as u16) + 5; // items + separators + borders + filter
        let popup_h   = content_h.min(24).min(screen.height);

        let popup = Rect {
            x:      screen.width.saturating_sub(popup_w) / 2,
            y:      screen.height.saturating_sub(popup_h) / 2,
            width:  popup_w,
            height: popup_h,
        };
        self.rendered_rect = Some(popup);

        let title_text = f.translate(label_key);
        let block = Block::default()
            .title(Span::styled(
                format!(" {} ", title_text),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(popup);
        f.render_widget(Clear, popup);
        f.render_widget(block, popup);

        if inner.height == 0 || inner.width == 0 {
            return;
        }

        // Filter row — only shown when no specific type is pre-set
        let items_top = if self.show_filter {
            let filter_rect = Rect { height: 1, ..inner };
            self.filter_row_rect = Some(filter_rect);

            let current_filter = &self.type_options[self.type_filter_idx];
            let filter_style = if self.on_filter_row {
                Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            let filter_line = Line::from(vec![
                Span::styled("  Filter: ", Style::default().fg(Color::DarkGray)),
                Span::styled("◀ ",         Style::default().fg(Color::Cyan)),
                Span::styled(current_filter.clone(), filter_style),
                Span::styled(" ▶",         Style::default().fg(Color::Cyan)),
            ]);
            f.render_stateful_widget(
                Paragraph::new(filter_line),
                filter_rect,
                &mut ParagraphState::new(),
            );
            inner.y + 1
        } else {
            self.filter_row_rect = None;
            inner.y
        };

        if items_top >= inner.bottom() {
            return;
        }
        let items_area = Rect {
            y:      items_top,
            height: inner.bottom().saturating_sub(items_top),
            ..inner
        };

        self.item_rects.clear();
        let mut row_y = items_area.y;
        let mut last_category: Option<SlotCategory> = None;
        let mut visible_idx = 0usize;

        for entry in &visible {
            let cat = entry.category;
            if last_category != Some(cat) && row_y < items_area.bottom() {
                let sep_label = match cat {
                    SlotCategory::Configured => " ── Services ──",
                    SlotCategory::Available  => " ── Available ──",
                    SlotCategory::Store      => " ── Store ──",
                };
                let sep_rect = Rect { y: row_y, height: 1, ..items_area };
                f.render_stateful_widget(
                    Paragraph::new(Line::from(Span::styled(
                        sep_label,
                        Style::default().fg(Color::DarkGray),
                    ))),
                    sep_rect,
                    &mut ParagraphState::new(),
                );
                row_y += 1;
                last_category = Some(cat);
            }

            if row_y >= items_area.bottom() {
                break;
            }

            let is_cursor  = visible_idx == self.cursor;
            let marker     = if is_cursor { "◉ " } else { "○ " };
            let (cat_icon, cat_color) = match cat {
                SlotCategory::Configured => ("✓", Color::Green),
                SlotCategory::Available  => ("+", Color::Yellow),
                SlotCategory::Store      => ("↓", Color::Blue),
            };

            let item_rect = Rect { y: row_y, height: 1, ..items_area };
            self.item_rects.push((visible_idx, item_rect));

            let item_style = if is_cursor {
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let sep_style  = Style::default().fg(Color::DarkGray);
            let type_style = Style::default().fg(Color::DarkGray);
            let icon_style = Style::default().fg(cat_color);

            // Reserve space: "  ◉ " (4) + "  ·  icon" (7) + optional "  ·  type"
            let type_reserve  = if entry.service_type.is_empty() { 0 } else { 5 + entry.service_type.len() };
            let base_reserve  = 4 + 7 + type_reserve;
            let label_width   = (items_area.width as usize).saturating_sub(base_reserve).max(1);
            let display_trunc = if entry.display.len() > label_width {
                format!("{:.width$}", entry.display, width = label_width)
            } else {
                entry.display.clone()
            };

            let mut spans = vec![
                Span::styled(format!("  {}{}", marker, display_trunc), item_style),
            ];
            if !entry.service_type.is_empty() {
                spans.push(Span::styled("  ·  ", sep_style));
                spans.push(Span::styled(entry.service_type.clone(), type_style));
            }
            spans.push(Span::styled("  ·  ", sep_style));
            spans.push(Span::styled(cat_icon, icon_style));

            f.render_stateful_widget(
                Paragraph::new(Line::from(spans)),
                item_rect,
                &mut ParagraphState::new(),
            );

            row_y += 1;
            visible_idx += 1;
        }

        // Hint line at the bottom
        if row_y < items_area.bottom() {
            let hint_rect = Rect {
                y:      items_area.bottom().saturating_sub(1),
                height: 1,
                ..items_area
            };
            f.render_stateful_widget(
                Paragraph::new(Line::from(Span::styled(
                    "↑↓=Navigate  Enter=Select  Esc=Cancel",
                    Style::default().fg(Color::DarkGray),
                ))),
                hint_rect,
                &mut ParagraphState::new(),
            );
        }
    }
}
