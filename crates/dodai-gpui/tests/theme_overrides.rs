//! App-wide theme overrides (`set_corners`, `set_glow_intensity`, `set_scanlines`) apply to
//! every installed theme, survive switching, and undo only what they changed.

use dodai_core::Corners;
use dodai_gpui::*;
use gpui::TestAppContext;

#[gpui::test]
fn reset_corners_keeps_later_glow_and_scanlines(cx: &mut TestAppContext) {
    cx.update(|cx| {
        install_switcher(cx, dodai_core::switcher());
        let own: Vec<Corners> = cx
            .dodai()
            .switcher()
            .themes()
            .iter()
            .map(|t| t.shape.corners)
            .collect();
        set_corners(cx, Corners::ALL);
        assert!(corners_overridden(cx));
        assert!(cx
            .dodai()
            .switcher()
            .themes()
            .iter()
            .all(|t| t.shape.corners == Corners::ALL));
        set_glow_intensity(cx, 0.25);
        set_scanlines(cx, true);
        reset_corners(cx);
        assert!(!corners_overridden(cx));
        for (t, c) in cx.dodai().switcher().themes().iter().zip(&own) {
            assert_eq!(t.shape.corners, *c, "{} got its own corners back", t.id());
            assert_eq!(t.effects.glow_intensity, 0.25, "{} kept the glow", t.id());
            assert!(t.effects.scanlines, "{} kept the scanlines", t.id());
        }
    });
}

#[gpui::test]
fn a_new_switcher_drops_the_old_corner_override(cx: &mut TestAppContext) {
    cx.update(|cx| {
        install_switcher(cx, dodai_core::switcher());
        set_corners(cx, Corners::ALL);
        let fresh = dodai_core::switcher();
        let own: Vec<Corners> = fresh.themes().iter().map(|t| t.shape.corners).collect();
        install_switcher(cx, fresh);
        assert!(!corners_overridden(cx));
        reset_corners(cx);
        let now: Vec<Corners> = cx
            .dodai()
            .switcher()
            .themes()
            .iter()
            .map(|t| t.shape.corners)
            .collect();
        assert_eq!(now, own);
    });
}

#[gpui::test]
fn overrides_keep_the_active_theme(cx: &mut TestAppContext) {
    cx.update(|cx| {
        install_switcher(cx, dodai_core::switcher());
        select_theme(cx, 2);
        let id = cx.dodai().theme().id();
        set_corners(cx, Corners::ALL);
        set_glow_intensity(cx, 0.5);
        reset_corners(cx);
        assert_eq!(cx.dodai().theme().id(), id);
    });
}
