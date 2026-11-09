//! App-wide shape tuning: how round the corners are and how borders look, on top of whatever a
//! design defines. Like the surface mix it is a runtime setting, not part of a design.

use crate::{Color, CornerStyle, RoleShape, ShapeKind, ShapeRole, ShapeSize, ShapeSystem, Theme};

/// Which colour borders take.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BorderTint {
    /// Each component's own border colour.
    #[default]
    Design,
    /// The stronger neutral outline.
    Strong,
    /// The primary colour.
    Accent,
    /// The text colour.
    Text,
}

impl BorderTint {
    /// Every tint, in picker order.
    pub const ALL: [Self; 4] = [Self::Design, Self::Strong, Self::Accent, Self::Text];
    /// Picker label (`Design` reads "Default" in the UI).
    pub const fn label(self) -> &'static str {
        match self {
            Self::Design => "Default",
            Self::Strong => "Strong",
            Self::Accent => "Accent",
            Self::Text => "Text",
        }
    }
}

/// Which colour filled controls take: buttons, checked boxes, on switches, slider fills, the
/// selected tab. Every choice comes from the current theme, so each design, accent and mode
/// offers its own set; see [`ControlColour::offered`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ControlColour {
    /// The design's own primary.
    #[default]
    Design,
    /// The theme's accent.
    Accent,
    /// The theme's secondary colour.
    Secondary,
    /// The text colour: quiet, monochrome controls.
    Ink,
}

impl ControlColour {
    /// Every choice, in picker order.
    pub const ALL: [Self; 4] = [Self::Design, Self::Accent, Self::Secondary, Self::Ink];

    /// Picker label (`Design` reads "Default" in the UI).
    pub const fn label(self) -> &'static str {
        match self {
            Self::Design => "Default",
            Self::Accent => "Accent",
            Self::Secondary => "Secondary",
            Self::Ink => "Ink",
        }
    }

    /// The fill this choice gives on `t`: the source colour pushed to 3:1 against both the
    /// page and a card, so a filled control always stands out.
    pub fn colour(self, t: &Theme) -> Color {
        let p = &t.palette;
        let base = match self {
            Self::Design => return p.primary,
            Self::Accent => p.accent,
            Self::Secondary => p.secondary,
            Self::Ink => p.text,
        };
        let (bg, card) = (p.background, p.surface);
        let lower = if bg.contrast(base) < card.contrast(base) {
            bg
        } else {
            card
        };
        base.with_contrast(lower, 3.0)
    }

    /// The choices that look different on `t`, `Design` first. A choice that lands on the
    /// design's own colour (or an earlier choice) is left out, e.g. `Ink` where the primary
    /// already is the text colour.
    pub fn offered(t: &Theme) -> Vec<Self> {
        let mut out: Vec<(Self, Color)> = Vec::new();
        for c in Self::ALL {
            let col = c.colour(t);
            if out.iter().all(|(_, o)| !looks_alike(*o, col)) {
                out.push((c, col));
            }
        }
        out.into_iter().map(|(c, _)| c).collect()
    }
}

/// Whether two colours read as the same at a glance.
fn looks_alike(a: Color, b: Color) -> bool {
    let d = |x: u8, y: u8| x.abs_diff(y);
    d(a.r, b.r).max(d(a.g, b.g)).max(d(a.b, b.b)) < 28
}

impl Theme {
    /// This theme with filled controls in `c` (see [`ControlColour`]): the primary, its hover
    /// and pressed shades, a readable label colour and the primary-family roles follow.
    pub fn with_control_colour(mut self, c: ControlColour) -> Self {
        if c == ControlColour::Design {
            return self;
        }
        let fill = c.colour(&self);
        let on = self.on(fill);
        let p = &mut self.palette;
        p.primary = fill;
        p.primary_hover = fill.mix(p.background, 0.18);
        p.primary_active = fill.mix(p.background, 0.3);
        p.on_primary = on;
        let d = crate::palette::Roles::derive(p, self.meta.mode == crate::Mode::Dark);
        let r = &mut p.roles;
        r.primary_container = d.primary_container;
        r.inverse_primary = d.inverse_primary;
        r.primary_fixed = d.primary_fixed;
        r.primary_fixed_dim = d.primary_fixed_dim;
        r.on_primary_fixed = d.on_primary_fixed;
        r.on_primary_fixed_variant = d.on_primary_fixed_variant;
        // Tonal secondary controls (Material) follow the chosen colour too.
        let mut tinted = p.clone();
        tinted.secondary = fill;
        let ds = crate::palette::Roles::derive(&tinted, self.meta.mode == crate::Mode::Dark);
        let r = &mut p.roles;
        r.secondary_container = ds.secondary_container;
        r.on_secondary_container = ds.on_secondary_container;
        self
    }
}

/// Which way a hard offset shadow falls.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ShadowDir {
    /// Down and to the right (the usual).
    #[default]
    BottomRight,
    /// Down and to the left.
    BottomLeft,
    /// Up and to the right.
    TopRight,
    /// Up and to the left.
    TopLeft,
}

impl ShadowDir {
    /// Every direction, in picker order.
    pub const ALL: [Self; 4] = [
        Self::BottomRight,
        Self::BottomLeft,
        Self::TopRight,
        Self::TopLeft,
    ];
    /// Picker label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::BottomRight => "Bottom right",
            Self::BottomLeft => "Bottom left",
            Self::TopRight => "Top right",
            Self::TopLeft => "Top left",
        }
    }
    /// Unit offset `(dx, dy)`, y pointing down.
    pub const fn dir(self) -> (f32, f32) {
        match self {
            Self::BottomRight => (1.0, 1.0),
            Self::BottomLeft => (-1.0, 1.0),
            Self::TopRight => (1.0, -1.0),
            Self::TopLeft => (-1.0, -1.0),
        }
    }
}

/// How strongly a design's gradient shows: the canvas mesh, the `wash` over surfaces and the
/// `rim` along outlines. Only designs that have a gradient (Dodai) react to it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GradientMode {
    /// No gradient anywhere: a flat canvas and plain surfaces.
    Off,
    /// As designed.
    #[default]
    Subtle,
    /// A stronger mesh, wash and rim.
    Bold,
}

impl GradientMode {
    /// Every mode, in picker order.
    pub const ALL: [Self; 3] = [Self::Off, Self::Subtle, Self::Bold];
    /// Picker label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::Subtle => "Subtle",
            Self::Bold => "Bold",
        }
    }
    /// Multiplier on the wash and rim effects.
    pub const fn effect_gain(self) -> f32 {
        match self {
            Self::Off => 0.0,
            Self::Subtle => 1.0,
            Self::Bold => 1.25,
        }
    }
    /// Multiplier on the canvas mesh.
    pub const fn mesh_gain(self) -> f32 {
        match self {
            Self::Off => 0.0,
            Self::Subtle => 1.0,
            Self::Bold => 1.9,
        }
    }
}

/// A family of components that can be rounded on its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoundnessGroup {
    /// Things you press or type into: buttons, chips, tags, badges, inputs, checkboxes.
    Controls,
    /// Things that hold content: cards, panels, menus, dialogs, sheets, bars.
    Containers,
    /// Round or small marks: icon buttons, FABs, indicators, tooltips.
    Marks,
}

impl RoundnessGroup {
    /// Every group, in picker order.
    pub const ALL: [Self; 3] = [Self::Controls, Self::Containers, Self::Marks];
    /// Picker label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Controls => "Controls",
            Self::Containers => "Containers",
            Self::Marks => "Marks",
        }
    }
    const fn index(self) -> usize {
        match self {
            Self::Controls => 0,
            Self::Containers => 1,
            Self::Marks => 2,
        }
    }
    /// The group a role belongs to.
    pub const fn of(role: ShapeRole) -> Self {
        use ShapeRole::*;
        match role {
            Button | Chip | Tag | Badge | Input | Checkbox | SearchBar => Self::Controls,
            Card | Panel | Menu | Dialog | Sheet | SideSheet | Snackbar | Carousel | Toolbar
            | AppBar | RichTooltip => Self::Containers,
            Fab | FabSmall | FabLarge | IconButton | Indicator | Tooltip => Self::Marks,
        }
    }
}

/// Shape tuning applied to every component.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShapeTune {
    /// Multiplier on rounded corners: 0 square, 1 as designed, above 1 rounder.
    pub roundness: f32,
    /// Extra multiplier per [`RoundnessGroup`] on top of `roundness`, indexed like
    /// [`RoundnessGroup::ALL`]; 1 leaves the group at the global value.
    pub groups: [f32; 3],
    /// Border width in px; `None` keeps each design's own.
    pub border_width: Option<f32>,
    /// Border colour.
    pub tint: BorderTint,
    /// Give components that have no border one.
    pub outline_all: bool,
    /// Offset in px of hard shadows; `None` keeps each design's own (designs without hard
    /// shadows stay without).
    pub hard_shadow: Option<f32>,
    /// Direction hard shadows fall.
    pub shadow_dir: ShadowDir,
    /// Draw every corner as this style (round, straight cut, concave scoop); `None` keeps each
    /// design's own.
    pub corner_style: Option<CornerStyle>,
    /// Control height and spacing multiplier (see [`Theme::with_density`]); 1 as designed.
    pub density: f32,
    /// How strongly the design's gradient shows (see [`GradientMode`]).
    pub gradient: GradientMode,
    /// Which colour filled controls take (see [`ControlColour`]).
    pub control_colour: ControlColour,
}

impl ShapeTune {
    /// The design as shipped.
    pub const DEFAULT: Self = Self {
        roundness: 1.0,
        groups: [1.0; 3],
        border_width: None,
        tint: BorderTint::Design,
        outline_all: false,
        hard_shadow: None,
        shadow_dir: ShadowDir::BottomRight,
        corner_style: None,
        density: 1.0,
        gradient: GradientMode::Subtle,
        control_colour: ControlColour::Design,
    };

    /// Corner multiplier for one role: the global roundness times its group's.
    pub fn roundness_of(&self, role: ShapeRole) -> f32 {
        (self.roundness * self.groups[RoundnessGroup::of(role).index()]).max(0.0)
    }

    /// Whether every role is rounded the same.
    fn uniform(&self) -> bool {
        self.groups.iter().all(|g| (g - 1.0).abs() <= f32::EPSILON)
    }

    /// Whether this changes nothing.
    pub fn is_default(&self) -> bool {
        *self == Self::DEFAULT
    }
}

impl Default for ShapeTune {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl ShapeSystem {
    /// Whether any role (or the fallback) is a pill.
    fn has_pill(&self) -> bool {
        self.default.size == ShapeSize::Full || self.roles.iter().any(|r| r.size == ShapeSize::Full)
    }

    /// This shape language with every rounded corner scaled by `f`: 0 is square, 1 unchanged.
    /// See [`with_roundness_by`](Self::with_roundness_by).
    pub fn with_roundness(&self, f: f32, control_height: f32) -> Self {
        self.with_roundness_by(control_height, |_| f)
    }

    /// This shape language with each role's rounded corners scaled by `factor(role)`.
    /// Cut corners keep their size. A fully rounded corner becomes a real radius
    /// (`pill` × `control_height`), scaled like the rest.
    pub fn with_roundness_by(
        &self,
        control_height: f32,
        factor: impl Fn(ShapeRole) -> f32,
    ) -> Self {
        let px = |size: ShapeSize| match size {
            ShapeSize::Full => self.pill * control_height,
            other => self.scale.px(other),
        };
        let mut out = self.clone();
        // Spell out every role so each can take its own factor, not just the listed ones.
        out.roles = ShapeRole::ALL
            .iter()
            .map(|&role| {
                let (style, size, sides) = self
                    .roles
                    .iter()
                    .rev()
                    .find(|e| e.role == role)
                    .map(|e| (e.style, e.size, e.sides))
                    .unwrap_or((self.default.style, self.default.size, self.default.sides));
                let size = if style == CornerStyle::Cut || size == ShapeSize::None {
                    size
                } else {
                    ShapeSize::Px(px(size) * factor(role).max(0.0))
                };
                RoleShape::new(role, style, size, sides)
            })
            .collect();
        // Anything reading the named steps directly follows the Controls factor.
        let f = factor(ShapeRole::Button).max(0.0);
        out.scale.xs *= f;
        out.scale.sm *= f;
        out.scale.md *= f;
        out.scale.lg *= f;
        out.scale.xl *= f;
        out
    }
}

impl Theme {
    /// This theme with `tune` applied. Chamfered designs keep their cuts; roundness only
    /// affects rounded corners.
    pub fn tuned(&self, tune: &ShapeTune) -> Theme {
        let mut t = self.clone();
        t.tune = *tune;
        let sys = match tune.corner_style {
            Some(style) => self.shape_system().with_style(style),
            None => self.shape_system(),
        };
        if tune.corner_style.is_some() {
            t.shape_system = Some(sys.clone());
        }
        let rounds = (tune.roundness - 1.0).abs() > f32::EPSILON || !tune.uniform();
        if (rounds || sys.has_pill()) && t.shape.kind() != ShapeKind::Chamfered {
            t.shape.radius *= tune.roundness.max(0.0);
            t.shape_system =
                Some(sys.with_roundness_by(self.shape.control_height, |r| tune.roundness_of(r)));
        }
        if let Some(w) = tune.border_width {
            t.shape.border_width = w.max(0.0);
        }
        if let Some(s) = tune.hard_shadow {
            if t.effects.hard_shadow > 0.0 {
                t.effects.hard_shadow = s.max(0.0);
            }
        }
        if (tune.density - 1.0).abs() > f32::EPSILON {
            t = t.with_density(tune.density);
        }
        if tune.gradient != GradientMode::Subtle {
            t.surface_style.backdrop *= tune.gradient.mesh_gain();
            t.surface_style.gradient = tune.gradient.effect_gain();
        }
        // A choice this theme does not offer (it looks like another one) reads as the default,
        // as the picker shows it.
        let c = if ControlColour::offered(&t).contains(&tune.control_colour) {
            tune.control_colour
        } else {
            ControlColour::Design
        };
        t.with_control_colour(c)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn density_in_the_tune_scales_controls() {
        let t = Theme::default();
        let roomy = t.tuned(&ShapeTune {
            density: 1.5,
            ..ShapeTune::DEFAULT
        });
        assert!(roomy.shape.control_height > t.shape.control_height);
        assert!(roomy.spacing.md > t.spacing.md);
    }

    #[test]
    fn min_target_raises_small_controls_only() {
        let t = Theme::default();
        let touch = t.clone().with_min_target(Theme::TOUCH_TARGET);
        assert!(touch.shape.control_height >= Theme::TOUCH_TARGET);
        assert!(touch.spacing.sm >= Theme::TOUCH_GAP);
        // Already-roomy themes are unchanged, and the call is idempotent.
        assert_eq!(touch.clone().with_min_target(Theme::TOUCH_TARGET), touch);
        let small = t.with_min_target(10.0);
        assert!(small.shape.control_height >= 24.0);
    }

    #[test]
    fn gradient_mode_scales_the_mesh_and_the_effects() {
        let base = Theme::default();
        assert_eq!(base.tuned(&ShapeTune::DEFAULT), base);
        let mut with = base.clone();
        with.surface_style.backdrop = 0.4;
        let off = with.tuned(&ShapeTune {
            gradient: GradientMode::Off,
            ..ShapeTune::DEFAULT
        });
        assert_eq!(off.surface_style.backdrop, 0.0);
        assert_eq!(off.surface_style.gradient, 0.0);
        let bold = with.tuned(&ShapeTune {
            gradient: GradientMode::Bold,
            ..ShapeTune::DEFAULT
        });
        assert!(bold.surface_style.backdrop > 0.4 && bold.surface_style.gradient > 1.0);
        assert_eq!(GradientMode::ALL.len(), 3);
    }

    #[test]
    fn every_offered_control_colour_stands_out_and_reads() {
        for base in crate::themes::all() {
            let offered = ControlColour::offered(&base);
            assert_eq!(offered[0], ControlColour::Design, "{}", base.id());
            for (i, &c) in offered.iter().enumerate() {
                let t = base.tuned(&ShapeTune {
                    control_colour: c,
                    ..ShapeTune::DEFAULT
                });
                let p = &t.palette;
                if c != ControlColour::Design {
                    for bg in [p.background, p.surface] {
                        assert!(p.primary.contrast(bg) >= 3.0, "{} {c:?}", base.id());
                    }
                    assert!(
                        p.on_primary.contrast(p.primary) >= 3.0,
                        "{} {c:?}",
                        base.id()
                    );
                }
                for &other in &offered[..i] {
                    assert!(
                        !looks_alike(c.colour(&base), other.colour(&base)),
                        "{} {c:?} repeats {other:?}",
                        base.id()
                    );
                }
            }
        }
    }

    #[test]
    fn control_colour_is_offered_per_theme_combination() {
        let ink_led = crate::themes::all()
            .into_iter()
            .find(|t| looks_alike(t.palette.primary, t.palette.text))
            .expect("a design whose primary is its ink");
        assert!(!ControlColour::offered(&ink_led).contains(&ControlColour::Ink));
        let t = Theme::default();
        let accent = t.clone().with_control_colour(ControlColour::Accent);
        assert_ne!(accent.palette.primary, t.palette.primary);
        assert_eq!(
            accent.palette.text, t.palette.text,
            "only the fill family moves"
        );
        assert_eq!(t.clone().with_control_colour(ControlColour::Design), t);
    }

    #[test]
    fn control_colour_retints_the_tonal_secondary_container() {
        let t = Theme::default();
        let accent = t.clone().with_control_colour(ControlColour::Accent);
        assert_ne!(
            accent.palette.roles.secondary_container,
            t.palette.roles.secondary_container
        );
    }

    #[test]
    fn default_tuning_changes_nothing() {
        let t = Theme::default();
        assert_eq!(t.tuned(&ShapeTune::DEFAULT), t);
    }

    #[test]
    fn roundness_zero_makes_every_role_square() {
        let sys = ShapeSystem::from_shape(&crate::Shape {
            corners: crate::Corners::NONE,
            radius: 12.0,
            chamfer: 0.0,
            scoop: false,
            border_width: 1.0,
            control_height: 40.0,
        });
        let sq = sys.with_roundness(0.0, 40.0);
        assert_eq!(
            sq.spec(crate::ShapeRole::Button).top_left.size(),
            0.0,
            "scaled to nothing"
        );
        assert!(sys.spec(crate::ShapeRole::Button).top_left.size() > 0.0);
    }

    fn pill_system() -> ShapeSystem {
        ShapeSystem::from_shape(&crate::Shape {
            corners: crate::Corners::NONE,
            radius: 12.0,
            chamfer: 0.0,
            scoop: false,
            border_width: 1.0,
            control_height: 40.0,
        })
        .with_default(
            crate::CornerStyle::Round,
            ShapeSize::Full,
            crate::Corners::ALL,
        )
        .with_role(
            ShapeRole::Card,
            crate::CornerStyle::Round,
            ShapeSize::Medium,
            crate::Corners::ALL,
        )
    }

    fn size(sys: &ShapeSystem, role: ShapeRole) -> f32 {
        sys.spec(role).top_left.size()
    }

    #[test]
    fn pills_scale_like_any_other_corner() {
        let sys = pill_system();
        // 100%: a real radius below a full pill (20 on a 40 px control), so there is headroom.
        assert_eq!(size(&sys.with_roundness(1.0, 40.0), ShapeRole::Chip), 16.0);
        assert_eq!(size(&sys.with_roundness(1.5, 40.0), ShapeRole::Chip), 24.0);
        assert_eq!(size(&sys.with_roundness(0.5, 40.0), ShapeRole::Chip), 8.0);
        assert_eq!(size(&sys.with_roundness(2.0, 40.0), ShapeRole::Card), 24.0);
    }

    #[test]
    fn groups_round_only_their_own_roles() {
        let sys = pill_system();
        let by = sys.with_roundness_by(40.0, |r| {
            if RoundnessGroup::of(r) == RoundnessGroup::Containers {
                0.0
            } else {
                1.0
            }
        });
        assert_eq!(size(&by, ShapeRole::Card), 0.0, "containers squared");
        assert_eq!(size(&by, ShapeRole::Chip), 16.0, "controls untouched");
    }

    #[test]
    fn tune_groups_multiply_the_global_roundness() {
        let mut tune = ShapeTune::DEFAULT;
        tune.roundness = 2.0;
        tune.groups[1] = 0.5;
        assert_eq!(tune.roundness_of(ShapeRole::Card), 1.0);
        assert_eq!(tune.roundness_of(ShapeRole::Button), 2.0);
    }

    #[test]
    fn every_role_has_a_group() {
        for r in ShapeRole::ALL {
            let _ = RoundnessGroup::of(r);
        }
    }

    #[test]
    fn corner_style_scoops_and_cuts_every_role() {
        let t = Theme::default();
        let size = |t: &Theme| t.role_shape(ShapeRole::Card);
        let scooped = t.tuned(&ShapeTune {
            corner_style: Some(CornerStyle::Scoop),
            ..ShapeTune::DEFAULT
        });
        let s = size(&scooped);
        assert!(s.scoop && s.chamfer > 0.0, "scoop carries a size");
        let cut = t.tuned(&ShapeTune {
            corner_style: Some(CornerStyle::Cut),
            ..ShapeTune::DEFAULT
        });
        let c = size(&cut);
        assert!(!c.scoop && c.chamfer > 0.0);
        assert_eq!(c.chamfer, s.chamfer, "same size, different edge");
        assert!(s.outline(100.0, 60.0).len() > c.outline(100.0, 60.0).len());
    }

    #[test]
    fn scooped_outline_bows_into_the_box() {
        let sh = crate::Shape {
            corners: crate::Corners::TOP_LEFT,
            radius: 0.0,
            chamfer: 10.0,
            scoop: true,
            border_width: 1.0,
            control_height: 40.0,
        };
        let p = sh.outline(100.0, 50.0);
        let straight = crate::Shape {
            scoop: false,
            ..sh.clone()
        }
        .outline(100.0, 50.0);
        assert!(p.len() > straight.len());
        // Every arc point sits inside the straight cut's triangle's far side: x + y >= 10.
        assert!(p.iter().take(9).all(|&(x, y)| x + y >= 10.0 - 1e-3));
        assert!(p
            .iter()
            .all(|&(x, y)| (0.0..=100.0).contains(&x) && (0.0..=50.0).contains(&y)));
    }
}
