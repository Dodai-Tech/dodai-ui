//! [`EnergyPod`]: the battery of an electric car, its charge and the power flowing out (driving)
//! or in (regeneration and charging), with the range left.

use super::*;

/// The battery of an electric car: the charge, the power flowing out or back in, and the range.
///
/// Sport draws a battery of cells under the charge, with a power arc that grows right from the
/// top while driving and left while recovering or charging. Every other look shows the same as
/// three rows of a [`RowsPod`] in that look: charge, power and range.
#[derive(IntoElement)]
pub struct EnergyPod {
    follow: Follow,
    charge: f32,
    power: f32,
    max_power: f32,
    max_regen: f32,
    range: SharedString,
    charging: bool,
    look: Look,
    style: PodStyle,
    size: f32,
}

/// Where `power` kW sits on a centre-zero scale: `0..=1` to the right while driving (of
/// `max_power`), `-1..=0` to the left while recovering (of `max_regen`). Pure.
pub fn power_frac(power: f32, max_power: f32, max_regen: f32) -> f32 {
    let p = pm::finite(power, 0.0);
    if p >= 0.0 {
        (p / max_power.max(1e-3)).min(1.0)
    } else {
        (p / max_regen.max(1e-3)).max(-1.0)
    }
}

/// The tone of a charge in percent: danger under 10, warning under 20, success otherwise. Pure.
pub fn charge_tone(percent: f32) -> Tone {
    match pm::percent(percent) {
        p if p < 10.0 => Tone::Danger,
        p if p < 20.0 => Tone::Warning,
        _ => Tone::Success,
    }
}

/// Power as signed text: `+42 kW` while driving, `-18 kW` while recovering, `0 kW` at rest.
/// Pure.
pub fn power_text(power: f32) -> String {
    let p = pm::finite(power, 0.0).round();
    if p > 0.0 {
        format!("+{p:.0} kW")
    } else if p < 0.0 {
        format!("-{:.0} kW", -p)
    } else {
        "0 kW".to_string()
    }
}

impl EnergyPod {
    follow_api!();

    /// A battery at `charge` percent (0 to 100), no power flowing.
    #[track_caller]
    pub fn new(charge: f32) -> Self {
        Self {
            follow: Follow::here(),
            charge: pm::percent(charge),
            power: 0.0,
            max_power: 150.0,
            max_regen: 60.0,
            range: SharedString::default(),
            charging: false,
            look: Look::Sport,
            style: PodStyle::Ring,
            size: 300.0,
        }
    }
    /// The power in kW: positive while driving, negative while recovering or charging.
    pub fn power(mut self, kw: f32) -> Self {
        self.power = pm::finite(kw, 0.0);
        self
    }
    /// The ends of the power arc in kW: the most the motor draws and the most it recovers
    /// (default 150 and 60).
    pub fn limits(mut self, max_power: f32, max_regen: f32) -> Self {
        self.max_power = pm::finite(max_power, 150.0).abs().max(1.0);
        self.max_regen = pm::finite(max_regen, 60.0).abs().max(1.0);
        self
    }
    /// The range left, e.g. `312 km`.
    pub fn range(mut self, r: impl Into<SharedString>) -> Self {
        self.range = r.into();
        self
    }
    /// Plugged in and charging: the heading says so and the power counts as flowing in.
    pub fn charging(mut self, on: bool) -> Self {
        self.charging = on;
        self
    }
    /// The look (see [`Look`]); every look but [`Look::Sport`] draws the battery as three rows
    /// of a [`RowsPod`].
    pub fn look(mut self, look: Look) -> Self {
        self.look = look;
        self
    }
    /// How the frame is drawn (open ring, closed disc, glow or bezel).
    pub fn style(mut self, style: PodStyle) -> Self {
        self.style = style;
        self
    }
    /// Diameter in px.
    pub fn size(mut self, s: f32) -> Self {
        self.size = pod_size(s);
        self
    }

    fn heading(&self) -> &'static str {
        if self.charging {
            "Charging"
        } else {
            "Energy"
        }
    }
}

impl RenderOnce for EnergyPod {
    fn render(mut self, w: &mut Window, cx: &mut App) -> impl IntoElement {
        self.charge = pm::percent(follow(w, cx, &self.follow, 0, self.charge));
        self.power = pm::finite(follow(w, cx, &self.follow, 1, self.power), 0.0);
        let flow = power_frac(self.power, self.max_power, self.max_regen);
        let tone = charge_tone(self.charge);
        if self.look != Look::Sport {
            let flow_tone = if flow < 0.0 {
                Tone::Success
            } else {
                Tone::Info
            };
            let mut rows = vec![
                PodRow::new(Symbol::TractionBattery, format!("{:.0} %", self.charge))
                    .bar(self.charge / 100.0, tone),
                PodRow::new(
                    if self.charging {
                        Symbol::Plug
                    } else if flow < 0.0 {
                        Symbol::Recuperation
                    } else {
                        Symbol::Hybrid
                    },
                    power_text(self.power),
                )
                .bar(flow.abs(), flow_tone),
            ];
            if !self.range.is_empty() {
                rows.push(PodRow::new(Symbol::Range, self.range.clone()));
            }
            return RowsPod::new(rows)
                .heading(self.heading())
                .look(self.look)
                .style(self.style)
                .size(self.size)
                .into_any_element();
        }
        let t = theme_of(cx);
        let size = self.size;
        let c = size / 2.0;
        let (ink, muted) = (hsla(t.palette.text), hsla(t.palette.text_muted));
        let level = tone.color(&t);
        let flow_col = hsla(if flow < 0.0 {
            t.palette.success
        } else {
            t.palette.info
        });
        let (bw, bh) = (size * 0.4, size * 0.16);
        let (bx, by) = (c - bw / 2.0, c - bh / 2.0);
        let cells = 10;
        let lit = pm::lit_cells(self.charge, cells);
        let arc_r = c - size * 0.12;
        let reach = 0.3;
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            // The power arc: a track either side of twelve o'clock, lit from the top.
            stroke(
                w,
                o,
                &turn_arc(c, c, arc_r, -reach, reach),
                size * 0.018,
                scale_alpha(ink, 0.12),
                false,
            );
            if flow.abs() > 1e-3 {
                let (a, b) = if flow < 0.0 {
                    (reach * flow, 0.0)
                } else {
                    (0.0, reach * flow)
                };
                stroke(
                    w,
                    o,
                    &turn_arc(c, c, arc_r, a, b),
                    size * 0.018,
                    flow_col,
                    false,
                );
            }
            stroke(
                w,
                o,
                &[
                    polar(c, c, arc_r - size * 0.03, 0.0),
                    polar(c, c, arc_r + size * 0.03, 0.0),
                ],
                size * 0.006,
                ink,
                false,
            );
            // The battery: a body with its terminal and a row of cells.
            let body = [(bx, by), (bx + bw, by), (bx + bw, by + bh), (bx, by + bh)];
            stroke(w, o, &body, (size * 0.008).max(1.5), ink, true);
            let nub = [
                (bx + bw, by + bh * 0.3),
                (bx + bw + size * 0.02, by + bh * 0.3),
                (bx + bw + size * 0.02, by + bh * 0.7),
                (bx + bw, by + bh * 0.7),
            ];
            fill(w, o, &nub, ink);
            let pad = bh * 0.14;
            let cw = (bw - pad * 2.0) / cells as f32;
            for i in 0..cells {
                let x = bx + pad + cw * i as f32;
                let cell = [
                    (x + cw * 0.12, by + pad),
                    (x + cw * 0.88, by + pad),
                    (x + cw * 0.88, by + bh - pad),
                    (x + cw * 0.12, by + bh - pad),
                ];
                fill(
                    w,
                    o,
                    &cell,
                    if i < lit {
                        level
                    } else {
                        scale_alpha(ink, 0.1)
                    },
                );
            }
        };
        let foot_icon = if self.charging {
            Symbol::Plug
        } else {
            Symbol::Range
        };
        let foot = if self.charging {
            "Charging".into()
        } else {
            self.range.clone()
        };
        let content = div()
            .absolute()
            .left_0()
            .top_0()
            .size_full()
            .child(
                text_box(c, size * 0.2, size * 0.5)
                    .text_size(px(size * 0.042))
                    .text_color(flow_col)
                    .child(power_text(self.power)),
            )
            .child(
                text_box(c, by - size * 0.13, size * 0.5)
                    .text_size(px(size * 0.085))
                    .text_color(ink)
                    .child(format!("{:.0} %", self.charge)),
            )
            .child(
                text_box(c, by + bh + size * 0.07, size * 0.6)
                    .items_center()
                    .gap(px(size * 0.02))
                    .text_size(px(size * 0.05))
                    .text_color(if self.charging { level } else { muted })
                    .child(Icon::symbol(foot_icon).size(size * 0.06).color(muted))
                    .child(foot),
            );
        ring(
            size,
            [Some(Tone::Success), Some(Tone::Info)],
            self.style,
            &t,
            paint,
            content,
        )
        .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn power_sits_left_while_recovering_and_right_while_driving() {
        assert_eq!(power_frac(75.0, 150.0, 60.0), 0.5);
        assert_eq!(power_frac(-30.0, 150.0, 60.0), -0.5);
        assert_eq!(power_frac(900.0, 150.0, 60.0), 1.0);
        assert_eq!(power_frac(-900.0, 150.0, 60.0), -1.0);
        assert_eq!(power_frac(f32::NAN, 150.0, 60.0), 0.0);
        assert!(power_frac(10.0, 0.0, 0.0).is_finite());
    }

    #[test]
    fn a_low_battery_warns_then_alarms() {
        assert_eq!(charge_tone(80.0), Tone::Success);
        assert_eq!(charge_tone(15.0), Tone::Warning);
        assert_eq!(charge_tone(5.0), Tone::Danger);
        assert_eq!(charge_tone(f32::NAN), Tone::Danger);
    }

    #[test]
    fn power_reads_with_its_sign() {
        assert_eq!(power_text(42.4), "+42 kW");
        assert_eq!(power_text(-18.0), "-18 kW");
        assert_eq!(power_text(0.2), "0 kW");
        assert_eq!(power_text(f32::NAN), "0 kW");
    }
}
