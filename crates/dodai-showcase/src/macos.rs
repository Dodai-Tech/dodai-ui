//! macOS only: the Dock and app-switcher icon. gpui has no window-icon API, and a bare
//! binary (`cargo run`) shows a generic icon until the app sets one.

use objc2_app_kit::{NSApplication, NSImage};
use objc2_foundation::{MainThreadMarker, NSData};

/// Set the Dock icon from `png`. Returns `false` off the main thread or when AppKit
/// cannot read `png`.
pub(crate) fn set_dock_icon(png: &[u8]) -> bool {
    let Some(mtm) = MainThreadMarker::new() else {
        return false;
    };
    let Some(image) = NSImage::initWithData(mtm.alloc(), &NSData::with_bytes(png)) else {
        return false;
    };
    // SAFETY: on the main thread (we hold a `MainThreadMarker`), with a valid, non-nil image.
    unsafe { NSApplication::sharedApplication(mtm).setApplicationIconImage(Some(&image)) };
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dock_icon_is_a_png() {
        assert!(crate::DOCK_ICON.starts_with(b"\x89PNG\r\n\x1a\n"));
    }

    #[test]
    fn off_the_main_thread_nothing_is_set() {
        let set = std::thread::spawn(|| set_dock_icon(crate::DOCK_ICON))
            .join()
            .unwrap();
        assert!(!set);
    }
}
