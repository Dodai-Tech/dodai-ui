//! Instruments must survive hostile values (NaN, ±inf, huge, inverted ranges, empty data)
//! without panicking, under a chamfered and a rounded theme.

use dodai_gpui::install;
use dodai_gpui_instruments::*;
use gpui::{div, prelude::*, px, size, AnyElement, Context, TestAppContext, Window};

const BAD: [f32; 7] = [
    f32::NAN,
    f32::INFINITY,
    f32::NEG_INFINITY,
    -5.0,
    1e30,
    0.0,
    3.0,
];

fn instruments(v: f32) -> Vec<AnyElement> {
    let mut out: Vec<AnyElement> = Vec::new();
    for st in GaugeStyle::ALL {
        out.push(Gauge::new(v).style(st).zones(true).into_any_element());
    }
    #[cfg(feature = "medical")]
    out.push(VitalsTile::new("HR", v).into_any_element());
    out.extend([
        Gauge::new(v).range(v, 0.0).into_any_element(),
        DialGauge::new(Scale::new(v, 10.0), v).into_any_element(),
        DialGauge::new(Scale::new(5.0, 5.0), v).into_any_element(),
        Annunciator::new([("A", Severity::Caution, false)]).into_any_element(),
        Annunciator::new(Vec::<(&str, Severity, bool)>::new()).into_any_element(),
        SegmentDisplay::new(format!("{v}")).into_any_element(),
        Oscilloscope::new([v, -v, 0.0]).into_any_element(),
        Oscilloscope::new([]).into_any_element(),
        Spectrum::new([v, 0.5]).into_any_element(),
        DotMatrix::new([v]).into_any_element(),
        SignalBars::new(usize::MAX).into_any_element(),
        HexMap::new(0, [v]).into_any_element(),
        HexMap::new(3, []).into_any_element(),
        RingStack::new([("a", v)]).into_any_element(),
        GradientDial::new(Scale::new(v, 10.0), v)
            .ramp(Ramp::Spectrum)
            .into_any_element(),
        GradientDial::new(Scale::new(5.0, 5.0), v).into_any_element(),
        GradientRing::new(v).ramp(Ramp::Heat).into_any_element(),
        GradientBar::new(Scale::new(-v, v), v)
            .cells(usize::MAX)
            .into_any_element(),
        GradientBar::new(Scale::new(0.0, 1.0), v)
            .width(f32::NAN)
            .into_any_element(),
    ]);
    // The dash pieces come with the `automotive` feature.
    #[cfg(feature = "automotive")]
    out.extend([
        RingTach::new(Scale::new(v, 8.0), v, v)
            .marker(v)
            .fuel(v, "x")
            .into_any_element(),
        SidePod::new(Scale::new(0.0, v), v)
            .icon(dodai_gpui::icons::Symbol::Fuel)
            .into_any_element(),
        MenuRing::new(["a", "b"])
            .selected(usize::MAX)
            .into_any_element(),
        LimitSign::new(u32::MAX).into_any_element(),
        TyrePod::new()
            .bars([Some(v), None, Some(-v), Some(2.0)])
            .low_below(v)
            .size(v)
            .into_any_element(),
        RowsPod::new([
            PodRow::new(dodai_gpui::icons::Symbol::Fuel, "x").bar(v, Tone::Info),
            PodRow::new(dodai_gpui::icons::Symbol::Fuel, "").span(),
        ])
        .size(v)
        .into_any_element(),
        RowsPod::new(Vec::<PodRow>::new()).into_any_element(),
        NoticePod::new(dodai_gpui::icons::Symbol::Washer, "")
            .size(v)
            .into_any_element(),
        GForcePod::new(v, -v).limit(v).size(v).into_any_element(),
        BoostPod::new(v).size(v).into_any_element(),
        MediaPod::new("").size(v).into_any_element(),
        CompassPod::new(v).size(v).into_any_element(),
        DualGauge::new(
            DualScale::new(Scale::new(v, 6.0), v, "bar"),
            DualScale::new(Scale::new(5.0, 5.0), -v, ""),
            dodai_gpui::icons::Symbol::OilCan,
        )
        .size(v)
        .into_any_element(),
        DashQuad::new(
            RingTach::new(Scale::new(0.0, 8.0), v, v),
            [BoostPod::new(v).into(), GForcePod::new(v, v).into()],
            [CompassPod::new(v).into(), TyrePod::new().into()],
        )
        .width(v)
        .into_any_element(),
        DashTriptych::new(
            RingTach::new(Scale::new(0.0, 8.0), v, v),
            SidePod::new(Scale::new(0.0, 8.0), v),
            MenuRing::new(Vec::<&str>::new()),
        )
        .into_any_element(),
    ]);
    out
}

struct Page;

impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let mut root = div().flex().flex_wrap().gap(px(8.0)).size_full();
        for v in BAD {
            for c in instruments(v) {
                root = root.child(div().w(px(220.0)).h(px(180.0)).child(c));
            }
        }
        root
    }
}

#[gpui::test]
fn instruments_survive_degenerate_values(cx: &mut TestAppContext) {
    for id in ["cyberpunk", "material"] {
        let theme = dodai_core::switcher()
            .themes()
            .iter()
            .find(|t| t.id() == id)
            .cloned()
            .expect("built-in theme");
        cx.update(|cx| install(cx, theme.clone()));
        let (_, vcx) = cx.add_window_view(|_, _| Page);
        vcx.simulate_resize(size(px(1600.0), px(1200.0)));
        vcx.run_until_parked();
    }
}
