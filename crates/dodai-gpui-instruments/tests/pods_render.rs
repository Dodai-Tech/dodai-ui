//! Render tests for the dash pods: every pod, in every look, under every built-in theme, lays
//! out to a non-empty box of the size it was asked for.
#![cfg(feature = "automotive")]

use dodai_gpui::icons::Symbol;
use dodai_gpui_instruments::*;
use gpui::{div, prelude::*, px, size, AnyElement, Context, TestAppContext, Window};

const SIZE: f32 = 200.0;

struct Host;

impl Render for Host {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let mut col = div().flex().flex_col().gap(px(4.0));
        for (name, el, _) in pods() {
            let name: gpui::SharedString = name.into();
            col = col.child(
                div()
                    .flex()
                    .child(div().debug_selector(move || name.to_string()).child(el)),
            );
        }
        col
    }
}

/// `(name, pod, expected width)` for every pod and look.
fn pods() -> Vec<(String, AnyElement, f32)> {
    let mut out: Vec<(String, AnyElement, f32)> = Vec::new();
    for look in PodLook::ALL {
        // Every look is a square pod of the size asked for.
        let w = SIZE;
        let n = |s: &str| format!("{s}-{}", look.name());
        out.push((
            n("tyre"),
            TyrePod::new()
                .bars([Some(2.2), Some(2.3), Some(2.1), Some(1.4)])
                .look(look)
                .size(SIZE)
                .into_any_element(),
            w,
        ));
        out.push((
            n("gforce"),
            GForcePod::new(0.4, -0.2)
                .look(look)
                .size(SIZE)
                .into_any_element(),
            w,
        ));
        out.push((
            n("compass"),
            CompassPod::new(300.0)
                .look(look)
                .size(SIZE)
                .into_any_element(),
            w,
        ));
        out.push((
            n("media"),
            MediaPod::new("Title")
                .look(look)
                .size(SIZE)
                .into_any_element(),
            w,
        ));
        out.push((
            n("notice"),
            NoticePod::new(Symbol::Washer, "Washer fluid")
                .look(look)
                .size(SIZE)
                .into_any_element(),
            w,
        ));
        out.push((
            n("rows"),
            RowsPod::new([PodRow::new(Symbol::Fuel, "40 l").bar(0.5, Tone::Info)])
                .look(look)
                .size(SIZE)
                .into_any_element(),
            w,
        ));
    }
    out.push((
        "boost".into(),
        BoostPod::new(60.0).size(SIZE).into_any_element(),
        SIZE,
    ));
    out.push((
        "dual".into(),
        DualGauge::new(
            DualScale::new(Scale::new(0.0, 6.0), 3.0, "bar"),
            DualScale::new(Scale::new(60.0, 150.0), 90.0, "°C"),
            Symbol::OilCan,
        )
        .size(SIZE)
        .into_any_element(),
        SIZE,
    ));
    out.push((
        "side".into(),
        SidePod::new(Scale::new(0.0, 10.0), 4.0)
            .size(SIZE)
            .into_any_element(),
        SIZE,
    ));
    for look in GaugeLook::ALL {
        out.push((
            format!("face-{}", look.name()),
            GaugeFace::new(Scale::new(0.0, 8.0), 3.2)
                .look(look)
                .size(SIZE)
                .into_any_element(),
            f32::NAN,
        ));
    }
    out
}

#[gpui::test]
fn every_pod_lays_out_in_every_theme(cx: &mut TestAppContext) {
    let themes = dodai_core::switcher().themes().to_vec();
    assert!(themes.len() > 4, "the built-in themes");
    for theme in themes {
        cx.update(|cx| dodai_gpui::install(cx, theme.clone()));
        let spec: Vec<(String, f32)> = pods().into_iter().map(|(n, _, w)| (n, w)).collect();
        let (_, vcx) = cx.add_window_view(|_, _| Host);
        vcx.simulate_resize(size(px(1200.0), px(6000.0)));
        vcx.run_until_parked();
        for (name, width) in spec {
            let key: &'static str = Box::leak(name.clone().into_boxed_str());
            let b = vcx
                .debug_bounds(key)
                .unwrap_or_else(|| panic!("{name} under {}: not laid out", theme.name()));
            let (w, h) = (f32::from(b.size.width), f32::from(b.size.height));
            assert!(
                w > 1.0 && h > 1.0 && w.is_finite() && h.is_finite(),
                "{name} under {}: {w} x {h}",
                theme.name()
            );
            if width.is_finite() {
                assert!(
                    (w - width).abs() < 1.5,
                    "{name} under {}: width {w}, wanted {width}",
                    theme.name()
                );
            }
        }
    }
}
