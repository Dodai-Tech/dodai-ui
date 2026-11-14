//! Token-styled boxes (cards, alerts, chips, avatars, code, tooltips, rows, steps, skeletons,
//! popups, the closed dropdown), the busy indicators and sized buttons.

use dodai_core::{busy, ButtonVariant, ControlSize, ControlState, Kit, KitStyle, ShapeKind};
use iced::{
    alignment, mouse,
    widget::{
        button,
        canvas::{self, path::Arc, Canvas, Geometry, Path, Stroke},
        container, row, text,
    },
    Background, Border, Element, Length, Padding, Radians, Rectangle, Renderer, Theme as IcedBase,
};

use crate::{color, BoxStyle, IcedTheme};

impl IcedTheme {
    /// The padding of a [`KitStyle`] as iced padding.
    fn kit_padding(st: &KitStyle) -> Padding {
        Padding::from([st.padding.1, st.padding.0])
    }

    /// [`BoxStyle`] of `kit` (not hovered).
    pub fn kit_box_style(&self, kit: Kit) -> BoxStyle {
        let st = self.theme.kit(kit, false);
        BoxStyle {
            fill: st.fill,
            stroke: st.border,
            glow: st.glow,
            hard_shadow: st.lift,
            frame: None,
            padding: Self::kit_padding(&st),
        }
    }

    /// Corner radius of `kit`: pills are fully round on rounded designs.
    fn kit_radius(&self, st: &KitStyle) -> f32 {
        match (self.theme.shape.kind(), st.pill) {
            (ShapeKind::Rounded, true) => 9999.0,
            _ => self.radius(),
        }
    }

    /// `content` in a box of `kit`; text without its own colour takes the kit's.
    pub fn kit_box<'a, Message: 'a>(
        &self,
        kit: Kit,
        content: impl Into<Element<'a, Message>>,
    ) -> Element<'a, Message> {
        self.kit_box_with(kit, content, &dodai_core::Surface::INHERIT)
    }

    /// The glass of a `kit` box: the app-wide mix where it reaches the box's kind, and `over`.
    pub fn kit_glass(&self, kit: Kit, over: &dodai_core::Surface) -> dodai_core::Glass {
        match kit.surface_class() {
            Some(c) => self.glass(c, over),
            None => dodai_core::surface_glass(
                &self.theme,
                dodai_core::SurfaceClass::Button,
                over,
                dodai_core::SurfaceFx::NONE,
            ),
        }
    }

    /// [`kit_box`](Self::kit_box) with the box's own surface override (frost, fill opacity).
    pub fn kit_box_with<'a, Message: 'a>(
        &self,
        kit: Kit,
        content: impl Into<Element<'a, Message>>,
        over: &dodai_core::Surface,
    ) -> Element<'a, Message> {
        let glass = self.kit_glass(kit, over);
        let mut st = self.theme.kit(kit, false);
        st.fill = st.fill.map(|c| glass.color(c));
        let fg = color(st.fg);
        let inner = container(content).style(move |_| container::Style {
            text_color: Some(fg),
            ..container::Style::default()
        });
        if st.pill {
            // Pills are native boxes with a full radius on every shape (a chamfered design
            // draws them square), so they never go through the canvas frame.
            let mut b = self.kit_box_style(kit);
            b.fill = st.fill;
            let style = self.box_style(&b);
            let radius = self.kit_radius(&st);
            let shadow = if st.lift {
                self.lift_shadow()
            } else {
                style.shadow
            };
            let pill = container(inner)
                .padding(Self::kit_padding(&st))
                .style(move |_| container::Style {
                    border: Border {
                        radius: radius.into(),
                        ..style.border
                    },
                    shadow,
                    ..style
                });
            return self.frosted_round(pill, glass, radius, 0.0);
        }
        let mut b = self.kit_box_style(kit);
        b.fill = st.fill;
        let inset = self.box_inset(&b);
        self.frosted_round(self.shaped_box(inner, b), glass, self.radius(), inset)
    }

    /// A segmented control: the options in one bordered strip ([`Kit::Segmented`]), the picked
    /// one filled ([`Kit::Segment`]) with a check on tonal designs. `on_select` gets the index.
    pub fn segmented<'a, Message: Clone + 'a>(
        &self,
        labels: &[String],
        selected: usize,
        on_select: impl Fn(usize) -> Message,
    ) -> Element<'a, Message> {
        let t = &self.theme;
        let tonal = t.look().tonal;
        let h = if tonal {
            t.sizes().segmented_height
        } else {
            t.shape.control_height - 6.0
        };
        let segs = iced::widget::row(labels.iter().enumerate().map(|(i, l)| {
            let on = i == selected;
            let label = if on && tonal {
                format!("\u{2713} {}", self.label(l))
            } else {
                self.label(l)
            };
            self.kit_button(
                Kit::Segment { selected: on },
                container(iced::widget::text(label).size(t.typography.size_md))
                    .height(h)
                    .align_y(iced::alignment::Vertical::Center),
                on_select(i),
            )
        }))
        .spacing(if tonal { 0.0 } else { t.spacing.half() });
        self.kit_box(Kit::Segmented, segs)
    }

    /// A clickable box of `kit`: idle and hovered styles from the tokens.
    pub fn kit_button<'a, Message: Clone + 'a>(
        &self,
        kit: Kit,
        content: impl Into<Element<'a, Message>>,
        on_press: Message,
    ) -> Element<'a, Message> {
        let idle = self.theme.kit(kit, false);
        let hover = self.theme.kit(kit, true);
        let radius = self.kit_radius(&idle);
        let width = self.theme.shape.border_width;
        let shadow = if idle.lift {
            self.lift_shadow()
        } else {
            iced::Shadow::default()
        };
        button(content)
            .padding(Self::kit_padding(&idle))
            .style(move |_t, status| {
                let st = match status {
                    button::Status::Hovered | button::Status::Pressed => hover,
                    _ => idle,
                };
                button::Style {
                    background: st.fill.map(|c| Background::Color(color(c))),
                    text_color: color(st.fg),
                    border: Border {
                        color: st.border.map_or(iced::Color::TRANSPARENT, color),
                        width: if st.border.is_some() { width } else { 0.0 },
                        radius: radius.into(),
                    },
                    shadow,
                    ..button::Style::default()
                }
            })
            .on_press(on_press)
            .into()
    }

    /// A round avatar of `size` px with `initials`.
    pub fn avatar<'a, Message: 'a>(&self, initials: &str, size: f32) -> Element<'a, Message> {
        let st = self.theme.kit(Kit::Avatar, false);
        let fg = color(st.fg);
        let bg = st.fill.map(color).unwrap_or(iced::Color::TRANSPARENT);
        container(
            text(initials.to_string())
                .size(size * 0.36)
                .color(fg)
                .align_x(alignment::Horizontal::Center),
        )
        .width(size)
        .height(size)
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center)
        .style(move |_| container::Style {
            background: Some(Background::Color(bg)),
            border: Border {
                radius: (size / 2.0).into(),
                ..Border::default()
            },
            ..container::Style::default()
        })
        .into()
    }

    /// A placeholder block of `w`×`h` (`w == 0` fills the width).
    pub fn skeleton<'a, Message: 'a>(&self, w: f32, h: f32) -> Element<'a, Message> {
        let st = self.theme.kit(Kit::Skeleton, false);
        let bg = st.fill.map(color).unwrap_or(iced::Color::TRANSPARENT);
        let radius = self.radius().min(h / 2.0);
        container("")
            .width(if w > 0.0 {
                Length::Fixed(w)
            } else {
                Length::Fill
            })
            .height(h)
            .style(move |_| container::Style {
                background: Some(Background::Color(bg)),
                border: Border {
                    radius: radius.into(),
                    ..Border::default()
                },
                ..container::Style::default()
            })
            .into()
    }

    /// A spinning ring of `size` px; `t` is the time in seconds.
    pub fn spinner<'a, Message: 'a>(&self, size: f32, t: f32) -> Element<'a, Message> {
        Canvas::new(Spinner {
            style: self.theme.look().spinner,
            lit: color(self.theme.palette.primary),
            dim: color(self.theme.palette.border),
            t,
        })
        .width(size)
        .height(size)
        .into()
    }

    /// An indeterminate bar; `t` is the time in seconds.
    pub fn indeterminate<'a, Message: 'a>(&self, t: f32) -> Element<'a, Message> {
        let tok = self.theme.progress(None);
        Canvas::new(Bar {
            track: color(tok.track),
            fill: color(tok.fill),
            left: busy::slide_left(t),
        })
        .width(Length::Fill)
        .height(4)
        .into()
    }

    /// A button of `size` with an optional leading icon; a label-less button is square-ish
    /// (as wide as it is high). `None` renders it disabled. Chamfered designs get square
    /// corners here (the canvas button has no icon slot).
    pub fn button_sized<'a, Message: Clone + 'a>(
        &self,
        label: &str,
        icon: Option<Element<'a, Message>>,
        variant: ButtonVariant,
        size: ControlSize,
        on_press: Option<Message>,
    ) -> Element<'a, Message> {
        if icon.is_none() && self.theme.shape.kind() == ShapeKind::Chamfered {
            return self
                .shaped_button_sized(label, variant, size, on_press)
                .into();
        }
        self.button_compact(label, icon, variant, size, on_press)
    }

    /// [`button_sized`](Self::button_sized) with `end` cropped the way this design crops buttons
    /// (`None`, or an end the design has no crop for, is a plain button). Cut corners and ticket
    /// stubs are drawn on a canvas, so they carry no icon; tight corners stay a native button.
    pub fn button_cropped<'a, Message: Clone + 'a>(
        &self,
        label: &str,
        icon: Option<Element<'a, Message>>,
        variant: ButtonVariant,
        size: ControlSize,
        end: Option<dodai_core::Crop>,
        on_press: Option<Message>,
    ) -> Element<'a, Message> {
        use dodai_core::ButtonCrop as B;
        let Some(crop) = end.and_then(|e| self.theme.button_crop(e)) else {
            return self.button_sized(label, icon, variant, size, on_press);
        };
        match crop {
            B::Squared {
                corners,
                radius,
                rest,
            } => {
                let pick = |on: bool| if on { radius } else { rest };
                let radii = [
                    pick(corners.top_left),
                    pick(corners.top_right),
                    pick(corners.bottom_right),
                    pick(corners.bottom_left),
                ];
                let style = self.button(variant);
                self.button_compact_styled(label, icon, size, on_press, move |t, s| {
                    let mut st = style(t, s);
                    st.border.radius = iced::border::Radius {
                        top_left: radii[0],
                        top_right: radii[1],
                        bottom_right: radii[2],
                        bottom_left: radii[3],
                    };
                    st
                })
            }
            B::CutTicked { .. } | B::Scooped { .. } if icon.is_none() => {
                let mut b = self.shaped_button_sized(label, variant, size, on_press);
                let sh = &self.theme.shape;
                match crop {
                    B::CutTicked { corners, tick } => {
                        b.shape.corners = corners;
                        b.shape.chamfer = sh.chamfer.max(sh.control_height);
                        b.shape.scoop = false;
                        b.tick = Some(crate::shaped_button::Tick {
                            edge: tick,
                            width: self.theme.spacing.md,
                            bar: sh.border_width.max(1.0),
                            gap: self.theme.tick_gap(),
                        });
                        b.padding_x += self.theme.spacing.md;
                    }
                    B::Scooped { corners, size } => {
                        b.shape.corners = corners;
                        b.shape.chamfer = size;
                        b.shape.scoop = true;
                    }
                    _ => {}
                }
                b.into()
            }
            _ => self.button_sized(label, icon, variant, size, on_press),
        }
    }

    /// Like [`button_sized`](Self::button_sized), always a native button: tight (no space
    /// reserved for a glow), with square corners on chamfered designs. For dense strips such as
    /// segmented pickers.
    pub fn button_compact<'a, Message: Clone + 'a>(
        &self,
        label: &str,
        icon: Option<Element<'a, Message>>,
        variant: ButtonVariant,
        size: ControlSize,
        on_press: Option<Message>,
    ) -> Element<'a, Message> {
        self.button_compact_styled(label, icon, size, on_press, self.button(variant))
    }

    /// This theme with the busy shimmer band swept over the buttons it builds; `secs` is the
    /// time in seconds ([`dodai_core::motion::button_shimmer`]).
    pub fn with_shimmer(&self, secs: f32) -> Self {
        let mut lt = self.clone();
        lt.shimmer = Some(dodai_core::motion::button_shimmer(secs));
        lt
    }

    /// The busy shimmer's colour on a `variant` button: a light sweep on filled buttons (dark
    /// label text would smear), the accent itself on outline and ghost ones.
    pub fn shimmer_colour(&self, variant: ButtonVariant) -> iced::Color {
        color(self.theme.shimmer_colour(variant))
    }

    /// [`button_compact`](Self::button_compact) with its own style function.
    pub fn button_compact_styled<'a, Message: Clone + 'a>(
        &self,
        label: &str,
        icon: Option<Element<'a, Message>>,
        size: ControlSize,
        on_press: Option<Message>,
        style: impl Fn(&iced::Theme, iced::widget::button::Status) -> iced::widget::button::Style + 'a,
    ) -> Element<'a, Message> {
        let m = self.theme.control_metrics(size);
        let resting = self.shimmer.map(|x| {
            (
                x,
                style(&iced::Theme::Dark, iced::widget::button::Status::Active),
            )
        });
        let mut content = row![].spacing(6).align_y(alignment::Vertical::Center);
        if let Some(i) = icon {
            content = content.push(i);
        }
        if !label.is_empty() {
            content = content.push(text(self.label(label)).size(m.text_size));
        }
        let pad = if label.is_empty() { 0.0 } else { m.padding };
        let b = button(
            container(content)
                .center_y(Length::Fill)
                .center_x(if label.is_empty() {
                    Length::Fill
                } else {
                    Length::Shrink
                }),
        )
        .padding([0.0, pad])
        .height(m.height)
        .style(style);
        let b = if label.is_empty() {
            b.width(m.height)
        } else {
            b
        };
        let el: Element<'a, Message> = b.on_press_maybe(on_press).into();
        let Some((x, st)) = resting else { return el };
        // The busy sweep over a native button, inside its rounded corners.
        let over = crate::geometry::ShimmerOver {
            radius: st.border.radius,
            color: match st.background {
                Some(_) => color(self.theme.effects.highlight),
                None => iced::Color {
                    a: 0.25,
                    ..st.border.color
                },
            },
            x,
        };
        iced::widget::stack![
            el,
            iced::widget::Canvas::new(over)
                .width(Length::Fill)
                .height(Length::Fill)
        ]
        .into()
    }

    /// Resolved colours of a button for tests and custom drawing.
    pub fn button_colours(
        &self,
        variant: ButtonVariant,
        state: ControlState,
    ) -> dodai_core::ButtonTokens {
        let mut t = self.theme.button(variant, state);
        if let Some(g) = self.glass.own {
            t.background = t.background.map(|c| g.color(c));
        }
        t
    }
}

struct Spinner {
    style: dodai_core::SpinnerStyle,
    lit: iced::Color,
    dim: iced::Color,
    t: f32,
}

impl<Message> canvas::Program<Message> for Spinner {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        _theme: &IcedBase,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let size = bounds.width.min(bounds.height);
        let cells = busy::cells(self.style, size);
        if !cells.is_empty() {
            for c in cells {
                let (x, y, w, h, a) = busy::pose(self.style, &c, size, self.t);
                let col = iced::Color {
                    a: self.lit.a * a,
                    ..self.lit
                };
                let p = iced::Point::new(x, y);
                if c.round {
                    frame.fill(
                        &Path::circle(iced::Point::new(x + w / 2.0, y + h / 2.0), w / 2.0),
                        col,
                    );
                } else if c.radius > 0.0 {
                    frame.fill(
                        &Path::rounded_rectangle(p, iced::Size::new(w, h), c.radius.into()),
                        col,
                    );
                } else {
                    frame.fill_rectangle(p, iced::Size::new(w, h), col);
                }
            }
            return vec![frame.into_geometry()];
        }
        let w = (bounds.width / 8.0).max(2.0);
        let r = (bounds.width.min(bounds.height) - w) / 2.0;
        let centre = frame.center();
        let ring = Path::circle(centre, r);
        frame.stroke(&ring, Stroke::default().with_color(self.dim).with_width(w));
        let arc = Path::new(|b| {
            b.arc(Arc {
                center: centre,
                radius: r,
                start_angle: Radians(busy::spin_angle(self.t)),
                end_angle: Radians(busy::spin_angle(self.t) + std::f32::consts::PI * 0.75),
            });
        });
        frame.stroke(&arc, Stroke::default().with_color(self.lit).with_width(w));
        vec![frame.into_geometry()]
    }
}

struct Bar {
    track: iced::Color,
    fill: iced::Color,
    left: f32,
}

impl<Message> canvas::Program<Message> for Bar {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        _theme: &IcedBase,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        frame.fill_rectangle(iced::Point::ORIGIN, bounds.size(), self.track);
        let (x, w) = segment(self.left, bounds.width);
        frame.fill_rectangle(
            iced::Point::new(x, 0.0),
            iced::Size::new(w, bounds.height),
            self.fill,
        );
        vec![frame.into_geometry()]
    }
}

/// The visible part `(x, width)` of the sliding segment of an indeterminate bar `width` px
/// wide whose left edge is at `left` (a fraction of the bar, may be negative): clipped to the
/// bar, never negative.
pub fn segment(left: f32, width: f32) -> (f32, f32) {
    let x0 = (left * width).clamp(0.0, width);
    let x1 = ((left + busy::SEGMENT) * width).clamp(0.0, width);
    (x0, (x1 - x0).max(0.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::*;

    #[test]
    fn segment_is_clipped_to_the_bar() {
        assert_eq!(segment(-busy::SEGMENT, 100.0), (0.0, 0.0));
        let (x, w) = segment(-0.1, 100.0);
        assert_eq!(x, 0.0);
        assert!((w - 20.0).abs() < 1e-3);
        let (x, w) = segment(0.9, 100.0);
        assert!((x - 90.0).abs() < 1e-3 && (x + w - 100.0).abs() < 1e-3);
        let (x, w) = segment(1.0, 100.0);
        assert_eq!((x, w), (100.0, 0.0));
        for i in 0..200 {
            let (x, w) = segment(busy::slide_left(i as f32 * 0.021), 80.0);
            assert!(x >= 0.0 && x + w <= 80.0 + 1e-3);
        }
    }

    #[test]
    fn kit_boxes_take_their_fill_from_the_kit_tokens() {
        let lt = IcedTheme::new(cyberpunk());
        let b = lt.kit_box_style(Kit::Code);
        assert_eq!(b.fill, lt.theme().kit(Kit::Code, false).fill);
        assert!(lt.kit_box_style(Kit::Popup).hard_shadow);
        assert!(!lt.kit_box_style(Kit::Skeleton).hard_shadow);
    }

    #[test]
    fn pills_are_round_only_on_rounded_designs() {
        let st = IcedTheme::new(rounded_light())
            .theme()
            .kit(Kit::Avatar, false);
        assert!(IcedTheme::new(rounded_light()).kit_radius(&st) > 1000.0);
        assert_eq!(IcedTheme::new(hard_square()).kit_radius(&st), 0.0);
        assert_eq!(IcedTheme::new(cyberpunk()).kit_radius(&st), 0.0);
    }

    #[test]
    fn bigger_buttons_are_taller() {
        let lt = IcedTheme::new(rounded_light());
        let h = |z| lt.theme().control_metrics(z).height;
        assert!(h(ControlSize::XSmall) < h(ControlSize::Medium));
        assert!(h(ControlSize::Medium) < h(ControlSize::XLarge));
    }
}

#[cfg(test)]
mod size_tests {
    use super::*;
    use crate::testutil::*;

    #[test]
    fn shaped_buttons_have_finite_sizes_at_every_step() {
        for t in dodai_core::themes::all() {
            let lt = IcedTheme::new(t);
            for z in ControlSize::ALL {
                for label in ["XS", "Primary", "Outline 2", "Loading…"] {
                    let b = lt.shaped_button_sized::<()>(label, ButtonVariant::Primary, z, None);
                    let (w, h) = b.outer_size();
                    assert!(
                        w.is_finite()
                            && h.is_finite()
                            && w > 0.0
                            && h > 0.0
                            && w < 1000.0
                            && h < 400.0,
                        "{} {z:?} {label}: {w}x{h}",
                        lt.theme().id()
                    );
                }
            }
        }
        let _ = cyberpunk();
    }
}
