//! Gear displays in every [`Look`]: the P R N D selector as a row, a stack or a single large
//! figure, the manual gear, and a prompt to shift up or down. Every colour comes from the theme.

use crate::gpui::look::{resolve, Look};
use crate::gpui::segment::SegmentDisplay;
use crate::gpui::shapes::Ink;
use dodai_core::Theme;
use dodai_gpui::paint::{fill, scale_alpha, stroke};
use dodai_gpui::text::first_family;
use dodai_gpui::theme_of;
use gpui::prelude::FluentBuilder;
use gpui::{
    canvas, div, px, AnyElement, App, Div, FontWeight, IntoElement, ParentElement, RenderOnce,
    SharedString, Styled, Window,
};

/// The positions of the selector, in the order the row and the stack show them.
const SLOTS: [&str; 4] = ["P", "R", "N", "D"];

/// Where the gearbox is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Gear {
    /// Park.
    #[default]
    Park,
    /// Reverse.
    Reverse,
    /// Neutral.
    Neutral,
    /// Drive.
    Drive,
    /// Shifted by hand, in this gear (1 and up).
    Manual(u8),
}

impl Gear {
    /// The automatic positions, in selector order.
    pub const AUTOMATIC: [Self; 4] = [Self::Park, Self::Reverse, Self::Neutral, Self::Drive];

    /// The selector position lit for this gear: a manual gear lights the drive position. Pure.
    pub fn slot(self) -> usize {
        match self {
            Self::Park => 0,
            Self::Reverse => 1,
            Self::Neutral => 2,
            Self::Drive | Self::Manual(_) => 3,
        }
    }

    /// The large figure: the letter, or the number of a manual gear. Pure.
    pub fn figure(self) -> SharedString {
        match self {
            Self::Manual(n) => n.to_string().into(),
            g => SLOTS[g.slot()].into(),
        }
    }

    /// The label at the lit position: the letter, or `M` and the gear when shifted by hand.
    /// Pure.
    pub fn label(self) -> SharedString {
        match self {
            Self::Manual(n) => format!("M{n}").into(),
            g => SLOTS[g.slot()].into(),
        }
    }

    /// The word under the large figure. Pure.
    pub fn caption(self) -> &'static str {
        match self {
            Self::Park => "Park",
            Self::Reverse => "Reverse",
            Self::Neutral => "Neutral",
            Self::Drive => "Drive",
            Self::Manual(_) => "Manual",
        }
    }
}

/// Which way the driver should shift next.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ShiftHint {
    /// No prompt.
    #[default]
    None,
    /// Shift up: the engine is near its best change point.
    Up,
    /// Shift down: the engine is labouring.
    Down,
}

/// How the selector is arranged.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GearLayout {
    /// P R N D side by side.
    #[default]
    Row,
    /// P R N D one above the other.
    Stack,
    /// The engaged gear alone, large, with its name under it.
    Single,
}

impl GearLayout {
    /// Every layout, in showcase order.
    pub const ALL: [Self; 3] = [Self::Row, Self::Stack, Self::Single];

    /// A short name for a caption or a picker.
    pub fn name(self) -> &'static str {
        match self {
            Self::Row => "Row",
            Self::Stack => "Stack",
            Self::Single => "Single",
        }
    }
}

/// The points of a shift chevron `s` high centred on `(x, y)`, pointing up or down: a closed
/// triangle when `solid`, else an open caret. Pure.
pub fn chevron((x, y): (f32, f32), s: f32, up: bool, solid: bool) -> Vec<(f32, f32)> {
    let h = s * 0.5;
    let (tip, base) = if up { (y - h, y + h) } else { (y + h, y - h) };
    let half = s * if solid { 0.55 } else { 0.6 };
    vec![(x - half, base), (x, tip), (x + half, base)]
}

/// A gear selector drawn in a [`Look`]: P R N D with the engaged position lit, a manual gear,
/// and an optional prompt to shift up or down.
#[derive(IntoElement)]
pub struct GearSelector {
    gear: Gear,
    layout: GearLayout,
    hint: ShiftHint,
    size: f32,
    /// The look drawn: the given one, else the design's, settled when rendered.
    look: Look,
    /// The look given with `.look(..)`; without one, the design's own.
    given_look: Option<Look>,
}

impl GearSelector {
    /// A selector in `gear`, as a row 20 px high, without a prompt.
    pub fn new(gear: Gear) -> Self {
        Self {
            gear,
            layout: GearLayout::Row,
            hint: ShiftHint::None,
            size: 20.0,
            look: Look::Sport,
            given_look: None,
        }
    }

    /// Arrange it as a row, a stack or a single figure.
    pub fn layout(mut self, layout: GearLayout) -> Self {
        self.layout = layout;
        self
    }

    /// Prompt to shift up or down.
    pub fn hint(mut self, hint: ShiftHint) -> Self {
        self.hint = hint;
        self
    }

    /// Letter height in px; the single figure is about three times as tall.
    pub fn size(mut self, size: f32) -> Self {
        self.size = size.max(8.0);
        self
    }

    /// Draw it in `look`; by default the design's own.
    pub fn look(mut self, look: Look) -> Self {
        self.given_look = Some(look);
        self
    }

    /// One selector position, lit or not.
    fn slot(&self, t: &Theme, ink: Ink, label: SharedString, on: bool) -> AnyElement {
        let s = self.size;
        let display = first_family(t.typography.display);
        let mono = first_family(t.typography.mono);
        let cell = div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .min_w(px(s * 1.4))
            .h(px(s * 1.6))
            .px(px(s * 0.2))
            .text_size(px(s))
            .line_height(px(s * 1.1));
        let off = scale_alpha(ink.muted, 0.55);
        match self.look {
            Look::Sport => cell
                .font_family(display)
                .font_weight(if on {
                    FontWeight::SEMIBOLD
                } else {
                    FontWeight::NORMAL
                })
                .text_color(if on { ink.text } else { off })
                .child(label)
                .child(
                    div()
                        .mt(px(s * 0.12))
                        .w(px(s * 0.9))
                        .h(px((s * 0.08).max(1.5)))
                        .rounded(px(s))
                        .when(on, |d| d.bg(ink.primary)),
                )
                .into_any_element(),
            Look::Chronograph => cell
                .font_family(display)
                .font_weight(FontWeight::LIGHT)
                .text_color(if on { ink.text } else { off })
                .when(on, |d| {
                    d.rounded(px(s))
                        .border_1()
                        .border_color(scale_alpha(ink.text, 0.6))
                        .bg(ink.face)
                })
                .child(label)
                .into_any_element(),
            Look::Vintage => cell
                .font_family(display)
                .font_weight(FontWeight::MEDIUM)
                .text_color(if on { ink.warn } else { off })
                .child(label)
                .child(
                    div()
                        .mt(px(s * 0.1))
                        .size(px((s * 0.18).max(3.0)))
                        .rounded(px(s))
                        .when(on, |d| d.bg(ink.warn)),
                )
                .into_any_element(),
            Look::HeadsUp => cell
                .font_family(mono)
                .text_color(if on {
                    ink.primary
                } else {
                    scale_alpha(ink.primary, 0.35)
                })
                .when(on, |d| {
                    d.border_l_1()
                        .border_r_1()
                        .border_color(ink.primary)
                        .bg(scale_alpha(ink.primary, 0.08))
                })
                .child(label)
                .into_any_element(),
            Look::Segment => {
                let txt: String = label.chars().filter(|c| *c != 'M').collect();
                cell.child(
                    SegmentDisplay::new(txt)
                        .height(s * 0.95)
                        .color(if on {
                            ink.text
                        } else {
                            scale_alpha(ink.text, 0.22)
                        })
                        .ghost(false),
                )
                .into_any_element()
            }
            Look::Halo => cell
                .font_family(display)
                .font_weight(FontWeight::LIGHT)
                .text_color(if on { ink.text } else { off })
                .rounded(px(s))
                .when(on, |d| {
                    d.bg(scale_alpha(ink.primary, 0.16))
                        .border_1()
                        .border_color(scale_alpha(ink.primary, 0.5))
                })
                .child(label)
                .into_any_element(),
            Look::GlassCockpit => cell
                .font_family(mono)
                .text_color(if on { ink.info } else { off })
                .when(on, |d| d.border_1().border_color(ink.text))
                .child(label)
                .into_any_element(),
            Look::Race => cell
                .font_family(display)
                .font_weight(FontWeight::BOLD)
                .italic()
                .text_color(if on { ink.on_primary } else { off })
                .when(on, |d| d.bg(ink.primary))
                .child(label)
                .into_any_element(),
            Look::Line => cell
                .font_family(display)
                .font_weight(FontWeight::LIGHT)
                .text_color(if on {
                    ink.text
                } else {
                    scale_alpha(ink.muted, 0.4)
                })
                .child(label)
                .child(
                    div()
                        .mt(px(s * 0.14))
                        .size(px((s * 0.16).max(3.0)))
                        .rounded(px(s))
                        .when(on, |d| d.bg(ink.primary)),
                )
                .into_any_element(),
        }
    }

    /// The panel every position sits on: the look's housing.
    fn housing(&self, t: &Theme, ink: Ink) -> Div {
        let s = self.size;
        let d = div().flex().items_center().gap(px(s * 0.25)).p(px(s * 0.3));
        match self.look {
            Look::Chronograph => d
                .rounded(px(s))
                .border_1()
                .border_color(scale_alpha(ink.text, 0.25)),
            Look::Vintage => d
                .rounded(px(s * 0.2))
                .bg(scale_alpha(ink.face, 0.9))
                .border_1()
                .border_color(ink.line),
            Look::HeadsUp => d
                .border_t_1()
                .border_b_1()
                .border_color(scale_alpha(ink.primary, 0.3)),
            Look::Segment => d
                .rounded(px(t.shape.radius))
                .bg(scale_alpha(ink.ground, 0.85))
                .border_1()
                .border_color(ink.line),
            Look::Halo => d
                .rounded(px(s * 2.0))
                .bg(scale_alpha(ink.face, 0.5))
                .border_1()
                .border_color(scale_alpha(ink.primary, 0.2)),
            Look::GlassCockpit => d.bg(ink.ground).border_1().border_color(ink.line),
            Look::Race => d.bg(ink.ground).border_l_4().border_color(ink.primary),
            Look::Sport | Look::Line => d,
        }
    }

    /// The shift prompt, `s` high: a solid triangle, or a hairline caret in the quiet looks.
    fn prompt(&self, ink: Ink, s: f32) -> Option<AnyElement> {
        let up = match self.hint {
            ShiftHint::None => return None,
            ShiftHint::Up => true,
            ShiftHint::Down => false,
        };
        let solid = !matches!(self.look, Look::Line | Look::Chronograph | Look::Vintage);
        let col = match self.look {
            Look::Vintage => ink.warn,
            Look::GlassCockpit => ink.info,
            _ => ink.primary,
        };
        let lw = (s * 0.08).max(1.0);
        Some(
            canvas(
                |_, _, _| {},
                move |b, _, w, _| {
                    let o = b.origin;
                    let pts = chevron((s / 2.0, s / 2.0), s * 0.6, up, solid);
                    if solid {
                        fill(w, o, &pts, col);
                    } else {
                        stroke(w, o, &pts, lw, col, false);
                    }
                },
            )
            .size(px(s))
            .into_any_element(),
        )
    }

    /// The engaged gear alone: a large figure, its name and the prompt beside it.
    fn single(&self, t: &Theme, ink: Ink) -> AnyElement {
        let s = self.size * 3.0;
        let display = first_family(t.typography.display);
        let figure: AnyElement = match self.look {
            Look::Segment => SegmentDisplay::new(self.gear.figure().to_string())
                .height(s * 0.9)
                .color(ink.text)
                .into_any_element(),
            look => div()
                .font_family(if look == Look::GlassCockpit {
                    first_family(t.typography.mono)
                } else {
                    display
                })
                .text_size(px(s))
                .line_height(px(s * 1.05))
                .font_weight(match look {
                    Look::Race => FontWeight::BOLD,
                    Look::Sport | Look::Vintage => FontWeight::MEDIUM,
                    _ => FontWeight::LIGHT,
                })
                .when(look == Look::Race, |d| d.italic())
                .text_color(match look {
                    Look::HeadsUp => ink.primary,
                    Look::GlassCockpit => ink.info,
                    Look::Vintage => ink.warn,
                    _ => ink.text,
                })
                .child(self.gear.figure())
                .into_any_element(),
        };
        let caption = div()
            .text_size(px(t.typography.size_xs))
            .text_color(ink.muted)
            .child(self.gear.caption().to_uppercase());
        let hint = self.prompt(ink, self.size * 1.1);
        self.housing(t, ink)
            .gap(px(self.size * 0.5))
            .px(px(self.size * 0.8))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .child(figure)
                    .child(caption),
            )
            .children(hint)
            .into_any_element()
    }
}

impl RenderOnce for GearSelector {
    fn render(mut self, _w: &mut Window, cx: &mut App) -> impl IntoElement {
        self.look = resolve(self.given_look, cx);
        let t = theme_of(cx);
        let ink = Ink::of(&t);
        if self.layout == GearLayout::Single {
            return self.single(&t, ink);
        }
        let lit = self.gear.slot();
        let slots: Vec<AnyElement> = SLOTS
            .iter()
            .enumerate()
            .map(|(i, l)| {
                let label = if i == lit {
                    self.gear.label()
                } else {
                    SharedString::from(*l)
                };
                self.slot(&t, ink, label, i == lit)
            })
            .collect();
        let hint = self.prompt(ink, self.size);
        let housing = self.housing(&t, ink);
        let housing = if self.layout == GearLayout::Stack {
            housing.flex_col()
        } else {
            housing
        };
        housing.children(slots).children(hint).into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_manual_gear_lights_drive_and_shows_its_number() {
        assert_eq!(Gear::Manual(4).slot(), Gear::Drive.slot());
        assert_eq!(Gear::Manual(4).figure().as_ref(), "4");
        assert_eq!(Gear::Manual(4).label().as_ref(), "M4");
        assert_eq!(Gear::Reverse.figure().as_ref(), "R");
        for (i, g) in Gear::AUTOMATIC.into_iter().enumerate() {
            assert_eq!(g.slot(), i);
            assert_eq!(g.label(), g.figure());
        }
    }

    #[test]
    fn every_gear_and_layout_has_a_name() {
        assert_eq!(Gear::Manual(1).caption(), "Manual");
        let names: Vec<_> = GearLayout::ALL.iter().map(|l| l.name()).collect();
        assert_eq!(names, ["Row", "Stack", "Single"]);
    }

    #[test]
    fn the_chevron_points_the_way_to_shift() {
        let up = chevron((10.0, 10.0), 8.0, true, true);
        assert!(up[1].1 < up[0].1, "up: the tip is above the base");
        let down = chevron((10.0, 10.0), 8.0, false, true);
        assert!(down[1].1 > down[0].1, "down: the tip is below the base");
        assert_eq!(up[0].0, 10.0 - 8.0 * 0.55);
        assert_eq!(chevron((0.0, 0.0), 8.0, true, false)[2].0, 8.0 * 0.6);
    }
}
