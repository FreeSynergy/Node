// RenderCtx — thin adapter bridging rat-salsa's (Rect, &mut Buffer) API
// to the Frame-style API used throughout the render modules.
//
// rat-salsa calls render(area, buf, state, ctx) — no Frame provided.
// All existing render functions are written against Frame::render_widget() etc.
// RenderCtx exposes the same surface so the function bodies stay unchanged;
// only the type in the signature changes.
//
// Design: Facade pattern — same interface, different backend.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::{StatefulWidget, Widget};

pub struct RenderCtx<'a> {
    /// Full area of the terminal for this frame.
    area: Rect,
    /// Mutable reference to the terminal buffer.
    buf: &'a mut Buffer,
    /// Cursor position to apply after the frame is done.
    /// Set via `set_cursor_position()`, read by the rat-salsa render callback.
    cursor: Option<(u16, u16)>,
}

impl<'a> RenderCtx<'a> {
    pub fn new(area: Rect, buf: &'a mut Buffer) -> Self {
        Self { area, buf, cursor: None }
    }

    /// The full terminal area — mirrors `Frame::area()`.
    #[inline]
    pub fn area(&self) -> Rect {
        self.area
    }

    /// Render a stateless widget — mirrors `Frame::render_widget()`.
    #[inline]
    pub fn render_widget<W: Widget>(&mut self, widget: W, area: Rect) {
        widget.render(area, self.buf);
    }

    /// Render a stateful widget — mirrors `Frame::render_stateful_widget()`.
    #[inline]
    pub fn render_stateful_widget<W>(&mut self, widget: W, area: Rect, state: &mut W::State)
    where
        W: StatefulWidget,
    {
        widget.render(area, self.buf, state);
    }

    /// Set the cursor position for this frame — mirrors `Frame::set_cursor_position()`.
    /// The rat-salsa callback reads this and forwards it to `ctx.set_screen_cursor()`.
    #[inline]
    pub fn set_cursor_position(&mut self, pos: (u16, u16)) {
        self.cursor = Some(pos);
    }

    /// Extract the stored cursor position (called once by the render callback).
    #[inline]
    pub fn take_cursor(&mut self) -> Option<(u16, u16)> {
        self.cursor.take()
    }

    /// Direct buffer access for widgets that need it (e.g. Clear).
    #[inline]
    pub fn buffer_mut(&mut self) -> &mut Buffer {
        self.buf
    }
}
