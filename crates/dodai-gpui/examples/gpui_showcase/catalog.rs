//! Section metadata: which animation controls apply, subsections, category blurbs/icons.

/// The chart families (sidebar subsections of "Charts") and their sections, in sidebar order.
/// Every chart section is listed here exactly once; each is also its panel title.
pub(crate) const CHART_FAMILIES: [(&str, &[&str]); 9] = [
    (
        "Trends",
        &[
            "Line · area",
            "Time axis",
            "Band",
            "Stacked area",
            "Stream",
            "Horizon",
            "Sparkline",
            "Live throughput",
            "Calendar heatmap",
            "Control chart",
            "Event timeline",
        ],
    ),
    (
        "Comparison",
        &[
            "Bar",
            "Bullet",
            "Paired bars",
            "Lollipop · Dumbbell",
            "Slope",
            "Bump",
            "Radar",
            "Pareto",
            "Diverging bars",
        ],
    ),
    (
        "Part of a whole",
        &[
            "Pie",
            "Donut",
            "Treemap",
            "Sunburst",
            "Icicle",
            "Waffle",
            "Marimekko",
            "Radial bar",
            "100% stacked",
        ],
    ),
    (
        "Distribution",
        &[
            "Histogram",
            "Box plot",
            "Violin",
            "Ridgeline",
            "Beeswarm",
            "ECDF",
            "Error bars",
        ],
    ),
    (
        "Relationship",
        &[
            "Scatter",
            "Bubble",
            "Heatmap",
            "Hexbin",
            "Parallel coordinates",
            "Network graph",
        ],
    ),
    (
        "Flow",
        &["Sankey", "Chord", "Arc diagram", "Funnel", "Waterfall"],
    ),
    (
        "Finance",
        &[
            "Candlestick",
            "Renko",
            "Order book depth",
            "Secondary axis",
            "Chart stack",
        ],
    ),
    ("Planning", &["Gantt"]),
    ("Single value", &["KPI tile"]),
];

/// The chart family a chart section belongs to.
pub(crate) fn chart_family(title: &str) -> Option<&'static str> {
    CHART_FAMILIES
        .iter()
        .find(|(_, s)| s.contains(&title))
        .map(|(f, _)| *f)
}

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
        | "Shape gallery"
        | "Button crop"
        | "Roundness · Borders"
        | "Colour palette"
        | "Type scale"
        | "Motion tokens"
        | "Spacing · Elevation"
        | "Shape tokens"
        | "Frosted glass"
        | "Layout · Fluid scales"
        | "Icons"
        | "Light FX"
        | "Component textures"
        | "Container textures"
        | "Presets · Apply to"
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
        t if chart_family(t).is_some()
            || [
                "Gauges",
                "Instruments",
                "Vehicle",
                "Industrial",
                "Medical",
                "Military",
                "Aviation",
                "Signal lab",
                "Radar & telemetry",
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
        "Charts" => &[
            "Trends",
            "Comparison",
            "Part of a whole",
            "Distribution",
            "Relationship",
            "Flow",
            "Finance",
            "Planning",
            "Single value",
        ],
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
        "Charts" => chart_family(label),
        "Shaders" => Some(
            dodai_gpui::dodai_shaders::Family::ALL
                .iter()
                .find(|f| label.starts_with(f.name()))
                .map_or("Playground", |f| f.name()),
        ),
        "Shape" => Some(if label == "Shape gallery" {
            "Overview"
        } else {
            "Corners & borders"
        }),
        "Surface effects" => Some(match label {
            "Presets · Apply to" => "Start here",
            "Component textures" | "Container textures" => "Textures",
            "Light FX" | "Click ripple" => "Light & motion",
            _ => "Edges & signature",
        }),
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
        "Shape" => "Corners, borders and cuts.",
        "Surface effects" => "Theme-wide glow, plus textures, glitch, sheen and bloom.",
        "Actions" => "Buttons and triggers.",
        "Inputs" => "Text, choice, date and form controls.",
        "Navigation" => "Menus, tabs, overlays and sidebars.",
        "Data display" => "Tables, progress, media, time and content.",
        "Charts" => "Statistical, market and radial charts.",
        "Instruments" => "Vehicle, industrial and medical readouts.",
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
        "Instruments" => I::Dashboard,
        "Animation" => I::Wave,
        "Shaders" => I::Pixels,
        "Shape" => I::Cube,
        "Surface effects" => I::Texture,
        "Loading" => I::Repeat,
        _ => I::Dot,
    }
}

/// The design family that owns a section, when it shows a component only that design ships.
/// Other designs neither build nor list it. Everything not listed is shared by all designs.
pub(crate) fn owner(title: &str) -> Option<&'static [&'static str]> {
    match title {
        "Button groups · Split button" | "Loading indicator" | "Material 3 shaders" => {
            Some(&["material"])
        }
        "Instruments" | "Cyberpunk shaders" => Some(&["cyberpunk"]),
        "Frosted glass" => Some(&["dodai", "material", "neobrutalism", "cyberpunk"]),
        _ => None,
    }
}

#[cfg(test)]
mod owner_tests {
    use super::owner;

    #[test]
    fn each_design_owns_its_own_sections() {
        assert_eq!(owner("Loading indicator"), Some(&["material"][..]));
        assert_eq!(owner("Instruments"), Some(&["cyberpunk"][..]));
        assert!(owner("Frosted glass").unwrap().contains(&"material"));
        assert_eq!(owner("Buttons"), None);
    }
}
