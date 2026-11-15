//! # dodai-slint
//!
//! [Slint](https://slint.dev) adapter for the Dodai UI design system.
//!
//! * [`generate_theme_slint`] emits a `.slint` file with a `global Theme`.
//! * `ui/components.slint` ([`COMPONENTS_SLINT`]) holds reusable themed
//!   components; chamfers are drawn from [`Shape::outline`](dodai_core::Shape::outline)'s
//!   rules via `Path`.
//! * [`install_theme!`] pushes a theme (and glow intensity) into the globals at runtime.
//! * [`ThemeValues`] is the plain-data resolution all of the above use.

pub use slint;

mod generate;
mod values;

/// One frame of a surface as an SVG document; show it with
/// `slint::Image::load_from_svg_data` (see [`dodai_core::surface_svg`]).
pub use dodai_core::surface_svg;
pub use generate::{generate_theme_slint, property_names, COMPONENTS_SLINT, DODAI_SLINT};
pub use values::{
    first_family, outline, Rgba, ThemeValues, BUTTON_VARIANTS, STATES, TONES, TRANSPARENT,
};

/// Convert [`Rgba`] to a Slint colour.
pub fn color(c: Rgba) -> slint::Color {
    slint::Color::from_argb_u8(c[3], c[0], c[1], c[2])
}

/// Convert a colour list to a Slint model.
pub fn colors(v: &[Rgba]) -> slint::ModelRc<slint::Color> {
    slint::ModelRc::new(slint::VecModel::from(
        v.iter().map(|c| color(*c)).collect::<Vec<_>>(),
    ))
}

/// Push a [`Theme`](dodai_core::Theme) into the Slint `Theme` global of
/// your compiled UI. `$global` is the generated `Theme` type (exported from
/// your `.slint`), `$component` any component handle that exports it.
///
/// ```ignore
/// dodai_slint::install_theme!(Theme, &ui, &theme);
/// ```
#[macro_export]
macro_rules! install_theme {
    ($global:ty, $component:expr, $theme:expr) => {{
        let v = $crate::ThemeValues::from_theme($theme);
        let g = <$global>::get($component);
        g.on_label_case(|s| s.to_uppercase().into());
        g.set_background($crate::color(v.background));
        g.set_surface($crate::color(v.surface));
        g.set_surface_raised($crate::color(v.surface_raised));
        g.set_overlay($crate::color(v.overlay));
        g.set_border($crate::color(v.border));
        g.set_border_strong($crate::color(v.border_strong));
        g.set_text($crate::color(v.text));
        g.set_text_muted($crate::color(v.text_muted));
        g.set_text_disabled($crate::color(v.text_disabled));
        g.set_primary($crate::color(v.primary));
        g.set_on_primary($crate::color(v.on_primary));
        g.set_secondary($crate::color(v.secondary));
        g.set_danger($crate::color(v.danger));
        g.set_selection($crate::color(v.selection));
        g.set_focus_ring($crate::color(v.focus_ring));
        g.set_glow($crate::color(v.glow));
        g.set_radius(v.radius);
        g.set_chamfer(v.chamfer);
        g.set_border_width(v.border_width);
        g.set_control_height(v.control_height);
        g.set_size_xs(v.size_xs);
        g.set_size_sm(v.size_sm);
        g.set_size_md(v.size_md);
        g.set_size_lg(v.size_lg);
        g.set_size_xl(v.size_xl);
        g.set_size_xxl(v.size_xxl);
        g.set_sp_xs(v.sp_xs);
        g.set_sp_sm(v.sp_sm);
        g.set_sp_md(v.sp_md);
        g.set_sp_lg(v.sp_lg);
        g.set_sp_xl(v.sp_xl);
        g.set_glow_radius(v.glow_radius);
        g.set_hard_shadow(v.hard_shadow);
        g.set_label_tracking(v.label_tracking);
        g.set_chamfered(v.chamfered);
        g.set_cut_tl(v.cut_tl);
        g.set_cut_tr(v.cut_tr);
        g.set_cut_br(v.cut_br);
        g.set_cut_bl(v.cut_bl);
        g.set_scoop(v.scoop);
        g.set_glow_enabled(v.glow_enabled);
        g.set_scanlines(v.scanlines);
        g.set_uppercase(v.uppercase);
        g.set_font_display($crate::slint::SharedString::from(v.font_display.as_str()));
        g.set_font_body($crate::slint::SharedString::from(v.font_body.as_str()));
        g.set_font_mono($crate::slint::SharedString::from(v.font_mono.as_str()));
        g.set_button_bg($crate::colors(&v.button_bg));
        g.set_button_fg($crate::colors(&v.button_fg));
        g.set_button_border($crate::colors(&v.button_border));
        g.set_tag_bg($crate::colors(&v.tag_bg));
        g.set_tag_fg($crate::colors(&v.tag_fg));
        g.set_tag_border($crate::colors(&v.tag_border));
        g.set_tab_bg($crate::colors(&v.tab_bg));
        g.set_tab_fg($crate::colors(&v.tab_fg));
        g.set_tab_indicator($crate::colors(&v.tab_indicator));
        g.set_toggle_track($crate::colors(&v.toggle_track));
        g.set_toggle_thumb($crate::colors(&v.toggle_thumb));
        g.set_toggle_border($crate::colors(&v.toggle_border));
        g.set_progress_track($crate::colors(&v.progress_track));
        g.set_progress_fill($crate::colors(&v.progress_fill));
        g.set_progress_border($crate::colors(&v.progress_border));
        g.set_list_bg($crate::colors(&v.list_bg));
        g.set_list_fg($crate::colors(&v.list_fg));
        g.set_list_marker($crate::colors(&v.list_marker));
        g.set_panel_bg($crate::colors(&v.panel_bg));
        g.set_panel_border($crate::colors(&v.panel_border));
        g.set_panel_title($crate::colors(&v.panel_title));
        g.set_input_bg($crate::colors(&v.input_bg));
        g.set_input_border($crate::colors(&v.input_border));
        g.set_input_text($crate::color(v.input_text));
        g.set_input_placeholder($crate::color(v.input_placeholder));
    }};
}
