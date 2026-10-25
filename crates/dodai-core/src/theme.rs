//! The [`Theme`] value: identity + palette + design tokens.

use crate::{Effects, Mode, Palette, Shape, Spacing, Typography};

/// Stable identity of a theme (safe to persist in config files).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ThemeMeta {
    /// Machine id, e.g. `"cyberpunk"`; use for persistence / CSS `data-theme`.
    pub id: &'static str,
    /// Display name, e.g. `"Cyberpunk · Yellow"`.
    pub name: &'static str,
    /// Theme family, e.g. `"cyberpunk"`.
    pub family: &'static str,
    pub description: &'static str,
    pub mode: Mode,
}

/// A complete design-system theme.
#[derive(Debug, Clone, PartialEq)]
pub struct Theme {
    pub meta: ThemeMeta,
    pub palette: Palette,
    pub typography: Typography,
    pub spacing: Spacing,
    pub shape: Shape,
    pub effects: Effects,
    /// How this design system moves: default duration, easing and entry effect.
    pub motion: dodai_motion::Motion,
    /// Explicit shape language; `None` derives one from [`Theme::shape`].
    pub shape_system: Option<crate::ShapeSystem>,
    /// This design system's component rules (see [`crate::DesignSpec`]); baseline when unset.
    pub spec: crate::SpecRef,
    /// Textures and effects this design owns, and the mix it suggests.
    pub surface_style: crate::SurfaceStyle,
}

impl Theme {
    /// A step of the [`crate::Scale`] in px (a multiple of the control height).
    pub fn scale(&self, step: crate::Scale) -> f32 {
        self.shape.control_height * step.ratio()
    }

    /// Builder: set glow strength (0 = off, 1 = as designed, >1 = stronger).
    pub fn with_glow_intensity(mut self, intensity: f32) -> Self {
        self.effects.glow_intensity = intensity.max(0.0);
        self
    }

    /// Builder: turn the fine horizontal scanline texture on or off.
    pub fn with_scanlines(mut self, on: bool) -> Self {
        self.effects.scanlines = on;
        self
    }

    /// Builder: use this shape system (per-role corners, mix and match with
    /// any palette).
    pub fn with_shape_system(mut self, sys: crate::ShapeSystem) -> Self {
        self.shape_system = Some(sys);
        self
    }

    /// The shape system in effect (explicit, or derived from the legacy tokens).
    pub fn shape_system(&self) -> crate::ShapeSystem {
        self.shape_system
            .clone()
            .unwrap_or_else(|| crate::ShapeSystem::from_shape(&self.shape))
    }

    /// Resolved corners for a component role.
    pub fn shape_for(&self, role: crate::ShapeRole) -> crate::CornerSpec {
        self.shape_system().spec(role)
    }

    /// The legacy [`crate::Shape`] for a component role: radius or chamfer from the shape
    /// system, with the theme's border width and control height. Rounded corners with differing
    /// sizes collapse to the largest.
    pub fn role_shape(&self, role: crate::ShapeRole) -> crate::Shape {
        let spec = self.shape_for(role);
        let mut s = self.shape.clone();
        let cs = [
            spec.top_left,
            spec.top_right,
            spec.bottom_right,
            spec.bottom_left,
        ];
        let size = cs.iter().map(|c| c.size()).fold(0.0_f32, f32::max);
        use crate::shape_system::Corner;
        let cut = cs.iter().any(|c| matches!(c, Corner::Cut(_)));
        let scoop = cs.iter().any(|c| matches!(c, Corner::Scoop(_)));
        s.scoop = scoop;
        if cut || scoop {
            s.radius = 0.0;
            s.chamfer = size;
            s.corners = spec.sides();
        } else {
            s.radius = size;
            s.chamfer = 0.0;
        }
        s
    }

    /// The design system's declared capabilities (the classic look when it has no spec).
    pub fn look(&self) -> crate::Look {
        self.spec.get().map(|d| d.look()).unwrap_or_default()
    }

    /// Builder: attach a design-system spec (component rules and geometry).
    pub fn with_spec(mut self, spec: &'static dyn crate::DesignSpec) -> Self {
        self.spec = crate::SpecRef::new(spec);
        self
    }

    /// Builder: denser (`< 1`) or roomier (`> 1`) layout. Scales the control height and the
    /// spacing scale, so every control, row and gap follows; type, icons and fixed parts
    /// ([`crate::Sizes`]) keep their size. `1.0` changes nothing. Clamped to `0.5..=2.0`, and the
    /// control height stays on a half pixel so edges stay crisp.
    pub fn with_density(mut self, density: f32) -> Self {
        let d = if density.is_finite() {
            density.clamp(0.5, 2.0)
        } else {
            1.0
        };
        let snap = |v: f32| (v * d * 2.0).round() / 2.0;
        self.shape.control_height = snap(self.shape.control_height);
        let sp = &mut self.spacing;
        (sp.xs, sp.sm, sp.md, sp.lg, sp.xl) = (
            snap(sp.xs),
            snap(sp.sm),
            snap(sp.md),
            snap(sp.lg),
            snap(sp.xl),
        );
        self
    }

    /// Builder for touch screens: every control is at least `min_target` px tall (use
    /// [`Theme::TOUCH_TARGET`], 48 px) and neighbouring targets are at least
    /// [`Theme::TOUCH_GAP`] px apart. Larger controls and gaps are left alone, so a design that
    /// is already roomy (Cyberpunk, Aurora at 48) does not change. Type and icons keep their size.
    pub fn with_min_target(mut self, min_target: f32) -> Self {
        if min_target.is_finite() {
            let h = (min_target.clamp(24.0, 96.0) * 2.0).round() / 2.0;
            self.shape.control_height = self.shape.control_height.max(h);
        }
        let sp = &mut self.spacing;
        sp.xs = sp.xs.max(Self::TOUCH_GAP / 2.0);
        sp.sm = sp.sm.max(Self::TOUCH_GAP);
        sp.md = sp.md.max(sp.sm);
        sp.lg = sp.lg.max(sp.md);
        sp.xl = sp.xl.max(sp.lg);
        self
    }

    /// Recommended minimum touch target, px (Material 48 dp; Apple asks for 44 pt).
    pub const TOUCH_TARGET: f32 = 48.0;
    /// Recommended minimum gap between touch targets, px.
    pub const TOUCH_GAP: f32 = 8.0;

    /// Builder: cut these corners on chamfered controls (see [`crate::Corners`]).
    pub fn with_corners(mut self, c: crate::Corners) -> Self {
        self.shape.corners = c;
        self
    }

    pub fn id(&self) -> &'static str {
        self.meta.id
    }
    pub fn name(&self) -> &'static str {
        self.meta.name
    }
    pub fn is_dark(&self) -> bool {
        self.meta.mode == Mode::Dark
    }
}

impl Default for Theme {
    fn default() -> Self {
        crate::themes::neutral()
    }
}

#[cfg(test)]
mod density_tests {
    use crate::Theme;

    #[test]
    fn one_changes_nothing() {
        let t = Theme::default();
        let d = t.clone().with_density(1.0);
        assert_eq!(d.shape.control_height, t.shape.control_height);
        assert_eq!(d.spacing, t.spacing);
    }

    #[test]
    fn denser_shrinks_controls_and_gaps_and_roomier_grows_them() {
        let t = Theme::default();
        let dense = t.clone().with_density(0.8);
        let roomy = t.clone().with_density(1.25);
        assert!(dense.shape.control_height < t.shape.control_height);
        assert!(roomy.shape.control_height > t.shape.control_height);
        assert!(dense.spacing.md < t.spacing.md && roomy.spacing.md > t.spacing.md);
        assert_eq!(
            dense.shape.control_height * 2.0,
            (dense.shape.control_height * 2.0).round()
        );
    }

    #[test]
    fn out_of_range_and_nan_are_tamed() {
        let t = Theme::default();
        assert_eq!(t.clone().with_density(f32::NAN).spacing, t.spacing);
        assert_eq!(
            t.clone().with_density(100.0).shape.control_height,
            t.clone().with_density(2.0).shape.control_height
        );
    }
}
