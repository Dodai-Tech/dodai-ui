//! Server-side HTML for the `.dodai-*` components: plain functions that return markup, for pages
//! built without a UI framework (see the `topcoat_showcase` example).
//!
//! Every function takes a [`Cx`] (the theme and the app-wide surface mix) where the look depends
//! on the design, and returns a `String` to put inside a page that carries the stylesheet of
//! [`multi_theme_stylesheet`](crate::multi_theme_stylesheet). Text is escaped by the functions
//! that take it; markup arguments (`inner`) are inserted as they are. A pressable element takes
//! `attrs`, a raw attribute string: the page's script decides what they mean (the example uses
//! `data-on` and `data-a`).

use dodai_core::{
    arc_points, busy, knob_angle,
    orientation::{place, AxisId, AxisTone, Basis, GizmoLayout},
    ButtonCrop, ButtonVariant, CardVariant, ControlSize, CropEdge, Kit, KnobGeometry, Palette,
    SpinnerStyle, StepState, Surface, SurfaceClass, SurfaceFx, Theme, Tone,
};
use dodai_icons::IconId;

/// What the components read: the theme they draw in and the app-wide surface mix.
#[derive(Clone, Copy)]
pub struct Cx<'a> {
    /// The theme in use.
    pub theme: &'a Theme,
    /// The app-wide surface mix (containers it frosts turn see-through).
    pub fx: SurfaceFx,
}

/// `s` as HTML text or attribute value.
pub fn esc(s: &str) -> String {
    let mut o = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => o.push_str("&amp;"),
            '<' => o.push_str("&lt;"),
            '>' => o.push_str("&gt;"),
            '"' => o.push_str("&quot;"),
            _ => o.push(c),
        }
    }
    o
}

/// The `dodai-glass` inline style of a `class` surface under the mix with the component's own
/// `over`; `None` when the fill stays as designed.
pub fn glass(cx: &Cx, class: Option<SurfaceClass>, over: &Surface) -> Option<String> {
    if !over.is_set() && class.is_none() {
        return None;
    }
    let s = crate::glass_style(
        cx.theme,
        class.unwrap_or(SurfaceClass::Button),
        over,
        if class.is_some() {
            cx.fx
        } else {
            SurfaceFx::NONE
        },
    );
    (!s.is_empty()).then_some(s)
}

/// An icon as inline SVG in the current colour.
pub fn icon(cx: &Cx, id: IconId, size: u32, filled: bool) -> String {
    match dodai_icons::glyph_for_theme(id, cx.theme, filled) {
        Some(g) => format!(
            "<span class=\"dodai-icon\" style=\"display:inline-flex;line-height:0\">{}</span>",
            g.svg(size, "currentColor")
        ),
        None => String::new(),
    }
}

/// The classes of a button.
pub fn button_classes(
    variant: ButtonVariant,
    size: Option<ControlSize>,
    square: bool,
    icon_only: bool,
) -> String {
    let mut c = String::from(match variant {
        ButtonVariant::Primary => "dodai-btn dodai-btn--primary",
        ButtonVariant::Secondary => "dodai-btn dodai-btn--secondary",
        ButtonVariant::Outline => "dodai-btn dodai-btn--outline",
        ButtonVariant::OutlineSecondary => "dodai-btn dodai-btn--outline-secondary",
        ButtonVariant::Danger => "dodai-btn dodai-btn--danger",
        ButtonVariant::Ghost => "dodai-btn dodai-btn--ghost",
        ButtonVariant::Scrim => "dodai-btn dodai-btn--scrim",
    });
    if let Some(z) = size {
        c.push_str(match z {
            ControlSize::XSmall => " dodai-btn--xs",
            ControlSize::Small => " dodai-btn--sm",
            ControlSize::Medium => " dodai-btn--md",
            ControlSize::Large => " dodai-btn--lg",
            ControlSize::XLarge => " dodai-btn--xl",
        });
    }
    if square {
        c.push_str(" dodai-btn--square");
    }
    if icon_only {
        c.push_str(" dodai-btn--icon");
    }
    c
}

/// How a button looks and behaves.
#[derive(Clone, Copy, Default)]
pub struct Btn {
    /// Fill and border.
    pub variant: Option<ButtonVariant>,
    /// Height class.
    pub size: Option<ControlSize>,
    /// Square corners.
    pub square: bool,
    /// Only an icon inside.
    pub icon_only: bool,
    /// Shows a spinner and does not react.
    pub loading: bool,
    /// Does not react.
    pub disabled: bool,
    /// A sheen runs over it.
    pub shimmer: bool,
    /// One end cut off, by the design's crop.
    pub crop: Option<ButtonCrop>,
}

/// A button (`inner` is its content markup).
pub fn button(cx: &Cx, b: &Btn, over: &Surface, attrs: &str, inner: &str) -> String {
    let mut class = button_classes(
        b.variant.unwrap_or(ButtonVariant::Primary),
        b.size,
        b.square,
        b.icon_only,
    );
    if b.shimmer {
        class.push_str(" dodai-btn--shimmer");
    }
    let mut style = String::new();
    let mut tick = None;
    if let Some(crop) = b.crop {
        style = crate::crop_style(cx.theme, crop);
        if let ButtonCrop::CutTicked { tick: edge, .. } = crop {
            let (side, cls) = match edge {
                CropEdge::Left => ("left", "dodai-tick dodai-tick--left"),
                _ => ("right", "dodai-tick dodai-tick--right"),
            };
            if !b.icon_only {
                style.push_str(&format!(
                    "padding-{side}:calc(var(--dodai-space-lg) + var(--dodai-space-md));"
                ));
            }
            tick = Some(cls);
        }
    }
    if let Some(g) = glass(cx, Some(SurfaceClass::Button), over) {
        class.push_str(" dodai-glass");
        style.push_str(&g);
    }
    let off = if b.disabled || b.loading {
        " disabled"
    } else {
        ""
    };
    let spin = if b.loading {
        "<span class=\"dodai-spinner\" style=\"--s:14px\"></span>"
    } else {
        ""
    };
    let tick = tick.map_or(String::new(), |c| format!("<i class=\"{c}\"></i>"));
    format!("<button class=\"{class}\" style=\"{style}\"{off} {attrs}>{spin}{inner}{tick}</button>")
}

/// The classes a kit box carries.
pub fn kit_class(kit: Kit) -> String {
    let mut c = String::from("dodai-kit");
    match kit {
        Kit::Card(v) => {
            c.push_str(" dodai-kit--card");
            if v == CardVariant::Elevated {
                c.push_str(" dodai-kit--elevated");
            }
        }
        Kit::Popup => c.push_str(" dodai-kit--popup"),
        Kit::Dot(_) => c.push_str(" dodai-kit--dot"),
        _ => {}
    }
    c
}

/// A box that looks like `kit`; with `press` it is a button and hovers.
pub fn kit_box(
    cx: &Cx,
    kit: Kit,
    style: &str,
    over: &Surface,
    press: bool,
    attrs: &str,
    inner: &str,
) -> String {
    let mut style = format!("{}{style}", crate::kit_style(kit));
    let mut class = kit_class(kit);
    if let Some(g) = glass(cx, kit.surface_class(), over) {
        class.push_str(" dodai-glass");
        style.push_str(&g);
    }
    if press {
        format!(
            "<button class=\"{class} dodai-kit--press\" style=\"{style}\" {attrs}>{inner}</button>"
        )
    } else {
        format!("<div class=\"{class}\" style=\"{style}\" {attrs}>{inner}</div>")
    }
}

/// A panel with an optional title.
pub fn panel(
    cx: &Cx,
    title: Option<&str>,
    focus: bool,
    raised: bool,
    over: &Surface,
    style: &str,
    inner: &str,
) -> String {
    let mut class = String::from("dodai-panel");
    if focus {
        class.push_str(" dodai-panel--focus");
    }
    if raised {
        class.push_str(" dodai-panel--raised");
    }
    let mut style = style.to_string();
    if let Some(g) = glass(cx, Some(SurfaceClass::Panel), over) {
        class.push_str(" dodai-glass");
        style.push_str(&g);
    }
    let title = title.map_or(String::new(), |t| {
        format!("<div class=\"dodai-panel__title\">{}</div>", esc(t))
    });
    format!(
        "<div class=\"{class}\" style=\"{style}\"><span class=\"dodai-panel__shade\" aria-hidden=\"true\"></span>{title}{inner}</div>"
    )
}

/// The tag colour class of a status tone.
pub fn tag_class(tone: Tone) -> &'static str {
    match tone {
        Tone::Info => "",
        Tone::Success => "dodai-tag--success",
        Tone::Warning => "dodai-tag--warning",
        Tone::Danger => "dodai-tag--danger",
    }
}

/// A tag.
pub fn tag(tone: Tone, soft: bool, text: &str) -> String {
    let soft = if soft { "dodai-tag--soft" } else { "" };
    format!(
        "<span class=\"dodai-tag {} {soft}\">{}</span>",
        tag_class(tone),
        esc(text)
    )
}

/// Tabs; `attrs(i)` are the attributes of tab `i`.
pub fn tabs(
    cx: &Cx,
    labels: &[String],
    active: usize,
    secondary: bool,
    disabled: &[usize],
    icons: &[IconId],
    attrs: &dyn Fn(usize) -> String,
) -> String {
    let mut o = format!(
        "<div class=\"{}\" role=\"tablist\">",
        if secondary {
            "dodai-tabs dodai-tabs--secondary"
        } else {
            "dodai-tabs"
        }
    );
    for (i, t) in labels.iter().enumerate() {
        let mut class = String::from("dodai-tab");
        if i == active {
            class.push_str(" dodai-tab--active");
        }
        if disabled.contains(&i) {
            class.push_str(" dodai-tab--disabled");
        }
        if icons.get(i).is_some() {
            class.push_str(" dodai-tab--icon");
        }
        let ic = icons
            .get(i)
            .map_or(String::new(), |ic| icon(cx, *ic, 18, false));
        o.push_str(&format!(
            "<a class=\"{class}\" role=\"tab\" aria-selected=\"{}\" aria-disabled=\"{}\" href=\"#\" {}>{ic}<span>{}</span></a>",
            i == active,
            disabled.contains(&i),
            attrs(i),
            esc(t)
        ));
    }
    o.push_str("</div>");
    o
}

/// A switch.
pub fn switch(checked: bool, disabled: bool, label: &str, attrs: &str) -> String {
    format!(
        "<label class=\"dodai-switch\"><input type=\"checkbox\" role=\"switch\"{}{} {attrs}><span class=\"dodai-switch__box\"></span>{}</label>",
        if checked { " checked" } else { "" },
        if disabled { " disabled" } else { "" },
        esc(label)
    )
}

/// A checkbox (`mixed`: the dash state).
pub fn checkbox(checked: bool, disabled: bool, mixed: bool, label: &str, attrs: &str) -> String {
    format!(
        "<label class=\"{}\"><input type=\"checkbox\"{}{} {attrs}><span class=\"dodai-check__box\"></span>{}</label>",
        if mixed {
            "dodai-check dodai-check--mixed"
        } else {
            "dodai-check"
        },
        if checked { " checked" } else { "" },
        if disabled { " disabled" } else { "" },
        esc(label)
    )
}

/// A radio button of the group `name`.
pub fn radio(name: &str, checked: bool, disabled: bool, label: &str, attrs: &str) -> String {
    format!(
        "<label class=\"dodai-radio\"><input type=\"radio\" name=\"{}\"{}{} {attrs}><span class=\"dodai-radio__box\"></span>{}</label>",
        esc(name),
        if checked { " checked" } else { "" },
        if disabled { " disabled" } else { "" },
        esc(label)
    )
}

/// A progress bar (`value` 0..=1).
pub fn progress(value: f32, tone: Option<Tone>) -> String {
    let pct = value.clamp(0.0, 1.0) * 100.0;
    let tone = match tone {
        Some(Tone::Success) => "dodai-progress--success",
        Some(Tone::Warning) => "dodai-progress--warning",
        Some(Tone::Danger) => "dodai-progress--danger",
        _ => "",
    };
    format!(
        "<div class=\"dodai-progress {tone}\" role=\"progressbar\" aria-valuenow=\"{pct:.0}\"><span style=\"--v:{pct}%\"></span></div>"
    )
}

/// What a slider shows.
#[derive(Clone, Default)]
pub struct SliderOpts {
    /// Lowest value.
    pub min: f32,
    /// Highest value.
    pub max: f32,
    /// Step of the input.
    pub step: f32,
    /// Does not react.
    pub disabled: bool,
    /// Stands up.
    pub vertical: bool,
    /// Height of a vertical slider, px.
    pub length: u32,
    /// Marks at every step.
    pub ticks: bool,
    /// Fills from the middle.
    pub centered: bool,
    /// A value bubble over (or under) the thumb.
    pub bubble: Option<String>,
    /// The bubble goes under the track.
    pub bubble_below: bool,
}

/// A slider at `value`.
pub fn slider(value: f32, o: &SliderOpts, attrs: &str) -> String {
    let span = (o.max - o.min).max(f32::EPSILON);
    let pct = ((value - o.min) / span).clamp(0.0, 1.0) * 100.0;
    let (a, v) = if o.centered {
        (pct.min(50.0), pct.max(50.0))
    } else {
        (0.0, pct)
    };
    let (class, style) = if o.vertical {
        (
            "dodai-slider dodai-slider--v",
            format!("--a:{a}%;--v:{v}%;height:{}px", o.length),
        )
    } else {
        ("dodai-slider", format!("--a:{a}%;--v:{v}%"))
    };
    let input = format!(
        "<input class=\"{class}\" type=\"range\" min=\"{}\" max=\"{}\" step=\"{}\" value=\"{value}\"{} style=\"{style}\" {attrs}>",
        o.min,
        o.max,
        o.step,
        if o.disabled { " disabled" } else { "" }
    );
    if o.vertical {
        return input;
    }
    let at = |f: f32| format!("left:calc(8px + {f} * (100% - 16px))");
    let mut marks = String::new();
    if o.ticks {
        for f in dodai_core::slider_math::tick_fractions(o.min, o.max, Some(o.step)) {
            marks.push_str(&format!(
                "<span class=\"dodai-slider__tick\" style=\"{}\"></span>",
                at(f)
            ));
        }
    }
    let room = match (&o.bubble, o.bubble_below) {
        (None, _) => "",
        (Some(_), false) => "padding-top:30px",
        (Some(_), true) => "padding-bottom:30px",
    };
    let bubble = o.bubble.as_ref().map_or(String::new(), |b| {
        format!(
            "<span class=\"dodai-slider__bubble\" style=\"{};{}\">{}</span>",
            at(pct / 100.0),
            if o.bubble_below { "top:26px" } else { "top:0" },
            esc(b)
        )
    });
    format!("<div class=\"dodai-slider-box\" style=\"{room}\">{input}{marks}{bubble}</div>")
}

/// A two-thumb range slider over `[lo, hi, min, max, step]`; `lo_attrs` and `hi_attrs` are the
/// inputs' attributes.
pub fn range_slider(v: [f32; 5], vertical: Option<u32>, lo_attrs: &str, hi_attrs: &str) -> String {
    let [lo, hi, min, max, step] = v;
    let (lo, hi) = (lo.min(hi), hi.max(lo));
    let span = (max - min).max(f32::EPSILON);
    let pct = |v: f32| ((v - min) / span).clamp(0.0, 1.0) * 100.0;
    let (class, style, thumb) = match vertical {
        Some(len) => (
            "dodai-range dodai-range--v",
            format!("--a:{}%;--b:{}%;height:{len}px", pct(lo), pct(hi)),
            "dodai-slider dodai-slider--v",
        ),
        None => (
            "dodai-range",
            format!("--a:{}%;--b:{}%", pct(lo), pct(hi)),
            "dodai-slider",
        ),
    };
    format!(
        "<div class=\"{class}\" style=\"{style}\"><input class=\"{thumb}\" type=\"range\" min=\"{min}\" max=\"{max}\" step=\"{step}\" value=\"{lo}\" aria-label=\"From\" {lo_attrs}><input class=\"{thumb}\" type=\"range\" min=\"{min}\" max=\"{max}\" step=\"{step}\" value=\"{hi}\" aria-label=\"To\" {hi_attrs}></div>"
    )
}

/// A text input (several `lines` make a text area).
pub fn text_input(
    value: &str,
    placeholder: &str,
    password: bool,
    lines: u8,
    attrs: &str,
) -> String {
    if lines > 1 {
        return format!(
            "<textarea class=\"dodai-textarea\" rows=\"{lines}\" placeholder=\"{}\" {attrs}>{}</textarea>",
            esc(placeholder),
            esc(value)
        );
    }
    format!(
        "<input class=\"dodai-input\" type=\"{}\" value=\"{}\" placeholder=\"{}\" {attrs}>",
        if password { "password" } else { "text" },
        esc(value),
        esc(placeholder)
    )
}

/// A native dropdown; `attrs` go on the `<select>`.
pub fn select(items: &[String], selected: Option<usize>, placeholder: &str, attrs: &str) -> String {
    let mut o = format!("<select class=\"dodai-select\" {attrs}>");
    if selected.is_none() {
        o.push_str(&format!(
            "<option value=\"\" selected disabled>{}</option>",
            esc(placeholder)
        ));
    }
    for (i, it) in items.iter().enumerate() {
        o.push_str(&format!(
            "<option value=\"{i}\"{}>{}</option>",
            if selected == Some(i) { " selected" } else { "" },
            esc(it)
        ));
    }
    o.push_str("</select>");
    o
}

/// A segmented control; `attrs(i)` are the attributes of segment `i`.
pub fn segmented(
    cx: &Cx,
    labels: &[String],
    selected: usize,
    attrs: &dyn Fn(usize) -> String,
) -> String {
    let theme = cx.theme;
    let tonal = theme.look().tonal;
    let h = if tonal {
        theme.sizes().segmented_height
    } else {
        theme.shape.control_height - 6.0
    };
    let gap = if tonal { 0.0 } else { theme.spacing.half() };
    let upper = if theme.typography.uppercase_labels {
        "text-transform:uppercase;"
    } else {
        ""
    };
    let size = theme.typography.size_md;
    let mut inner = String::new();
    for (i, l) in labels.iter().enumerate() {
        let tick = if i == selected && tonal {
            "\u{2713} "
        } else {
            ""
        };
        inner.push_str(&kit_box(
            cx,
            Kit::Segment {
                selected: i == selected,
            },
            &format!("height:{h}px;display:flex;align-items:center;white-space:nowrap;{upper}font-size:{size}px"),
            &Surface::default(),
            true,
            &attrs(i),
            &format!("{tick}{}", esc(l)),
        ));
    }
    kit_box(
        cx,
        Kit::Segmented,
        &format!("display:inline-flex;align-self:flex-start;flex:none;gap:{gap}px"),
        &Surface::default(),
        false,
        "",
        &inner,
    )
}

/// An avatar with initials.
pub fn avatar(initials: &str, size: u32) -> String {
    format!(
        "<div class=\"dodai-kit dodai-avatar\" style=\"{}width:{size}px;height:{size}px;\">{}</div>",
        crate::kit_style(Kit::Avatar),
        esc(initials)
    )
}

/// The icon an alert of `tone` shows.
pub fn alert_icon(tone: Tone) -> IconId {
    match tone {
        Tone::Danger | Tone::Warning => IconId::Warning,
        Tone::Success => IconId::CheckCircle,
        Tone::Info => IconId::Info,
    }
}

/// A step of [`steps`].
pub struct StepItem<'a> {
    /// Its title.
    pub title: &'a str,
    /// A line under it.
    pub detail: Option<&'a str>,
}

/// A stepper track with `current` the active step.
pub fn steps(cx: &Cx, items: &[StepItem], current: usize) -> String {
    let n = items.len();
    let mut o = String::from("<div class=\"dodai-steps\">");
    for (i, s) in items.iter().enumerate() {
        let state = StepState::of(i, current);
        let inner = if state == StepState::Done {
            icon(cx, IconId::Check, 16, false)
        } else {
            format!("<span class=\"dodai-sm\">{}</span>", i + 1)
        };
        o.push_str(&kit_box(
            cx,
            Kit::StepDot(state),
            "width:28px;height:28px;padding:0;justify-content:center;",
            &Surface::default(),
            false,
            "",
            &inner,
        ));
        o.push_str(&format!(
            "<div class=\"dodai-col-text\"><span class=\"dodai-label\">{}</span>{}</div>",
            esc(s.title),
            s.detail.map_or(String::new(), |d| format!(
                "<span class=\"dodai-caption\">{}</span>",
                esc(d)
            ))
        ));
        if i + 1 < n {
            o.push_str("<span class=\"dodai-steps__line\"></span>");
        }
    }
    o.push_str("</div>");
    o
}

/// Breadcrumbs.
pub fn breadcrumb(cx: &Cx, items: &[String]) -> String {
    let n = items.len();
    let mut o = String::from("<nav class=\"dodai-crumbs\" aria-label=\"Breadcrumb\">");
    for (i, it) in items.iter().enumerate() {
        o.push_str(&format!(
            "<span aria-current=\"{}\">{}</span>",
            if i + 1 == n { "page" } else { "false" },
            esc(it)
        ));
        if i + 1 < n {
            o.push_str(&icon(cx, IconId::ChevronRight, 14, false));
        }
    }
    o.push_str("</nav>");
    o
}

/// The page numbers shown (`None` is a gap), 0-based.
pub fn page_slots(page: usize, pages: usize) -> Vec<Option<usize>> {
    dodai_core::paging::page_window(page + 1, pages, 7)
        .into_iter()
        .map(|it| match it {
            dodai_core::paging::PageItem::Page(p) => Some(p - 1),
            dodai_core::paging::PageItem::Ellipsis => None,
        })
        .collect()
}

/// Pagination; `attrs(i)` are the attributes of the button that goes to page `i`.
pub fn pagination(cx: &Cx, pages: usize, page: usize, attrs: &dyn Fn(usize) -> String) -> String {
    let nav = |label: IconId, to: usize, off: bool| {
        format!(
            "<button class=\"dodai-btn dodai-btn--ghost dodai-btn--sm dodai-btn--icon\"{} {}>{}</button>",
            if off { " disabled" } else { "" },
            attrs(to),
            icon(cx, label, 16, false)
        )
    };
    let mut o = String::from("<nav class=\"dodai-pager\" aria-label=\"Pagination\">");
    o.push_str(&nav(IconId::ChevronLeft, page.saturating_sub(1), page == 0));
    for slot in page_slots(page, pages) {
        match slot {
            Some(i) => o.push_str(&kit_box(
                cx,
                Kit::Page {
                    selected: i == page,
                },
                "",
                &Surface::default(),
                true,
                &attrs(i),
                &(i + 1).to_string(),
            )),
            None => o.push_str("<span class=\"dodai-muted\">…</span>"),
        }
    }
    o.push_str(&nav(
        IconId::ChevronRight,
        (page + 1).min(pages.saturating_sub(1)),
        page + 1 >= pages,
    ));
    o.push_str("</nav>");
    o
}

/// A node of [`tree`].
pub struct TreeItem {
    /// Its label.
    pub label: String,
    /// An icon before it.
    pub icon: Option<IconId>,
    /// Its children.
    pub children: Vec<TreeItem>,
    /// Starts open.
    pub open: bool,
    /// Drawn as the picked row.
    pub selected: bool,
}

fn tree_branch(cx: &Cx, it: &TreeItem) -> String {
    let lead = if it.children.is_empty() {
        "<span style=\"width:14px\"></span>".to_string()
    } else {
        format!(
            "<span class=\"dodai-tree__chev\">{}</span>",
            icon(cx, IconId::ChevronRight, 14, false)
        )
    };
    let ic = it.icon.map_or(String::new(), |i| icon(cx, i, 16, false));
    let row = kit_box(
        cx,
        Kit::Row {
            selected: it.selected,
        },
        "",
        &Surface::default(),
        false,
        "",
        &format!("{lead}{ic}<span>{}</span>", esc(&it.label)),
    );
    if it.children.is_empty() {
        return row;
    }
    let kids: String = it.children.iter().map(|c| tree_branch(cx, c)).collect();
    format!(
        "<details{}><summary>{row}</summary><div class=\"dodai-tree__children\">{kids}</div></details>",
        if it.open { " open" } else { "" }
    )
}

/// A tree view.
pub fn tree(cx: &Cx, items: &[TreeItem]) -> String {
    let inner: String = items.iter().map(|i| tree_branch(cx, i)).collect();
    format!("<div class=\"dodai-tree\">{inner}</div>")
}

/// An event of [`timeline`].
pub struct TimelineItem<'a> {
    /// What happened.
    pub title: &'a str,
    /// When.
    pub time: &'a str,
    /// More about it.
    pub detail: Option<&'a str>,
    /// Colour of its dot.
    pub tone: Option<Tone>,
}

/// A timeline.
pub fn timeline(cx: &Cx, events: &[TimelineItem]) -> String {
    let mut o = String::from("<div class=\"dodai-timeline\">");
    for e in events {
        let dot = kit_box(
            cx,
            Kit::Dot(e.tone),
            "",
            &Surface::default(),
            false,
            "",
            "<div></div>",
        );
        o.push_str(&format!(
            "<div class=\"dodai-timeline__item\">{dot}<div class=\"dodai-col-text\"><span>{}</span><span class=\"dodai-caption\">{}</span>{}</div></div>",
            esc(e.title),
            esc(e.time),
            e.detail.map_or(String::new(), |d| format!(
                "<span class=\"dodai-muted\">{}</span>",
                esc(d)
            ))
        ));
    }
    o.push_str("</div>");
    o
}

/// Which of `max` stars are filled for `value`.
pub fn star_fills(value: f32, max: u8) -> Vec<bool> {
    let full = value.round().max(0.0) as u8;
    (0..max).map(|i| i < full).collect()
}

/// A star rating.
pub fn rating(cx: &Cx, value: f32, max: u8) -> String {
    let stars: String = star_fills(value, max)
        .into_iter()
        .map(|on| icon(cx, IconId::Star, 20, on))
        .collect();
    format!(
        "<span class=\"dodai-rating\" role=\"img\" aria-label=\"{value} of {max}\">{stars}</span>"
    )
}

/// A number stepper; `minus` and `plus` are the buttons' attributes.
pub fn stepper(value: i32, min: i32, max: i32, minus: &str, plus: &str) -> String {
    format!(
        "<span class=\"dodai-stepper\"><button class=\"dodai-btn dodai-btn--outline dodai-btn--sm dodai-btn--icon\" aria-label=\"Decrease\"{} {minus}>−</button><span>{value}</span><button class=\"dodai-btn dodai-btn--outline dodai-btn--sm dodai-btn--icon\" aria-label=\"Increase\"{} {plus}>+</button></span>",
        if value <= min { " disabled" } else { "" },
        if value >= max { " disabled" } else { "" },
    )
}

/// The class suffix of a spinner style and the style of each of its cells.
pub fn spinner_cells(style: SpinnerStyle, size: f32) -> (&'static str, Vec<String>) {
    let kind = match style {
        SpinnerStyle::Ring => "ring",
        SpinnerStyle::Bars => "bars",
        SpinnerStyle::Orbit => "orbit",
        SpinnerStyle::Steps => "steps",
        SpinnerStyle::Grid => "grid",
        SpinnerStyle::Scanner => "scanner",
        SpinnerStyle::Hop => "hop",
        SpinnerStyle::Petals => "petals",
        SpinnerStyle::Quad => "quad",
        SpinnerStyle::Glitch => "glitch",
        SpinnerStyle::Shuffle => "shuffle",
        SpinnerStyle::Data => "data",
        SpinnerStyle::Pillars => "pillars",
        SpinnerStyle::Courses => "courses",
        SpinnerStyle::Bounce => "bounce",
    };
    let cells = busy::cells(style, size)
        .iter()
        .map(|c| {
            // A negative delay starts the cell part-way through its cycle, as the head left it.
            // (A shuffle piece's `lag` is its number: the pieces run on their own tracks.)
            let delay = if style == SpinnerStyle::Shuffle {
                0.0
            } else {
                -(-c.lag).rem_euclid(1.0) / busy::SPIN_RATE
            };
            // Quad and shuffle cells slide between the corners of the square: how far.
            let far = size - c.w;
            let fary = size - c.h;
            let fade = 1.0 - c.lag * 0.55;
            let extra = if style == SpinnerStyle::Quad {
                format!(";opacity:{fade:.2}")
            } else {
                String::new()
            };
            format!(
                "left:{:.2}px;top:{:.2}px;width:{:.2}px;height:{:.2}px;border-radius:{:.2}px;--far:{far:.2}px;--fary:{fary:.2}px;animation-delay:{:.3}s{extra}",
                c.x, c.y, c.w, c.h, c.radius, delay
            )
        })
        .collect();
    (kind, cells)
}

/// The design's busy spinner.
pub fn spinner(cx: &Cx, size: u32) -> String {
    let (kind, cells) = spinner_cells(cx.theme.look().spinner, size as f32);
    let cells: String = cells
        .iter()
        .map(|c| format!("<i style=\"{c}\"></i>"))
        .collect();
    format!(
        "<span class=\"dodai-spinner dodai-spinner--{kind}\" role=\"progressbar\" style=\"--s:{size}px\">{cells}</span>"
    )
}

/// An indeterminate progress bar.
pub fn indeterminate() -> String {
    "<div class=\"dodai-indeterminate\" role=\"progressbar\"><span></span></div>".to_string()
}

/// A skeleton placeholder (`width` 0 = full).
pub fn skeleton(width: u32, height: u32) -> String {
    let w = if width == 0 {
        "100%".to_string()
    } else {
        format!("{width}px")
    };
    format!(
        "<span class=\"dodai-skeleton\" style=\"width:{w};height:{height}px;{}background:var(--dodai-kit-skeleton-bg)\"></span>",
        crate::kit_style(Kit::Skeleton)
    )
}

/// An icon button with a count or dot badge.
pub fn badged(cx: &Cx, id: IconId, count: Option<(u32, Tone)>, dot: Option<Tone>) -> String {
    let badge = if let Some((n, tone)) = count {
        kit_box(
            cx,
            Kit::Count(tone),
            "",
            &Surface::default(),
            false,
            "",
            &n.to_string(),
        )
    } else if let Some(tone) = dot {
        kit_box(
            cx,
            Kit::Dot(Some(tone)),
            "",
            &Surface::default(),
            false,
            "",
            "<span></span>",
        )
    } else {
        String::new()
    };
    format!(
        "<span class=\"dodai-badged\"><button class=\"dodai-btn dodai-btn--ghost dodai-btn--md dodai-btn--icon\">{}</button>{badge}</span>",
        icon(cx, id, 18, false)
    )
}

fn knob_path(pts: &[(f32, f32)]) -> String {
    let mut d = String::new();
    for (i, (x, y)) in pts.iter().enumerate() {
        d.push_str(if i == 0 { "M" } else { "L" });
        d.push_str(&format!("{x:.2} {y:.2} "));
    }
    d
}

/// The inside of a rotary knob's SVG at `frac` (0..=1).
pub fn knob_svg(theme: &Theme, size: f32, frac: f32, disabled: bool) -> String {
    let st = theme.knob(disabled);
    let g = KnobGeometry::new(size);
    let c = size / 2.0;
    let frac = frac.clamp(0.0, 1.0);
    let track = knob_path(&arc_points(c, c, g.r_ring, 0.0, 1.0));
    let mut s = format!(
        "<path d=\"{track}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{}\"/>",
        st.track.to_css(),
        g.ring
    );
    if frac > 0.0 {
        let arc = knob_path(&arc_points(c, c, g.r_ring, 0.0, frac));
        if let Some((col, r)) = st.glow {
            s.push_str(&format!(
                "<path d=\"{arc}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{}\" opacity=\"0.5\" style=\"filter:blur({}px)\"/>",
                col.to_css(),
                g.ring + r * 0.5,
                (r * 0.5).max(1.0)
            ));
        }
        s.push_str(&format!(
            "<path d=\"{arc}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{}\"/>",
            st.fill.to_css(),
            g.ring
        ));
    }
    s.push_str(&format!(
        "<circle cx=\"{c}\" cy=\"{c}\" r=\"{}\" fill=\"{}\" stroke=\"{}\" stroke-width=\"{}\"/>",
        g.r_body,
        st.body.to_css(),
        st.body_edge.to_css(),
        st.border_width
    ));
    let th = knob_angle(frac);
    let (co, si) = (th.cos(), th.sin());
    s.push_str(&format!(
        "<line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" stroke=\"{}\" stroke-width=\"{}\"/>",
        c + co * g.r_body * 0.35,
        c + si * g.r_body * 0.35,
        c + co * g.r_body * 0.9,
        c + si * g.r_body * 0.9,
        st.tick.to_css(),
        g.tick
    ));
    s
}

/// A rotary knob; `attrs` go on the `<svg>`.
pub fn knob(
    theme: &Theme,
    value: f32,
    min: f32,
    max: f32,
    size: f32,
    disabled: bool,
    attrs: &str,
) -> String {
    let span = (max - min).max(f32::EPSILON);
    let svg = knob_svg(theme, size, (value - min) / span, disabled);
    let dim = if disabled {
        "opacity:.45;"
    } else {
        "cursor:ns-resize;"
    };
    format!(
        "<svg class=\"dodai-knob\" width=\"{size}\" height=\"{size}\" viewBox=\"0 0 {size} {size}\" style=\"{dim}touch-action:none;user-select:none\" role=\"slider\" aria-valuemin=\"{min}\" aria-valuemax=\"{max}\" aria-valuenow=\"{value}\" aria-disabled=\"{disabled}\" {attrs}>{svg}</svg>"
    )
}

fn gizmo_tone(p: &Palette, t: AxisTone) -> dodai_core::Color {
    match t {
        AxisTone::X => p.danger,
        AxisTone::Y => p.success,
        AxisTone::Z => p.primary,
    }
}

/// The inside of the orientation gizmo's SVG for an orbit camera at `yaw` and `pitch`.
pub fn gizmo_svg(theme: &Theme, size: f32, yaw: f32, pitch: f32) -> String {
    let p = &theme.palette;
    let layout = GizmoLayout::new(size);
    let placed = place(&Basis::from_orbit(yaw, pitch), &layout, &AxisId::DEFAULT);
    let [cx, cy] = layout.centre();
    let bw = theme.shape.border_width.max(1.0);
    let mut s = format!(
        "<circle cx=\"{cx}\" cy=\"{cy}\" r=\"{}\" fill=\"{}\" fill-opacity=\"0.72\" stroke=\"{}\" stroke-width=\"{bw}\"/>",
        size / 2.0 - bw / 2.0,
        p.surface.to_css(),
        p.border_strong.to_css(),
    );
    for a in &placed {
        let op = if a.axis.positive() { 1.0 } else { 0.55 };
        s.push_str(&format!(
            "<line x1=\"{cx}\" y1=\"{cy}\" x2=\"{:.2}\" y2=\"{:.2}\" stroke=\"{}\" stroke-opacity=\"{op}\" stroke-width=\"{}\" stroke-linecap=\"round\"/>",
            a.centre[0],
            a.centre[1],
            gizmo_tone(p, a.axis.tone()).to_css(),
            layout.stem
        ));
    }
    for a in &placed {
        let c = gizmo_tone(p, a.axis.tone());
        let (x, y) = (a.centre[0], a.centre[1]);
        if a.axis.positive() {
            s.push_str(&format!(
                "<circle cx=\"{x:.2}\" cy=\"{y:.2}\" r=\"{:.2}\" fill=\"{}\"/><text x=\"{x:.2}\" y=\"{y:.2}\" fill=\"{}\" font-size=\"{:.1}\" font-weight=\"700\" text-anchor=\"middle\" dominant-baseline=\"central\">{}</text>",
                a.radius,
                c.to_css(),
                theme.on(c).to_css(),
                a.radius * 2.0 * 0.55,
                a.axis.letter()
            ));
        } else {
            let w = (a.radius * 2.0 * 0.17).max(2.0);
            s.push_str(&format!(
                "<circle cx=\"{x:.2}\" cy=\"{y:.2}\" r=\"{:.2}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{w:.2}\"/>",
                a.radius - w / 2.0,
                c.to_css()
            ));
        }
    }
    let [hx, hy] = layout.home_centre();
    let hr = layout.home / 2.0;
    let k = hr * 0.55;
    s.push_str(&format!(
        "<circle cx=\"{hx:.2}\" cy=\"{hy:.2}\" r=\"{hr:.2}\" fill=\"{}\" stroke=\"{}\" stroke-width=\"{bw}\"/><path d=\"M{:.2} {hy:.2} L{hx:.2} {:.2} L{:.2} {hy:.2} M{:.2} {:.2} V{:.2} H{:.2} V{:.2}\" fill=\"none\" stroke=\"{}\" stroke-width=\"1.5\"/>",
        p.surface_raised.to_css(),
        p.border.to_css(),
        hx - k,
        hy - k,
        hx + k,
        hx - k * 0.7,
        hy - k * 0.2,
        hy + k * 0.8,
        hx + k * 0.7,
        hy - k * 0.2,
        p.text.to_css(),
    ));
    s
}

/// The orientation gizmo; `attrs` go on the `<svg>`.
pub fn gizmo(theme: &Theme, size: f32, yaw: f32, pitch: f32, attrs: &str) -> String {
    let svg = gizmo_svg(theme, size, yaw, pitch);
    format!(
        "<svg class=\"dodai-gizmo\" width=\"{size}\" height=\"{size}\" viewBox=\"0 0 {size} {size}\" style=\"cursor:pointer;touch-action:none;user-select:none\" {attrs}>{svg}</svg>"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use dodai_core::themes;

    fn cx(t: &Theme) -> Cx<'_> {
        Cx {
            theme: t,
            fx: SurfaceFx::NONE,
        }
    }

    #[test]
    fn text_is_escaped() {
        assert_eq!(
            esc("<a href=\"x\">&</a>"),
            "&lt;a href=&quot;x&quot;&gt;&amp;&lt;/a&gt;"
        );
        assert!(tag(Tone::Info, false, "<b>").contains("&lt;b&gt;"));
    }

    #[test]
    fn buttons_carry_variant_size_and_state() {
        let t = themes::all().remove(0);
        let c = cx(&t);
        let b = Btn {
            variant: Some(ButtonVariant::Danger),
            size: Some(ControlSize::Large),
            disabled: true,
            ..Btn::default()
        };
        let h = button(&c, &b, &Surface::default(), "data-a=\"x\"", "Go");
        assert!(h.contains("dodai-btn--danger") && h.contains("dodai-btn--lg"));
        assert!(h.contains(" disabled") && h.contains("data-a=\"x\"") && h.contains(">Go<"));
    }

    #[test]
    fn slider_and_progress_fill() {
        let o = SliderOpts {
            min: 0.0,
            max: 1.0,
            step: 0.01,
            ..SliderOpts::default()
        };
        assert!(slider(0.25, &o, "").contains("--v:25%"));
        assert!(progress(2.0, None).contains("--v:100%"));
    }

    #[test]
    fn rating_fills_whole_stars_and_rounds_halves_up() {
        assert_eq!(star_fills(3.5, 5), vec![true, true, true, true, false]);
        assert_eq!(star_fills(-2.0, 2), vec![false; 2]);
    }

    #[test]
    fn pagination_shows_a_window_with_gaps() {
        assert!(page_slots(0, 0).is_empty());
        let s = page_slots(4, 20);
        assert_eq!((s[0], s[s.len() - 1]), (Some(0), Some(19)));
        assert!(s.contains(&None) && s.contains(&Some(4)));
    }

    #[test]
    fn each_spinner_style_has_a_class_and_styled_cells() {
        assert_eq!(spinner_cells(SpinnerStyle::Ring, 24.0), ("ring", vec![]));
        assert_eq!(spinner_cells(SpinnerStyle::Grid, 24.0).1.len(), 9);
        let (_, cells) = spinner_cells(SpinnerStyle::Steps, 24.0);
        assert!(cells[0].ends_with("animation-delay:0.000s"));
    }

    #[test]
    fn components_render_their_markup() {
        let t = themes::all().remove(0);
        let c = cx(&t);
        let h = [
            kit_box(
                &c,
                Kit::Card(CardVariant::Elevated),
                "",
                &Surface::default(),
                false,
                "",
                "x",
            ),
            pagination(&c, 3, 1, &|i| format!("data-a=\"{i}\"")),
            steps(
                &c,
                &[
                    StepItem {
                        title: "One",
                        detail: None,
                    },
                    StepItem {
                        title: "Two",
                        detail: Some("d"),
                    },
                ],
                1,
            ),
            skeleton(0, 16),
            stepper(0, 0, 20, "", ""),
        ]
        .concat();
        for want in [
            "dodai-kit--card",
            "dodai-kit--elevated",
            "dodai-pager",
            "dodai-steps__line",
            "dodai-skeleton",
            "disabled",
        ] {
            assert!(h.contains(want), "missing {want}");
        }
    }

    #[test]
    fn the_gizmo_svg_has_the_disc_stems_balls_and_home() {
        let t = themes::all().remove(0);
        let s = gizmo_svg(&t, 130.0, 0.6, 0.45);
        assert_eq!(s.matches("<line").count(), AxisId::DEFAULT.len());
        assert!(s.contains(">X<") && s.contains("<path"));
    }
}
