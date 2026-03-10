// UI component system.
//
// Design Pattern: Component — each struct is a self-contained rendering unit.
// A Component gets an area Rect and reads what it needs from AppState.
// Components never talk to each other; the screen (dashboard.rs) composes them.
//
// To add a new component:
//   1. Create ui/components/my_component.rs with `pub struct MyComponent;`
//   2. Implement `Component for MyComponent`
//   3. Re-export here: `pub use my_component::MyComponent;`
//   4. Place it in a screen: `MyComponent.render(f, area, state);`

pub mod detail_panel;
pub mod footer_bar;
pub mod header_bar;
pub mod notif_stack;
pub mod sidebar_list;

pub use detail_panel::DetailPanel;
pub use footer_bar::FooterBar;
pub use header_bar::HeaderBar;
pub use notif_stack::NotifStack;
pub use sidebar_list::SidebarList;

use ratatui::layout::Rect;
use crate::ui::render_ctx::RenderCtx;
use crate::app::AppState;

/// Trait for all renderable UI components.
///
/// Each implementor is a unit struct (zero-cost, no heap allocation).
/// `state` is `&mut` to allow components that cache layout Rects for mouse
/// hit-testing (e.g. SidebarList, DetailPanel services table).
pub trait Component {
    fn render(&self, f: &mut RenderCtx<'_>, area: Rect, state: &mut AppState);
}
