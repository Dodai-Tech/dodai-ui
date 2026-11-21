//! Emits the `.slint` source of the `Theme` global from a [`Theme`].

use crate::values::{Rgba, ThemeValues};
use dodai_core::Theme;

enum Prop<'a> {
    Color(Rgba),
    Num(f32),
    Float(f32),
    Bool(bool),
    Str(&'a str),
    Colors(&'a [Rgba]),
}

fn lit(c: Rgba) -> String {
    format!("#{:02x}{:02x}{:02x}{:02x}", c[0], c[1], c[2], c[3])
}

impl Prop<'_> {
    fn ty(&self) -> &'static str {
        match self {
            Prop::Color(_) => "color",
            Prop::Num(_) => "length",
            Prop::Float(_) => "float",
            Prop::Bool(_) => "bool",
            Prop::Str(_) => "string",
            Prop::Colors(_) => "[color]",
        }
    }
    fn value(&self) -> String {
        match self {
            Prop::Color(c) => lit(*c),
            Prop::Num(n) => format!("{n}px"),
            Prop::Float(n) => format!("{n}"),
            Prop::Bool(b) => b.to_string(),
            Prop::Str(s) => format!("{s:?}"),
            Prop::Colors(v) => format!(
                "[{}]",
                v.iter().map(|c| lit(*c)).collect::<Vec<_>>().join(", ")
            ),
        }
    }
}

impl ThemeValues {
    fn props(&self) -> Vec<(&'static str, Prop<'_>)> {
        let p = |n, v| (n, v);
        vec![
            p("background", Prop::Color(self.background)),
            p("surface", Prop::Color(self.surface)),
            p("surface-raised", Prop::Color(self.surface_raised)),
            p("overlay", Prop::Color(self.overlay)),
            p("border", Prop::Color(self.border)),
            p("border-strong", Prop::Color(self.border_strong)),
            p("text", Prop::Color(self.text)),
            p("text-muted", Prop::Color(self.text_muted)),
            p("text-disabled", Prop::Color(self.text_disabled)),
            p("primary", Prop::Color(self.primary)),
            p("on-primary", Prop::Color(self.on_primary)),
            p("secondary", Prop::Color(self.secondary)),
            p("danger", Prop::Color(self.danger)),
            p("selection", Prop::Color(self.selection)),
            p("focus-ring", Prop::Color(self.focus_ring)),
            p("glow", Prop::Color(self.glow)),
            p("radius", Prop::Num(self.radius)),
            p("chamfer", Prop::Num(self.chamfer)),
            p("border-width", Prop::Num(self.border_width)),
            p("control-height", Prop::Num(self.control_height)),
            p("size-xs", Prop::Num(self.size_xs)),
            p("size-sm", Prop::Num(self.size_sm)),
            p("size-md", Prop::Num(self.size_md)),
            p("size-lg", Prop::Num(self.size_lg)),
            p("size-xl", Prop::Num(self.size_xl)),
            p("size-xxl", Prop::Num(self.size_xxl)),
            p("sp-xs", Prop::Num(self.sp_xs)),
            p("sp-sm", Prop::Num(self.sp_sm)),
            p("sp-md", Prop::Num(self.sp_md)),
            p("sp-lg", Prop::Num(self.sp_lg)),
            p("sp-xl", Prop::Num(self.sp_xl)),
            p("glow-radius", Prop::Num(self.glow_radius)),
            p("hard-shadow", Prop::Num(self.hard_shadow)),
            p("label-tracking", Prop::Float(self.label_tracking)),
            p("chamfered", Prop::Bool(self.chamfered)),
            p("cut-tl", Prop::Bool(self.cut_tl)),
            p("cut-tr", Prop::Bool(self.cut_tr)),
            p("cut-br", Prop::Bool(self.cut_br)),
            p("cut-bl", Prop::Bool(self.cut_bl)),
            p("scoop", Prop::Bool(self.scoop)),
            p("glow-enabled", Prop::Bool(self.glow_enabled)),
            p("scanlines", Prop::Bool(self.scanlines)),
            p("uppercase", Prop::Bool(self.uppercase)),
            p("font-display", Prop::Str(&self.font_display)),
            p("font-body", Prop::Str(&self.font_body)),
            p("font-mono", Prop::Str(&self.font_mono)),
            p("button-bg", Prop::Colors(&self.button_bg)),
            p("button-fg", Prop::Colors(&self.button_fg)),
            p("button-border", Prop::Colors(&self.button_border)),
            p("tag-bg", Prop::Colors(&self.tag_bg)),
            p("tag-fg", Prop::Colors(&self.tag_fg)),
            p("tag-border", Prop::Colors(&self.tag_border)),
            p("tab-bg", Prop::Colors(&self.tab_bg)),
            p("tab-fg", Prop::Colors(&self.tab_fg)),
            p("tab-indicator", Prop::Colors(&self.tab_indicator)),
            p("toggle-track", Prop::Colors(&self.toggle_track)),
            p("toggle-thumb", Prop::Colors(&self.toggle_thumb)),
            p("toggle-border", Prop::Colors(&self.toggle_border)),
            p("progress-track", Prop::Colors(&self.progress_track)),
            p("progress-fill", Prop::Colors(&self.progress_fill)),
            p("progress-border", Prop::Colors(&self.progress_border)),
            p("list-bg", Prop::Colors(&self.list_bg)),
            p("list-fg", Prop::Colors(&self.list_fg)),
            p("list-marker", Prop::Colors(&self.list_marker)),
            p("panel-bg", Prop::Colors(&self.panel_bg)),
            p("panel-border", Prop::Colors(&self.panel_border)),
            p("panel-title", Prop::Colors(&self.panel_title)),
            p("input-bg", Prop::Colors(&self.input_bg)),
            p("input-border", Prop::Colors(&self.input_border)),
            p("input-text", Prop::Color(self.input_text)),
            p("input-placeholder", Prop::Color(self.input_placeholder)),
        ]
    }
}

/// Source of `theme.slint`: a `global Theme` whose defaults are `theme`'s
/// resolved tokens. Write it next to `components.slint` (see the crate README)
/// and push later theme changes at runtime with [`install_theme!`](crate::install_theme).
pub fn generate_theme_slint(theme: &Theme) -> String {
    let v = ThemeValues::from_theme(theme);
    let mut s = format!(
        "// Generated by dodai-slint from theme `{}`. Do not edit.\n\nexport global Theme {{\n",
        theme.id()
    );
    for (name, prop) in v.props() {
        s.push_str(&format!(
            "    in-out property <{}> {}: {};\n",
            prop.ty(),
            name,
            prop.value()
        ));
    }
    s.push_str(
        "    /// The design's canvas mesh, drawn by `LBackdrop`; set by `install_theme!`.\n    in-out property <image> backdrop;\n    in-out property <bool> has-backdrop: false;\n",
    );
    s.push_str(
        "    /// Implemented by `install_theme!` (upper-cases); used when `uppercase` is set.\n    pure callback label-case(string) -> string;\n}\n",
    );
    s
}

/// The reusable component library (imports `theme.slint`).
pub const COMPONENTS_SLINT: &str = include_str!("../ui/components.slint");
/// Entry point re-exporting `Theme` and all components.
pub const DODAI_SLINT: &str = include_str!("../ui/dodai.slint");

/// Names of all generated `Theme` properties (kebab-case).
pub fn property_names() -> Vec<&'static str> {
    ThemeValues::from_theme(&Theme::default())
        .props()
        .into_iter()
        .map(|(n, _)| n)
        .collect()
}
