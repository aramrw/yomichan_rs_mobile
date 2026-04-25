//! Interactive demo modules showcasing GPUI capabilities.
//!
//! These demos work on both iOS and Android and demonstrate GPUI's
//! rendering, animation, and input handling on mobile devices.
//!
//! ## Demos
//!
//! - **Animation Playground** — Bouncing balls with physics, trails, and
//!   particle effects. Tap to spawn, swipe to fling.
//! - **Shader Showcase** — Dynamic gradient backgrounds, floating orbs with
//!   parallax, and ripple effects on touch.

mod animation_playground;
mod menu;
mod shader_showcase;

pub use animation_playground::AnimationPlayground;
pub use menu::{back_button, DemoApp};
pub use shader_showcase::ShaderShowcase;

// Color palette — Google Material theme (shared across demos)
pub const BACKGROUND: u32 = 0x121318;
pub const SURFACE: u32 = 0x1E1F25;
pub const OVERLAY: u32 = 0x282A2F;
pub const TEXT: u32 = 0xE2E2E9;
pub const SUBTEXT: u32 = 0xC4C6D0;
pub const RED: u32 = 0xEA4335;
pub const GREEN: u32 = 0x34A853;
pub const BLUE: u32 = 0x4285F4;
pub const YELLOW: u32 = 0xFBBC04;
pub const PINK: u32 = 0xF538A0;
pub const MAUVE: u32 = 0xA142F4;
pub const PEACH: u32 = 0xFA7B17;
pub const TEAL: u32 = 0x24C1E0;
pub const SKY: u32 = 0x4FC3F7;
pub const LAVENDER: u32 = 0xb4befe;

/// Get a colour from the vibrant palette, cycling by `seed`.
pub fn random_color(seed: usize) -> u32 {
    const COLORS: [u32; 10] = [
        RED, GREEN, BLUE, YELLOW, PINK, MAUVE, PEACH, TEAL, SKY, LAVENDER,
    ];
    COLORS[seed % COLORS.len()]
}
