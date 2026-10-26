//! The Neo-Brutalism **design-system spec**: every surface is a flat block with an outline and
//! a hard shadow; selected things fill with the main colour; nothing glows.

use dodai_core::{
    AppBarKind, AppBarTokens, ButtonTokens, ButtonVariant, CardVariant, Color, ControlSize,
    ControlState, CornerStyle, DefaultShape, DesignSpec, InputTokens, ListItemTokens,
    NavigationTokens, ProgressTokens, ShapePreset, ShapeScale, ShapeSize, ShapeSystem, SizeMetrics,
    StepState, StepTokens, SurfaceTokens, TabTokens, Theme, ToggleTokens, Tone, TypeRole,
};
use dodai_core::{Corners, IconSet, Look, Sizes};

/// The neo-brutalism spec (attach with [`dodai_core::Theme::with_spec`]).
#[derive(Debug)]
pub struct NeoBrutalism;

/// Shared instance for `Theme::with_spec`.
pub static SPEC: NeoBrutalism = NeoBrutalism;

/// Corner sizes: barely rounded.
pub const SHAPE_SCALE: ShapeScale = ShapeScale {
    xs: 2.0,
    sm: 5.0,
    md: 8.0,
    lg: 12.0,
    xl: 16.0,
};

/// The default shape language: 5 px corners everywhere.
pub fn shape_system() -> ShapeSystem {
    ShapeSystem {
        pill: ShapeSystem::PILL_OF_CONTROL,
        name: "Neo-Brutalism",
        scale: SHAPE_SCALE,
        roles: std::borrow::Cow::Borrowed(&[]),
        default: DefaultShape {
            style: CornerStyle::Round,
            size: ShapeSize::Small,
            sides: Corners::ALL,
        },
    }
}

/// Shape presets: the kit's 5 px corners, fully square, and pillowy.
pub fn shape_presets() -> Vec<ShapePreset> {
    let mut soft = shape_system();
    soft.name = "Soft";
    soft.default.size = ShapeSize::Large;
    vec![
        ShapePreset::new("brut", "Brutal", shape_system()),
        ShapePreset::new("sharp", "Sharp", ShapeSystem::sharp()),
        ShapePreset::new("soft", "Soft", soft),
    ]
}

fn block(bg: Option<Color>, fg: Color, line: Color, shadow: bool) -> SurfaceTokens {
    SurfaceTokens {
        background: bg,
        border: Some(line),
        foreground: fg,
        glow: false,
        shadow,
    }
}

/// Frosted glass: blur sigma in px and tint opacity.
const FROST_BLUR: f32 = 10.0;
const FROST_ALPHA: f32 = 0.72;
/// Hard outline width of a frosted panel, px.
const FROST_EDGE: f32 = 2.0;

impl DesignSpec for NeoBrutalism {
    fn name(&self) -> &'static str {
        "Neo-Brutalism"
    }

    /// Heavy strokes, ink outlines on every mark, and a solid fill under areas.
    fn chart_style(&self, t: &Theme) -> Option<dodai_core::ChartStyle> {
        Some(dodai_core::ChartStyle {
            line_width: 3.0,
            mark_outline: Some(t.palette.border),
            mark_outline_width: t.shape.border_width,
            grid_color: t.palette.border.with_alpha(0.35),
            area_alpha: 0.35,
            dial: dodai_core::DialLook::Analog,
            cluster: dodai_core::ClusterKind::Bold,
            scope: dodai_core::ScopeLook::Ink,
            ..dodai_core::ChartStyle::from_theme(t)
        })
    }

    /// Frosted glass, brutalist: an opaque-ish paper tint over a short, gritty 10 px blur. The
    /// hard border and shadow of the panel do the rest.
    fn frost(&self, t: &Theme) -> Option<dodai_core::Frost> {
        Some(dodai_core::Frost {
            tint: t.palette.surface.with_alpha(FROST_ALPHA),
            blur: FROST_BLUR,
            edge: Some((t.palette.border_strong, FROST_EDGE)),
        })
    }

    /// A chunky grid: wide gutters and margins so the hard shadows never touch.
    fn page_grid(&self, _t: &Theme, viewport: f32) -> Option<dodai_core::GridSpec> {
        Some(dodai_core::GridSpec {
            columns: if viewport >= 760.0 { 12 } else { 4 },
            gap: GRID_GAP,
            margin: GRID_MARGIN,
            max_width: Some(GRID_WIDTH_MAX),
        })
    }

    fn look(&self) -> Look {
        Look {
            sink: true,
            icons: IconSet::Bold,
            ..Look::default()
        }
    }

    /// The hard shadow is the outline colour: black on paper, off-white on graphite.
    fn hard_shadow_color(&self, t: &Theme) -> Option<Color> {
        Some(t.palette.border)
    }

    fn card(&self, t: &Theme, v: CardVariant) -> Option<SurfaceTokens> {
        let p = &t.palette;
        Some(match v {
            CardVariant::Elevated => block(Some(p.surface), p.text, p.border, true),
            CardVariant::Filled => block(Some(p.primary), p.on_primary, p.border, true),
            CardVariant::Outlined => block(None, p.text, p.border, false),
        })
    }

    /// Chips are small blocks; selected ones fill with the main colour.
    fn chip(&self, t: &Theme, selected: bool, hovered: bool) -> Option<SurfaceTokens> {
        let p = &t.palette;
        Some(if selected {
            block(Some(p.primary), p.on_primary, p.border, false)
        } else {
            block(
                Some(if hovered { p.surface_raised } else { p.surface }),
                p.text,
                p.border,
                false,
            )
        })
    }

    fn dialog(&self, t: &Theme) -> Option<SurfaceTokens> {
        let p = &t.palette;
        Some(block(Some(p.surface), p.text, p.border, true))
    }

    fn sheet(&self, t: &Theme) -> Option<SurfaceTokens> {
        let p = &t.palette;
        Some(block(Some(p.surface), p.text, p.border, true))
    }

    fn popup(&self, t: &Theme) -> Option<SurfaceTokens> {
        let p = &t.palette;
        Some(block(Some(p.surface), p.text, p.border, true))
    }

    /// Black at 80 %.
    fn scrim(&self, _t: &Theme) -> Option<Color> {
        Some(Color::hex(0x000000).with_alpha(0.8))
    }

    fn step(&self, t: &Theme, state: StepState) -> Option<StepTokens> {
        let p = &t.palette;
        Some(match state {
            StepState::Done => StepTokens {
                fill: Some(p.primary),
                stroke: p.border,
                mark: p.on_primary,
                text: p.text,
                glow: false,
            },
            StepState::Current => StepTokens {
                fill: Some(p.secondary),
                stroke: p.border,
                mark: p.on_secondary,
                text: p.text,
                glow: false,
            },
            StepState::Todo => StepTokens {
                fill: None,
                stroke: p.border,
                mark: p.text_muted,
                text: p.text_muted,
                glow: false,
            },
        })
    }

    /// The active tab is a main-colour block.
    fn tab(&self, t: &Theme, active: bool, hovered: bool) -> Option<TabTokens> {
        let p = &t.palette;
        Some(if active {
            TabTokens {
                background: Some(p.primary),
                foreground: p.on_primary,
                indicator: Some(p.border),
                outline: true,
            }
        } else {
            TabTokens {
                background: hovered.then(|| p.primary.with_alpha(0.7)),
                foreground: if hovered { p.on_primary } else { p.text },
                indicator: None,
                outline: false,
            }
        })
    }

    /// Hover and selection fill with the main colour (the kit's sidebar rows).
    /// Chunkier hit areas to match the thick outlines.
    fn sizes(&self, _: &Theme) -> Option<Sizes> {
        Some(Sizes {
            selection_box: 22.0,
            slider_thumb: 20.0,
            ..Sizes::default()
        })
    }

    /// Neo-brutalism drawer / rail geometry, fixed in px (control height 40.0).
    fn navigation(&self, t: &Theme) -> Option<NavigationTokens> {
        Some(NavigationTokens {
            item_height: 32.0,
            nested_height: 26.0,
            headline_height: 32.0,
            item_padding: 16.0,
            container_padding: 8.0,
            item_content_gap: 8.0,
            item_gap: 8.0,
            indent: 16.0,
            icon_size: t.sizes().icon_sm,
            drawer_max_width: 360.0,
            rail_width: 80.0,
            indicator: (70.0, 40.0),
            rail_label_gap: 8.0,
            bar_height: 80.0,
            bar_indicator_width: 78.0,
            pill: false,
            container: None,
            roles: (
                TypeRole::LabelLarge,
                TypeRole::LabelMedium,
                TypeRole::LabelMedium,
                TypeRole::TitleSmall,
            ),
        })
    }

    fn navigation_item(&self, t: &Theme, selected: bool, hovered: bool) -> Option<ListItemTokens> {
        self.list_item(t, selected, hovered)
    }

    fn divider(&self, t: &Theme) -> Option<Color> {
        Some(t.palette.border)
    }

    fn fab(&self, t: &Theme, v: ButtonVariant, s: ControlState) -> Option<ButtonTokens> {
        self.button(t, v, s)
    }

    /// XS to XL control sizes, fixed in px.
    fn control_metrics(&self, t: &Theme, size: ControlSize) -> Option<SizeMetrics> {
        let i = size as usize;
        let ty = &t.typography;
        Some(SizeMetrics {
            height: [30.0, 40.0, 50.0, 64.0, 80.0][i],
            padding: [12.0, 16.0, 20.0, 28.0, 36.0][i],
            text_size: [ty.size_sm, ty.size_md, ty.size_lg, ty.size_xl, ty.size_xxl][i],
            icon_size: [18.0, 20.0, 20.0, 22.0, 26.0][i],
            square_radius: [12.0, 12.0, 16.0, 28.0, 28.0][i],
        })
    }

    fn list_item(&self, t: &Theme, selected: bool, hovered: bool) -> Option<ListItemTokens> {
        let p = &t.palette;
        Some(if selected {
            ListItemTokens {
                background: Some(p.primary),
                foreground: p.on_primary,
                marker: None,
                outline: true,
            }
        } else if hovered {
            ListItemTokens {
                background: Some(p.primary.with_alpha(0.7)),
                foreground: p.on_primary,
                marker: None,
                outline: false,
            }
        } else {
            ListItemTokens {
                background: None,
                foreground: p.text,
                marker: None,
                outline: false,
            }
        })
    }

    /// A 4 px bottom rule, like the kit's nav; a solid plate that never changes on scroll.
    fn app_bar(&self, t: &Theme, kind: AppBarKind, _scrolled: bool) -> Option<AppBarTokens> {
        let p = &t.palette;
        let (row, height, title) = match kind {
            AppBarKind::CenterAligned | AppBarKind::Small => (64.0, 64.0, 22.0),
            AppBarKind::Medium => (64.0, 112.0, 28.0),
            AppBarKind::Large => (64.0, 152.0, 36.0),
            AppBarKind::Bottom => (72.0, 72.0, 18.0),
        };
        Some(AppBarTokens {
            row_height: row,
            height,
            background: Some(p.surface),
            border: Some(p.border),
            title: p.text,
            leading: p.text,
            trailing: p.text,
            title_size: title,
            collapsed_title_size: 22.0,
            glow: false,
        })
    }

    /// Badges are tiny blocks in the tone colour.
    fn badge(&self, t: &Theme, tone: Tone) -> Option<SurfaceTokens> {
        let c = t.palette.tone(tone);
        Some(block(Some(c), t.on(c), t.palette.border, false))
    }

    /// A white track with a main-colour thumb block; on = main-coloured track.
    fn toggle(&self, t: &Theme, on: bool, state: ControlState) -> Option<ToggleTokens> {
        let p = &t.palette;
        if state == ControlState::Disabled {
            return None;
        }
        Some(if on {
            ToggleTokens {
                track: p.primary,
                thumb: p.surface,
                border: p.border,
            }
        } else {
            ToggleTokens {
                track: p.surface_raised,
                thumb: p.surface,
                border: p.border,
            }
        })
    }

    /// A thick outlined bar filled with the main colour.
    fn progress(&self, t: &Theme, tone: Option<Tone>) -> Option<ProgressTokens> {
        let p = &t.palette;
        Some(ProgressTokens {
            track: p.surface,
            fill: tone.map_or(p.primary, |x| p.tone(x)),
            border: p.border,
            segments: 0,
        })
    }

    fn focus_ring(&self, t: &Theme) -> Option<(Color, f32)> {
        Some((t.palette.text, 3.0))
    }

    fn tooltip(&self, t: &Theme, _rich: bool) -> Option<(Color, Color)> {
        Some((t.palette.primary, t.palette.on_primary))
    }

    /// A yellow-on-outline notification block.
    fn snackbar(&self, t: &Theme) -> Option<(SurfaceTokens, Color)> {
        let p = &t.palette;
        Some((
            block(Some(p.surface), p.text, p.border, true),
            t.legible(p.primary),
        ))
    }

    /// Alerts are solid tone blocks.
    fn alert(&self, t: &Theme, tone: Tone, banner: bool) -> Option<SurfaceTokens> {
        let c = t.palette.tone(tone);
        Some(block(Some(c), t.on(c), t.palette.border, !banner))
    }

    /// Flat colour, outline, shadow (drawn by the renderer), sink on hover.
    fn button(
        &self,
        t: &Theme,
        variant: ButtonVariant,
        state: ControlState,
    ) -> Option<ButtonTokens> {
        use ButtonVariant::*;
        let p = &t.palette;
        if state == ControlState::Disabled {
            return Some(ButtonTokens {
                background: Some(p.surface_raised),
                foreground: p.text_disabled,
                border: Some(p.text_disabled),
            });
        }
        let (bg, fg) = match variant {
            Primary => (Some(p.primary), p.on_primary),
            Secondary => (Some(p.secondary), p.on_secondary),
            Danger => (Some(p.danger), t.on(p.danger)),
            Outline => (Some(p.surface), p.text),
            OutlineSecondary => (Some(p.surface), p.text),
            Scrim => (Some(p.surface), p.text),
            Ghost => (None, p.text),
        };
        Some(ButtonTokens {
            background: bg,
            foreground: fg,
            border: Some(p.border),
        })
    }

    fn input(&self, t: &Theme, focused: bool) -> Option<InputTokens> {
        let p = &t.palette;
        Some(InputTokens {
            background: p.surface,
            border: if focused { p.primary } else { p.border },
            text: p.text,
            placeholder: p.text_disabled,
            selection: p.primary.with_alpha(0.4),
        })
    }
}

/// Gap between grid columns, wide enough for the hard shadow.
pub const GRID_GAP: f32 = 24.0;
/// Outer margin of the page grid.
pub const GRID_MARGIN: f32 = 32.0;
/// The widest the page grid grows.
pub const GRID_WIDTH_MAX: f32 = 1200.0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_variant_attaches_the_spec() {
        for t in crate::variants() {
            assert_eq!(t.spec.name(), "Neo-Brutalism", "{}", t.id());
            assert_eq!(t.meta.family, "neobrutalism");
            assert!(t.effects.hard_shadow > 0.0 && !t.effects.glow_enabled());
        }
    }

    #[test]
    fn fourteen_unique_themes() {
        let v = crate::variants();
        assert_eq!(v.len(), 14);
        let ids: std::collections::HashSet<_> = v.iter().map(|t| t.id()).collect();
        assert_eq!(ids.len(), 14);
        assert_eq!(v[0].id(), "neobrutalism");
    }

    #[test]
    fn text_on_blocks_is_legible() {
        for t in crate::variants() {
            let p = &t.palette;
            for (bg, fg) in [
                (p.primary, p.on_primary),
                (p.secondary, p.on_secondary),
                (p.background, p.text),
                (p.surface, p.text),
            ] {
                assert!(fg.contrast(bg) >= 4.5, "{} {:?} on {:?}", t.id(), fg, bg);
            }
        }
    }

    #[test]
    fn shadow_follows_the_outline() {
        for t in crate::variants() {
            assert_eq!(t.hard_shadow_color(), t.palette.border);
        }
    }

    #[test]
    fn controls_carry_an_outline() {
        for t in crate::variants() {
            for v in [
                ButtonVariant::Primary,
                ButtonVariant::Secondary,
                ButtonVariant::Danger,
            ] {
                let b = t.button(v, ControlState::Idle);
                assert_eq!(b.border, Some(t.palette.border));
            }
            assert!(t.look().sink);
        }
    }

    #[test]
    fn look_and_tooltip() {
        for t in crate::variants() {
            let l = t.look();
            assert!(l.sink && l.icons == IconSet::Bold && !l.tonal, "{}", t.id());
            assert_eq!(t.divider(), t.palette.border, "{}", t.id());
            assert_eq!(t.tooltip(false).0, t.palette.primary, "{}", t.id());
        }
    }
}
