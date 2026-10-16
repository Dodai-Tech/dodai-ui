//! Applying a [`Theme`] to an egui [`Context`].

use crate::paint::color32;
use dodai_core::{Mode, ShapeKind, Theme};
use egui::{
    epaint::Shadow, Context, CornerRadius, FontId, Id, Stroke, Style, TextStyle, Vec2, Visuals,
};
use std::sync::Arc;

fn store_id() -> Id {
    Id::new("dodai-egui/theme")
}

/// Build egui [`Visuals`] from tokens.
pub fn visuals(t: &Theme) -> Visuals {
    let p = &t.palette;
    let mut v = if t.meta.mode == Mode::Dark {
        Visuals::dark()
    } else {
        Visuals::light()
    };
    let cr = if t.shape.kind() == ShapeKind::Rounded {
        CornerRadius::same(t.shape.radius.round().clamp(0.0, 255.0) as u8)
    } else {
        CornerRadius::ZERO
    };
    let bw = t.shape.border_width;
    v.panel_fill = color32(p.background);
    v.window_fill = color32(p.surface);
    v.extreme_bg_color = color32(p.background);
    v.faint_bg_color = color32(p.surface_raised);
    v.code_bg_color = color32(p.surface_raised);
    v.window_stroke = Stroke::new(bw, color32(p.border_strong));
    v.window_corner_radius = cr;
    v.menu_corner_radius = cr;
    v.window_shadow = Shadow::NONE;
    v.popup_shadow = Shadow::NONE;
    v.hyperlink_color = color32(p.link);
    v.override_text_color = None;
    v.selection.bg_fill = color32(p.selection);
    v.selection.stroke = Stroke::new(bw, color32(p.primary));
    let w = &mut v.widgets;
    for (wv, bg, fg, stroke) in [
        (&mut w.noninteractive, p.surface, p.text_muted, p.border),
        (&mut w.inactive, p.surface_raised, p.text, p.border),
        (&mut w.hovered, p.surface_raised, p.text, p.border_strong),
        (&mut w.active, p.primary, p.on_primary, p.primary),
        (&mut w.open, p.surface_raised, p.text, p.border_strong),
    ] {
        wv.bg_fill = color32(bg);
        wv.weak_bg_fill = color32(bg);
        wv.bg_stroke = Stroke::new(bw, color32(stroke));
        wv.fg_stroke = Stroke::new(1.0_f32, color32(fg));
        wv.corner_radius = cr;
        wv.expansion = 0.0;
    }
    v
}

/// Apply spacing and type sizes from tokens to a [`Style`].
pub fn apply_to_style(t: &Theme, style: &mut Style) {
    let (s, ty) = (&t.spacing, &t.typography);
    style.spacing.item_spacing = Vec2::splat(s.sm);
    style.spacing.button_padding = Vec2::new(s.lg, s.sm);
    style.spacing.interact_size.y = t.shape.control_height;
    style.spacing.window_margin = egui::Margin::same(s.md.round() as i8);
    style.spacing.menu_margin = egui::Margin::same(s.sm.round() as i8);
    style
        .text_styles
        .insert(TextStyle::Small, FontId::proportional(ty.size_xs));
    style
        .text_styles
        .insert(TextStyle::Body, FontId::proportional(ty.size_md));
    style
        .text_styles
        .insert(TextStyle::Button, FontId::proportional(ty.size_md));
    style
        .text_styles
        .insert(TextStyle::Heading, FontId::proportional(ty.size_xl));
    style
        .text_styles
        .insert(TextStyle::Monospace, FontId::monospace(ty.size_sm));
}

/// Install `theme` into `ctx`: visuals, spacing, type sizes, and the theme the
/// widgets read via [`active_theme`]. Cheap to call every frame (no-op when the
/// theme is unchanged), so glow-intensity changes just work.
pub fn install(ctx: &Context, theme: &Theme) {
    if let Some(cur) = ctx.data(|d| d.get_temp::<Arc<Theme>>(store_id())) {
        if *cur == *theme {
            return;
        }
    }
    ctx.set_visuals(visuals(theme));
    ctx.style_mut(|s| apply_to_style(theme, s));
    ctx.data_mut(|d| d.insert_temp(store_id(), Arc::new(theme.clone())));
    ctx.request_repaint();
}

/// Theme set by [`install`] (default theme if none installed yet).
pub fn active_theme(ctx: &Context) -> Arc<Theme> {
    ctx.data(|d| d.get_temp::<Arc<Theme>>(store_id()))
        .unwrap_or_else(|| Arc::new(Theme::default()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{chamfered, rounded, square};

    #[test]
    fn visuals_map_tokens() {
        for t in [chamfered(), rounded(), square()] {
            let v = visuals(&t);
            assert_eq!(v.panel_fill, color32(t.palette.background));
            assert_eq!(v.widgets.active.bg_fill, color32(t.palette.primary));
            assert_eq!(v.dark_mode, t.is_dark());
        }
    }

    #[test]
    fn corner_radius_follows_shape_kind() {
        assert!(visuals(&rounded()).widgets.inactive.corner_radius.nw > 0);
        assert_eq!(visuals(&square()).widgets.inactive.corner_radius.nw, 0);
        assert_eq!(visuals(&chamfered()).widgets.inactive.corner_radius.nw, 0);
    }

    #[test]
    fn style_uses_spacing_and_height() {
        let t = rounded();
        let mut s = Style::default();
        apply_to_style(&t, &mut s);
        assert_eq!(s.spacing.item_spacing.x, t.spacing.sm);
        assert_eq!(s.spacing.interact_size.y, t.shape.control_height);
        assert_eq!(
            s.text_styles[&TextStyle::Heading].size,
            t.typography.size_xl
        );
    }

    #[test]
    fn install_stores_theme() {
        let ctx = Context::default();
        let t = rounded().with_glow_intensity(1.5);
        install(&ctx, &t);
        assert_eq!(*active_theme(&ctx), t);
    }
}
