//! Live frosted glass over the desktop: the window itself is blurred by the OS (macOS
//! `NSVisualEffectView`, Windows acrylic, Wayland blur), so what shows through a see-through
//! fill is the real, moving content behind the window, blurred as it changes.
//!
//! `gpui` has no way to blur what the app already drew, so this is the one live blur it can
//! do: it covers the whole window, and the app's own canvas turns half see-through
//! ([`CANVAS_ALPHA`]) so the blurred desktop lights it.

use gpui::{App, Window, WindowBackgroundAppearance};
use std::sync::atomic::{AtomicBool, Ordering};

/// How opaque the app canvas stays while the window is blurred.
pub const CANVAS_ALPHA: f32 = 0.55;

static BLURRED: AtomicBool = AtomicBool::new(false);

/// Blur (or stop blurring) what is behind `window`; the canvas turns see-through to show it.
pub fn set_window_blur(window: &mut Window, cx: &mut App, on: bool) {
    BLURRED.store(on, Ordering::Relaxed);
    window.set_background_appearance(if on {
        WindowBackgroundAppearance::Blurred
    } else {
        WindowBackgroundAppearance::Opaque
    });
    cx.refresh_windows();
}

/// Whether the window blurs what is behind it.
pub fn window_blur() -> bool {
    BLURRED.load(Ordering::Relaxed)
}
