//! Section metadata: which animation controls apply, subsections, category blurbs/icons.

/// Which animation controls apply to a section (derived from the components it shows).
#[derive(Clone, Copy)]
pub(crate) struct Caps {
    /// Entry effects (fade / rise / drop / slide): any component wrapped by `motion::entry`.
    pub(crate) entry: bool,
    /// Value-in "Fill": charts, gauges, progress, sliders, knobs.
    pub(crate) fill: bool,
    /// Stagger: needs several animated components.
    pub(crate) stagger: bool,
}

pub(crate) fn caps(title: &str) -> Caps {
    let c = |entry, fill, stagger| Caps {
        entry,
        fill,
        stagger,
    };
    match title {
        // Shaders animate themselves; no entry/fill/stagger controls.
        t if t == "Shader playground"
            || t.ends_with(" shaders")
            || t.ends_with(" effects")
            || t.ends_with(" presets") =>
        {
            c(false, false, false)
        }
        // Raw / non-animated components only.
        "Tooltip"
        | "Loading indicator"
        | "Dialog · Bottom sheet · Side sheet"
        | "Button groups · Split button"
        | "Dynamics · physics models"
        | "Dynamics · one-shot mixins"
        | "Input"
        | "Collapsible sidebar"
        | "Border"
        | "Colour palette"
        | "Type scale"
        | "Motion tokens"
        | "Spacing · Elevation"
        | "Shape tokens"
        | "Icons"
        | "Icons Material"
        | "Glow · Scanlines"
        | "Password · OTP"
        | "Form field"
        | "Toggle group · Checkbox group"
        | "Date range"
        | "Data table"
        | "Code block · Markdown"
        | "Image · Avatar group"
        | "Circular progress · Banner"
        | "Clock · Timer"
        | "Stopwatch · Tap tempo"
        | "Scroll area"
        | "Popover · Hover card · Context menu · Dropdown" => c(false, false, false),
        // Single animated component.
        "Search view" | "Tag field" => c(true, false, false),
        // Value-in components.
        "Slider · Progress" | "Slider (stepped, ticks, value readout)" | "Range slider" => {
            c(true, true, true)
        }
        "Knob · Vertical sliders" => c(true, true, true),
        t if t.contains("chart")
            || [
                "Pie",
                "Donut",
                "Gauges",
                "Sparkline",
                "Instruments",
                "Area · histogram · waterfall",
                "Funnel · treemap · radial bar",
                "Candlestick · box plot · bubble",
                "Radar · scatter · heatmap",
            ]
            .contains(&t) =>
        {
            c(true, true, true)
        }
        _ => c(true, false, true),
    }
}

/// Subsections of a category (shown in the sidebar under the selected category).
pub(crate) fn subs(cat: &str) -> &'static [&'static str] {
    match cat {
        "Animation" => &["Entry", "Physics", "Dynamics"],
        "Shaders" => &[
            "Playground",
            "Cyberpunk",
            "Material 3",
            "Matrix",
            "Retro terminals",
            "Pixel art",
            "Cosmic & nature",
            "Ghostty",
            "Ghibli",
            "DMT Hyperspace",
        ],
        _ => &[],
    }
}

/// Subsection a section belongs to.
pub(crate) fn sub_of(cat: &str, label: &str) -> Option<&'static str> {
    match cat {
        "Animation" => Some(if label.starts_with("Dynamics") {
            "Dynamics"
        } else if ["Spring", "Bounce", "Elastic", "Back"]
            .iter()
            .any(|p| label.starts_with(p))
        {
            "Physics"
        } else {
            "Entry"
        }),
        "Shaders" => Some(
            dodai_gpui::dodai_shaders::Family::ALL
                .iter()
                .find(|f| label.starts_with(f.name()))
                .map_or("Playground", |f| f.name()),
        ),
        _ => None,
    }
}

pub(crate) fn section_id(title: &str) -> u32 {
    title
        .bytes()
        .fold(0u32, |a, b| a.wrapping_mul(31).wrapping_add(b as u32))
}

pub(crate) fn cat_blurb(name: &str) -> &'static str {
    match name {
        "Foundations" => "Design tokens: colour, type, spacing.",
        "Shape" => "Glow, scanlines, corner cuts and borders.",
        "Actions" => "Buttons and triggers.",
        "Inputs" => "Text, choice, date and form controls.",
        "Navigation" => "Menus, tabs, overlays and sidebars.",
        "Data display" => "Tables, progress, media, time and content.",
        "Charts" => "Statistical, market and radial charts.",
        "Animation" => "Entry effects and physics.",
        "Shaders" => "Pixel-shader backgrounds and effects.",
        "Loading" => "Spinners and skeletons.",
        _ => "",
    }
}

pub(crate) fn cat_icon(name: &str) -> dodai_gpui::IconName {
    use dodai_gpui::IconName as I;
    match name {
        "All" => I::Grid,
        "Foundations" => I::Layers,
        "Inputs" => I::Sliders,
        "Actions" => I::Play,
        "Navigation" => I::Menu,
        "Data display" => I::Table,
        "Charts" => I::BarChart,
        "Animation" => I::Wave,
        "Shaders" => I::Pixels,
        "Shape" => I::Cube,
        "Loading" => I::Repeat,
        _ => I::Dot,
    }
}
