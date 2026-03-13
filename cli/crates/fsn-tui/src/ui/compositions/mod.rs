// UI Composition system — swappable slot content.
//
// Design Pattern: Strategy — each Composition is a strategy for rendering
// a named layout slot (left sidebar, main area). The active tab determines
// which strategy pair is used without any if/else in the root renderer.
//
// A Composition differs from a Component:
//   Component  = low-level reusable widget (e.g. SidebarList, HeaderBar)
//   Composition = high-level slot occupant that owns a full layout zone
//
// To add a new composition:
//   1. Create ui/compositions/my_comp.rs with `pub struct MyComp;`
//   2. Implement `Composition for MyComp`.
//   3. Re-export here and add it to LayoutSlots::from_state() if needed.

pub mod navbar_main;

pub use navbar_main::NavBarMain;

use ratatui::layout::Rect;
use crate::ui::render_ctx::RenderCtx;
use crate::app::AppState;

// ── Composition trait ─────────────────────────────────────────────────────────

/// Trait for all named-slot renderers.
///
/// Implementors are zero-cost unit structs (like `Component`).
/// `state` is `&mut` to allow layout-rect caching for mouse hit-testing.
pub trait Composition {
    fn render(&self, f: &mut RenderCtx<'_>, area: Rect, state: &mut AppState);
}
