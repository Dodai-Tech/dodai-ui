//! The second reading of a twin pod outside the sport look: one strip under the main face, its
//! caption on the left and its figure on the right, drawn in the look's own style. Every colour
//! comes from the theme.

use super::detailed::layer;
use super::*;
use crate::gpui::race::{skew, LEAN};
use crate::gpui::segment::SegmentDisplay;
use crate::gpui::shapes::Ink;

/// How much of a twin pod's side the strip under its face takes, gap included.
pub(super) const STRIP: f32 = 0.17;

/// The severity of `d`'s value on its scale: where the value sits among its zones. Pure.
pub(super) fn severity(d: &DualScale) -> Severity {
    let spans: Vec<(f32, f32, Severity)> = d
        .scale
        .zone_spans()
        .into_iter()
        .filter(|z| z.2 != Severity::Normal)
        .collect();
    pm::severity_at(&spans, d.scale.fraction(d.value))
}

/// The caption of the strip: the reading's label, else its unit, in capitals. Pure.
pub(super) fn caption(d: &DualScale) -> SharedString {
    d.label
        .clone()
        .unwrap_or_else(|| d.unit.clone())
        .to_uppercase()
        .into()
}

/// The strip for `d`, `wd` by `h`, in `look`.
pub(super) fn strip(
    look: Look,
    d: &DualScale,
    wd: f32,
    h: f32,
    t: &dodai_core::Theme,
) -> AnyElement {
    let ink = Ink::of(t);
    let sev = severity(d);
    let alarm = (sev != Severity::Normal).then(|| severity_color(t, sev));
    let figure = readout(d.value, d.decimals, "");
    let unit = d.unit.clone();
    let cap = caption(d);
    let (fs, cs) = (h * 0.52, h * 0.3);
    let row = |cap_col: Hsla, fig: AnyElement, unit_col: Hsla| {
        div()
            .absolute()
            .size_full()
            .flex()
            .items_center()
            .justify_between()
            .px(px(h * 0.4))
            .child(
                div()
                    .text_size(px(cs))
                    .text_color(cap_col)
                    .child(cap.clone()),
            )
            .child(
                div()
                    .flex()
                    .items_baseline()
                    .gap(px(h * 0.12))
                    .child(fig)
                    .child(
                        div()
                            .text_size(px(cs))
                            .text_color(unit_col)
                            .child(unit.clone()),
                    ),
            )
    };
    let text_fig = |col: Hsla| {
        div()
            .text_size(px(fs))
            .text_color(col)
            .child(figure.clone())
            .into_any_element()
    };
    let base = div().relative().w(px(wd)).h(px(h));
    match look {
        // A recessed window, like the date on a watch.
        Look::Sport | Look::Chronograph | Look::Apex | Look::Daylight => base
            .rounded(px(h * 0.2))
            .bg(ink.face)
            .border_1()
            .border_color(ink.line)
            .child(row(
                ink.muted,
                text_fig(alarm.unwrap_or(ink.text)),
                ink.muted,
            )),
        // A small enamel plate with square corners.
        Look::Vintage => base
            .rounded(px(h * 0.06))
            .bg(scale_alpha(ink.text, 0.08))
            .border_1()
            .border_color(scale_alpha(ink.text, 0.3))
            .child(row(
                ink.muted,
                text_fig(alarm.unwrap_or(ink.text)),
                ink.muted,
            )),
        // Open space over a hairline, the figure in light type.
        Look::Line => base
            .border_t_1()
            .border_color(scale_alpha(ink.text, 0.2))
            .child(row(
                ink.muted,
                div()
                    .text_size(px(fs))
                    .font_weight(gpui::FontWeight::LIGHT)
                    .text_color(alarm.unwrap_or(ink.text))
                    .child(figure.clone())
                    .into_any_element(),
                ink.muted,
            )),
        // An open box drawn in the projector's colour.
        Look::HeadsUp => {
            let col = alarm.unwrap_or(ink.primary);
            base.border_1()
                .border_color(scale_alpha(col, 0.7))
                .child(row(col, text_fig(alarm.unwrap_or(ink.text)), col))
        }
        // A segment readout on the display glass.
        Look::Segment => {
            let col = alarm.unwrap_or(ink.primary);
            let seg = SegmentDisplay::new(figure.clone())
                .height(fs)
                .color(col)
                .into_any_element();
            base.rounded(px(h * 0.1))
                .bg(scale_alpha(ink.primary, 0.06))
                .border_1()
                .border_color(scale_alpha(ink.primary, 0.35))
                .child(row(ink.primary, seg, ink.muted))
        }
        // A pill of light.
        Look::Halo => {
            let col = alarm.unwrap_or(ink.primary);
            base.rounded(px(h / 2.0))
                .bg(scale_alpha(col, 0.1))
                .border_1()
                .border_color(scale_alpha(col, 0.5))
                .child(row(col, text_fig(alarm.unwrap_or(ink.text)), ink.muted))
        }
        // A boxed readout on the glass: cyan caption, green figure, inverse past an alarm.
        Look::GlassCockpit => {
            let invert = sev == Severity::Alarm;
            let (bg, fig) = match (invert, alarm) {
                (true, Some(c)) => (c, ink.ground),
                (_, Some(c)) => (ink.ground, c),
                _ => (ink.ground, ink.ok),
            };
            base.bg(bg)
                .border_1()
                .border_color(scale_alpha(ink.text, 0.6))
                .child(row(ink.info, text_fig(fig), ink.info))
        }
        // A slanted tag carrying the caption, the figure leaning beside it.
        Look::Race => {
            let tag_col = alarm.unwrap_or(ink.primary);
            let tw = wd * 0.5;
            let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
                fill(w, o, &skew(0.0, 0.0, tw, h), tag_col);
            };
            base.child(layer(paint)).child(
                div()
                    .absolute()
                    .size_full()
                    .flex()
                    .items_center()
                    .child(
                        div()
                            .w(px(tw))
                            .pl(px(h * LEAN + h * 0.2))
                            .text_size(px(cs))
                            .font_weight(gpui::FontWeight::BOLD)
                            .italic()
                            .text_color(ink.on_primary)
                            .child(cap.clone()),
                    )
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .justify_end()
                            .items_baseline()
                            .gap(px(h * 0.12))
                            .child(
                                div()
                                    .text_size(px(h * 0.8))
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .italic()
                                    .text_color(alarm.unwrap_or(ink.text))
                                    .child(figure.clone()),
                            )
                            .child(
                                div()
                                    .text_size(px(cs))
                                    .text_color(ink.muted)
                                    .child(unit.clone()),
                            ),
                    ),
            )
        }
    }
    .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_strip_reads_the_zone_its_value_is_in() {
        let scale = Scale::new(60.0, 150.0).zone(130.0, 150.0, Severity::Caution);
        assert_eq!(
            severity(&DualScale::new(scale.clone(), 90.0, "°C")),
            Severity::Normal
        );
        assert_eq!(
            severity(&DualScale::new(scale, 140.0, "°C")),
            Severity::Caution
        );
    }

    #[test]
    fn the_caption_falls_back_to_the_unit() {
        let d = DualScale::new(Scale::new(0.0, 6.0), 3.0, "bar");
        assert_eq!(caption(&d), "BAR");
        assert_eq!(caption(&d.label("Pressure")), "PRESSURE");
    }
}
