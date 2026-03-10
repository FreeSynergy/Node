// Animation state — single source of truth for all tick-driven visual effects.
//
// Design Pattern: Single Source of Truth — all animation timings, character sets,
// and derived visual values live here. To change any animation effect, edit only
// this file. Callers never hardcode frame indices or timing constants.
//
// Usage:
//   state.anim.advance()           — called once per 250ms tick
//   state.anim.spinner()           — braille spinner char (deploy overlay)
//   state.anim.running_char()      — pulsing ●/◉ for Running services
//   state.anim.running_color()     — pulsing Green/LightGreen
//   Anim::ttl_bar(elapsed, max, w) — ▓▓▓░░ progress bar for notifications
//   state.anim.notif_width(born, full) — slide-in width for notifications

use std::time::Duration;
use ratatui::style::Color;

// ── Character sets — edit here to change globally ─────────────────────────────

/// Braille spinner, 10 frames × 250ms = 2.5s full cycle.
const SPINNER: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// Running indicator: alternates on slow pulse.
const RUNNING_CHAR_A: &str = "●";   // primary (bright phase)
const RUNNING_CHAR_B: &str = "◉";   // secondary (dim phase)

/// TTL progress bar fill / empty characters.
const BAR_FILLED: char = '▓';
const BAR_EMPTY:  char = '░';

// ── Timing constants (ticks, 1 tick = 250 ms) ─────────────────────────────────

/// Slow pulse half-period → ~1.5s cycle. Used for: running indicator.
const SLOW_HALF: u32 = 6;

/// Fast pulse half-period → ~0.5s cycle. Used for: error/alert flash.
const FAST_HALF: u32 = 2;

/// Notification slide-in duration in ticks (~1s to reach full width).
const SLIDE_TICKS: u32 = 4;

// ── Animation state ───────────────────────────────────────────────────────────

#[derive(Default)]
pub struct Anim {
    tick: u32,
}

impl Anim {
    pub fn new() -> Self { Self::default() }

    /// Advance by one tick (250 ms). Call once per event-loop tick.
    pub fn advance(&mut self) {
        self.tick = self.tick.wrapping_add(1);
    }

    /// Raw tick counter — used to stamp `born_tick` on new notifications.
    pub fn tick(&self) -> u32 { self.tick }

    // ── Spinner ───────────────────────────────────────────────────────────────

    /// Current braille spinner frame. Cycles every 2.5s.
    /// Used in: deploy overlay title, any "loading" indicator.
    pub fn spinner(&self) -> &'static str {
        SPINNER[self.tick as usize % SPINNER.len()]
    }

    // ── Pulse helpers ─────────────────────────────────────────────────────────

    /// Slow pulse phase — alternates every ~1.5s.
    fn slow_phase(&self) -> bool { (self.tick / SLOW_HALF) % 2 == 0 }

    /// Fast pulse phase — alternates every ~0.5s.
    pub fn fast_phase(&self) -> bool { (self.tick / FAST_HALF) % 2 == 0 }

    // ── Running indicator ─────────────────────────────────────────────────────

    /// Animated char for a Running service. Alternates ●/◉ on slow pulse.
    /// Used in: sidebar service indicator, services table status column.
    pub fn running_char(&self) -> &'static str {
        if self.slow_phase() { RUNNING_CHAR_A } else { RUNNING_CHAR_B }
    }

    /// Animated color for a Running service. Pulses Green ↔ LightGreen.
    pub fn running_color(&self) -> Color {
        if self.slow_phase() { Color::Green } else { Color::LightGreen }
    }

    // ── Notification animations ───────────────────────────────────────────────

    /// Width (0..=full_width) for notification slide-in effect.
    /// Grows from 0 to full_width over SLIDE_TICKS ticks (~1s).
    pub fn notif_width(&self, born_tick: u32, full_width: u16) -> u16 {
        let age = self.tick.wrapping_sub(born_tick);
        let progress = (age as f32 / SLIDE_TICKS as f32).min(1.0);
        (full_width as f32 * progress).round() as u16
    }

    // ── TTL progress bar ──────────────────────────────────────────────────────

    /// Render a ▓▓▓░░░ TTL bar. `elapsed` of `max` has been consumed.
    /// Width controls total character count.
    /// Used in: notification second row.
    pub fn ttl_bar(elapsed: Duration, max: Duration, width: usize) -> String {
        if width == 0 { return String::new(); }
        let ratio = 1.0 - (elapsed.as_secs_f32() / max.as_secs_f32()).clamp(0.0, 1.0);
        let filled = (ratio * width as f32).round() as usize;
        let empty  = width.saturating_sub(filled);
        let mut bar = String::with_capacity(width);
        for _ in 0..filled { bar.push(BAR_FILLED); }
        for _ in 0..empty  { bar.push(BAR_EMPTY);  }
        bar
    }
}
