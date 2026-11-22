//! Controls must survive hostile inputs (NaN, ±inf, empty, inverted ranges) without
//! panicking, under every theme.

use dodai_ui_gpui::*;
use gpui::{div, prelude::*, px, size, Context, TestAppContext, Window};

struct Controls;

impl Render for Controls {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let bad = [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -5.0, 1e30, 0.0];
        let mut root = div().flex().flex_wrap().gap(px(8.0)).size_full();
        for (i, v) in bad.into_iter().enumerate() {
            root = root
                .child(ProgressBar::new(v))
                .child(Slider::new(("sl", i), v))
                .child(Knob::new(("kn", i), v))
                .child(CircularProgress::new(v))
                .child(RangeSlider::new(("rs", i), v, v));
        }
        root
            // empty / out-of-range structural inputs
            .child(Slider::new("sl-eq", 5.0).range(5.0, 5.0))
            .child(Slider::new("sl-inv", 5.0).range(9.0, 1.0))
            .child(Knob::new("kn-eq", 2.0).range(2.0, 2.0))
            .child(Pagination::new("pg0", 0, 0))
            .child(Pagination::new("pg1", 99, 3))
            .child(Stepper::new(["a", "b"], 9))
            .child(Stepper::new(Vec::<&str>::new(), 0))
            .child(Segmented::new("sg", Vec::<&str>::new()))
            .child(Carousel::new("car0"))
            .child(Carousel::new("car1").item("x").selected(7).visible(0))
            .child(Tabs::new("tabs"))
            .child(Accordion::new("acc").item("t", "b").expanded([5]))
            .child(Breadcrumb::new("bc", Vec::<&str>::new()))
            .child(AvatarGroup::new(Vec::<&str>::new()))
            .child(DataTable::new("dt0"))
            .child(
                DataTable::new("dt1")
                    .columns([Column::new("A")])
                    .rows([vec!["1", "2", "3"], vec![]]),
            )
            .child(Menu::new("m"))
            .child(TreeView::new("tv", vec![]))
    }
}

#[gpui::test]
fn controls_survive_degenerate_inputs(cx: &mut TestAppContext) {
    for theme in dodai_ui_core::switcher().themes().to_vec() {
        cx.update(|cx| install(cx, theme));
        let (_, vcx) = cx.add_window_view(|_, _| Controls);
        vcx.simulate_resize(size(px(1200.0), px(900.0)));
        vcx.run_until_parked();
    }
}
