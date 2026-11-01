//! The instrument demos of the charts page (signal lab, sci-fi HUD).

use super::*;
use gpui::Div;

impl Showcase {
    /// The two instrument panels: the signal lab and the sci-fi HUD.
    pub(crate) fn instrument_demos(&self, me: &Entity<Self>, cx: &mut Context<Self>) -> (Div, Div) {
        let t = theme_of(cx);
        let gap = px(t.spacing.md);
        let me = me.clone();
        let row = || div().flex().flex_wrap().items_center().gap(gap);
        let trail_picker = || {
            let m = me.clone();
            row()
                .child(Label::new("Trail"))
                .child(
                    Segmented::new("inst-trail", ["Off", "Subtle", "Default", "Long"])
                        .selected(self.inst_trail)
                        .on_select(move |i, _, cx| {
                            m.update(cx, |s, cx| {
                                s.inst_trail = i;
                                cx.notify();
                            })
                        }),
                )
                .child(Muted::new(
                    "The faded afterimage that follows the live trace to the right.",
                ))
        };
        let trail = [0.0, 0.5, 1.0, 2.0][self.inst_trail.min(3)];
        let spec: Vec<f32> = (0..18)
            .map(|i| 0.35 + 0.6 * ((i as f32 * 0.9).sin().abs()))
            .collect();
        let hexes: Vec<f32> = (0..7 * 12)
            .map(|i| {
                let (r, c) = ((i / 12) as f32, (i % 12) as f32);
                (((c * 0.55 + r * 0.8).sin() * (r * 0.9 - c * 0.3).cos()) * 0.5 + 0.5).powf(1.4)
            })
            .collect();
        let blips = [
            (0.08, 0.7),
            (0.31, 0.45),
            (0.52, 0.85),
            (0.77, 0.3),
            (0.9, 0.6),
        ]
        .map(|(a, r)| Blip::new(a, r));
        // Contacts drift on their own; the beam is pointed at the first one.
        let now = dodai_gpui::motion::time();
        let tracked = [(0.08, 0.7), (0.31, 0.45), (0.52, 0.85), (0.77, 0.3)]
            .map(|(a, r)| Blip::new(a + 0.06 * (now * 0.4 + a * 9.0).sin(), r));
        let tracked_beam = tracked[0].bearing;
        // Trail only applies to the live traces of the signal lab.
        let shell = |trail: bool, body: Div| {
            let d = div().flex().flex_col().gap_6();
            let d = if trail { d.child(trail_picker()) } else { d };
            d.child(body)
        };
        let wrap = || row().justify_around().flex_wrap();
        let lab = shell(
            true,
            wrap()
                .child(
                    Oscilloscope::ecg(2, 240)
                        .trail(trail)
                        .live(true)
                        .size(320.0, 120.0)
                        .caption("Pulse · 72 bpm"),
                )
                .child(
                    DotMatrix::new([
                        0.3, 0.55, 0.8, 0.45, 0.95, 0.6, 0.35, 0.7, 0.5, 0.85, 0.4, 0.65,
                    ])
                    .width(300.0)
                    .caption("Dot matrix"),
                )
                .child(
                    DotMatrix::new([])
                        .stream(&self.live_b, 100.0)
                        .width(300.0)
                        .caption("Dot matrix · streamed"),
                )
                .child(
                    div()
                        .flex()
                        .items_end()
                        .gap_4()
                        .child(SignalBars::new(2))
                        .child(SignalBars::new(4).height(40.0))
                        .child(SignalBars::new(5).height(52.0).bars(6)),
                )
                .child(
                    Oscilloscope::wave(&[(3.0, 1.0), (7.0, 0.4)], 160)
                        .trail(trail)
                        .live(true)
                        .caption("CH1 · 2 ms/div"),
                )
                .child(
                    Oscilloscope::new([])
                        .stream(&self.live_c)
                        .caption("Live ECG · streamed"),
                )
                .child(
                    Spectrum::new(spec)
                        .live(true)
                        .size(300.0, 160.0)
                        .caption("Audio · 18 band"),
                ),
        );
        let hud = shell(
            false,
            wrap()
                .child(RadarScope::new(blips).size(220.0).caption("Proximity"))
                .child(
                    RadarScope::new(tracked)
                        .sweep(tracked_beam)
                        .size(220.0)
                        .caption("Tracked · data-driven"),
                )
                .child(
                    HexMap::new(12, hexes)
                        .width(340.0)
                        .live(true)
                        .caption("Grid load"),
                )
                .child(
                    RingStack::new([("CPU", 78.0), ("MEM", 54.0), ("NET", 31.0), ("GPU", 90.0)])
                        .size(160.0),
                ),
        );
        (lab, hud)
    }
}
