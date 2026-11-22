//! Density follows every design: control height, spacing and the control-sized `Sizes` shrink
//! and grow together; icons and the touch target keep their size.
#![cfg(any(
    feature = "cyberpunk",
    feature = "material",
    feature = "neobrutalism",
    feature = "dodai"
))]

use dodai_ui_core::themes::all;

#[test]
fn every_design_follows_density() {
    for t in all() {
        let id = t.id();
        let (dense, roomy) = (t.clone().with_density(0.8), t.clone().with_density(1.25));
        let (base, small, big) = (t.sizes(), dense.sizes(), roomy.sizes());
        assert!(
            dense.shape.control_height < t.shape.control_height
                && roomy.shape.control_height > t.shape.control_height,
            "{id}: control height"
        );
        assert!(
            dense.spacing.md < t.spacing.md && roomy.spacing.md > t.spacing.md,
            "{id}: spacing"
        );
        assert!(
            small.segmented_height < base.segmented_height
                && big.segmented_height > base.segmented_height,
            "{id}: segmented height"
        );
        assert!(
            small.selection_box < base.selection_box && big.selection_box > base.selection_box,
            "{id}: selection box"
        );
        assert!(
            small.switch_track.0 < base.switch_track.0 && big.switch_track.0 > base.switch_track.0,
            "{id}: switch track"
        );
        assert!(
            small.slider_thumb < base.slider_thumb && big.slider_thumb > base.slider_thumb,
            "{id}: slider thumb"
        );
        assert_eq!(small.icon, base.icon, "{id}: icons keep their size");
        assert_eq!(small.touch_target, base.touch_target, "{id}: touch target");
        assert_eq!(t.clone().with_density(1.0).sizes(), base, "{id}: 1.0");
    }
}

#[test]
fn every_design_scales_button_sizes_with_density() {
    use dodai_ui_core::ControlSize;
    for t in all() {
        let (dense, roomy) = (t.clone().with_density(0.8), t.clone().with_density(1.25));
        for size in ControlSize::ALL {
            let (a, b, c) = (
                dense.control_metrics(size),
                t.control_metrics(size),
                roomy.control_metrics(size),
            );
            assert!(
                a.height < b.height && b.height < c.height,
                "{} {size:?}: height",
                t.id()
            );
            assert!(a.padding < c.padding, "{} {size:?}: padding", t.id());
        }
    }
}
