//! Section metadata: which animation controls apply, subsections, category blurbs/icons.

/// The chart families (sidebar subsections of "Charts") and their sections, in sidebar order:
/// one section per chart, then "Examples", whole dashboards composed from those charts.
/// Every chart section is listed here exactly once; each is also its panel title.
pub(crate) const CHART_FAMILIES: [(&str, &[&str]); 11] = [
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
            "Calendar heatmap",
            "Control chart",
            "Event timeline",
            "Span fan",
            "Peak range",
            "Bubble timeline",
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
            "Circle packing",
            "Proportional squares",
            "Waffle",
            "Marimekko",
            "Radial bar",
            "100% stacked",
            "Shard map",
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
            "Bubble swarm",
            "ECDF",
            "Error bars",
        ],
    ),
    (
        "Relationship",
        &[
            "Scatter",
            "Heatmap",
            "Hexbin",
            "Parallel coordinates",
            "Network graph",
            "Time scatter",
        ],
    ),
    (
        "Flow",
        &[
            "Sankey",
            "Split flow",
            "Chord",
            "Arc diagram",
            "Funnel",
            "Waterfall",
        ],
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
    (
        "Status",
        &[
            "Slot grid",
            "Status strip",
            "Bucket bar",
            "Stage bars",
            "Gradient scale",
        ],
    ),
    ("Single value", &["KPI tile"]),
    ("Examples", &["Live throughput", "Network overview"]),
];

/// The chart family names, in sidebar order (from [`CHART_FAMILIES`]).
const CHART_FAMILY_NAMES: [&str; CHART_FAMILIES.len()] = {
    let mut out = [""; CHART_FAMILIES.len()];
    let mut i = 0;
    while i < out.len() {
        out[i] = CHART_FAMILIES[i].0;
        i += 1;
    }
    out
};

/// Extra words the component filter matches for a section, beyond its category and title: what
/// its demo shows and what the chart is often called elsewhere.
const KEYWORDS: [(&str, &str); 7] = [
    (
        "Span fan",
        "empires history lifespan durations radial reigns",
    ),
    (
        "Bubble timeline",
        "breaches hacks events eruptions volcanoes bubbles over time packed",
    ),
    (
        "Time scatter",
        "milestones models releases bridges records frontier bubbles",
    ),
    (
        "Split flow",
        "plastic waste where it goes cascade tree flow",
    ),
    ("Shard map", "voronoi treemap population languages"),
    ("Peak range", "mountains molehills peaks pollen seasons"),
    ("Ridgeline", "joyplot distributions"),
];

/// The extra filter words of section `title` (empty if none).
pub(crate) fn keywords(title: &str) -> &'static str {
    KEYWORDS
        .iter()
        .find(|(t, _)| *t == title)
        .map_or("", |(_, k)| k)
}

/// Widest a chart demo tile grows: past it, a chart of fixed height reads as stretched (a
/// handful of points miles apart).
pub(crate) const CHART_TILE_MAX_W: f32 = 760.0;

/// Chart sections whose demo is wide by nature (long time spans, many columns, rows of tiles):
/// their tiles fill the card.
pub(crate) const WIDE_CHARTS: [&str; 17] = [
    "Span fan",
    "Bubble timeline",
    "Peak range",
    "Shard map",
    "Split flow",
    "Time scatter",
    "Slot grid",
    "Network overview",
    "Live throughput",
    "Calendar heatmap",
    "Horizon",
    "Event timeline",
    "Heatmap",
    "Sankey",
    "Gantt",
    "Chart stack",
    "KPI tile",
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
        "Charts" => &CHART_FAMILY_NAMES,
        // By what a shader is for, not by whose look it has: the family stays in each title.
        "Shaders" => &["Playground", "Sources", "Effects", "Presets"],
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
        "Shaders" => Some(if label.ends_with(" shaders") {
            "Sources"
        } else if label.ends_with(" effects") {
            "Effects"
        } else if label.ends_with(" presets") {
            "Presets"
        } else {
            "Playground"
        }),
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
