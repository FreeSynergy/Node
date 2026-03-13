// Root renderer — single entry point for all screen rendering.
//
// Design Pattern: Template Method — this module owns the fixed layout skeleton
// (Header + NavBar + Footer) and delegates the body slot content to the
// composition pair selected by `LayoutSlots::from_state()`.
//
// ┌──────────────────────────────────────────────────────────────────────────┐
// │  HEADER   [Logo + Title + Project Context]              [LangSwitch] [?] │ ← 6 rows, always
// ├──────────────────────────────────────────────────────────────────────────┤
// │  NAV-BAR  [Projects] [Hosts] [Services] [Bots] [Fed.] [Websites] [Store] [⚙] │ ← 1 row, always
// ├──────────────┬───────────────────────────────────────────────────────────┤
// │              │                                                            │
// │  LEFT        │   MAIN                                     RIGHT (opt.)   │
// │  (optional)  │   (always present)                                        │
// │              │                                                            │
// ├──────────────┴───────────────────────────────────────────────────────────┤
// │  FOOTER   [Copyright]                          [Context shortcuts]       │ ← 1 row, always
// └──────────────────────────────────────────────────────────────────────────┘
//
// Body slot routing (from_state):
//   NavTab::Store     → store_screen::render_body      (left=32)
//   NavTab::Settings  → settings_screen::render_body   (left=22)
//   form_queue active → new_project::render (full main, sidebar dim)
//   others            → SidebarList (left=28) + DetailPanel (main)
//
// To add a new tab composition: add a match arm here + sidebar/main render fns.

use ratatui::layout::Rect;

use crate::app::{AppState, NavTab, OverlayLayer};
use crate::ui::render_ctx::RenderCtx;
use crate::ui::layout::{AppLayout, LayoutConfig};
use crate::ui::components::{Component, FooterBar, HeaderBar, NotifStack, SidebarList, DetailPanel};
use crate::ui::compositions::{Composition, NavBarMain};
use crate::ui::{help_sidebar, new_project, settings_screen, store_screen};
use crate::ui::overlays;

// ── Layout constants ──────────────────────────────────────────────────────────

/// Sidebar width for Projects / Hosts / Services / Bots tabs.
const DEFAULT_SIDEBAR_WIDTH: u16 = 28;

/// Returns the left-panel width for the given tab.
fn left_width(tab: NavTab) -> Option<u16> {
    match tab {
        NavTab::Store    => Some(store_screen::SIDEBAR_WIDTH),
        NavTab::Settings => Some(settings_screen::SIDEBAR_WIDTH),
        _                => Some(DEFAULT_SIDEBAR_WIDTH),
    }
}

// ── Public entry point ────────────────────────────────────────────────────────

/// Render everything — the only render function that should be called from
/// the rat-salsa render callback.  Replaces `ui::mod::render`.
pub fn render(f: &mut RenderCtx<'_>, state: &mut AppState) {
    let full = f.area();
    state.click_map.clear();

    // Help sidebar slides in from the right when F1 is active.
    let help_w = (state.help_visible && full.width > help_sidebar::SIDEBAR_WIDTH + 40 + DEFAULT_SIDEBAR_WIDTH)
        .then_some(help_sidebar::SIDEBAR_WIDTH);

    let layout = AppLayout::compute(full, &LayoutConfig {
        topbar_height:  6,
        menubar_height: 1,
        left_width:     left_width(state.active_tab),
        right_width:    help_w,
        ..LayoutConfig::default()
    });

    // ── Fixed chrome (always rendered) ────────────────────────────────────────
    HeaderBar.render(f, layout.topbar, state);
    if let Some(nav_area) = layout.menubar {
        NavBarMain.render(f, nav_area, state);
    }
    FooterBar.render(f, layout.footer_primary, state);

    // ── Body (left + main) ────────────────────────────────────────────────────
    render_body(f, state, &layout);

    // ── Help panel (right slot) ───────────────────────────────────────────────
    if let Some(help_area) = layout.body.right {
        let kind    = state.active_form().map(|f| f.kind);
        let foc_key = state.active_form()
            .and_then(|f| f.focused_node())
            .map(|n| n.key());
        let sections = help_sidebar::build_help(state.screen, kind, foc_key, state.lang);
        help_sidebar::render_help_sidebar(f, help_area, &sections, state.lang);
    }

    // ── Overlay stack (each layer renders on top) ─────────────────────────────
    let layer_count = state.overlay_stack.len();
    for i in 0..layer_count {
        let layer = &state.overlay_stack[i];
        layer.render(f, state);
    }

    // Register Welcome overlay buttons in the click-map.
    // (Overlays render with &AppState, so they can't push to click_map themselves.)
    if matches!(state.top_overlay(), Some(OverlayLayer::Welcome { .. })) {
        use crate::click_map::ClickTarget;
        let (btn1, btn2) = overlays::welcome::button_rects(full, state);
        state.click_map.push(btn1, ClickTarget::WelcomeButton { index: 0 });
        state.click_map.push(btn2, ClickTarget::WelcomeButton { index: 1 });
    }

    // ── Toast notifications (always on top, top-right corner) ─────────────────
    NotifStack.render(f, full, state);
}

// ── Body dispatch ─────────────────────────────────────────────────────────────

fn render_body(f: &mut RenderCtx<'_>, state: &mut AppState, layout: &AppLayout) {
    // Form is open → new_project fills main; sidebar stays but is dim.
    if state.form_queue.is_some() {
        if let Some(left_area) = layout.body.left {
            SidebarList.render(f, left_area, state);
        }
        render_with_help_split(f, state, layout, |f, s, area| {
            new_project::render(f, s, area);
        });
        return;
    }

    match state.active_tab {
        NavTab::Store => {
            store_screen::render_body(f, state, layout.body.left, layout.body.main);
        }
        NavTab::Settings => {
            settings_screen::render_body(f, state, layout.body.left, layout.body.main);
        }
        _ => {
            // Projects / Hosts / Services / Bots / Federation / Websites
            if let Some(left_area) = layout.body.left {
                SidebarList.render(f, left_area, state);
            }
            DetailPanel.render(f, layout.body.main, state);
        }
    }
}

/// Calls `render_fn` with main area, splitting off the help panel if visible.
/// Used when the help sidebar should appear alongside a specific screen.
fn render_with_help_split<R>(
    f:         &mut RenderCtx<'_>,
    state:     &mut AppState,
    layout:    &AppLayout,
    render_fn: R,
) where
    R: FnOnce(&mut RenderCtx<'_>, &mut AppState, Rect),
{
    // The right slot is already carved out by the root layout when help_visible=true.
    // We just render into main — the split is handled at the root level.
    render_fn(f, state, layout.body.main);
}
