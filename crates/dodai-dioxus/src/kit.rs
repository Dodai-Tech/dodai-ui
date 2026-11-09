//! Token-styled boxes and the composite components built on them: cards, alerts, chips,
//! avatars, code, steps, breadcrumbs, pagination, tree, data table, timeline, rating, stepper,
//! busy indicators. Looks come from `dodai_core::Kit` through the stylesheet variables of
//! `dodai-topcoat` ([`dodai_topcoat::kit_style`]), so every design restyles them.

use dioxus::prelude::*;
use dodai_core::{busy, CardVariant, Kit, SpinnerStyle, StepState, Tone};

use crate::Icon;
use dodai_icons::IconId;

fn kit_class(kit: Kit) -> String {
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

/// Properties of [`KitBox`].
#[derive(Props, Clone, PartialEq)]
pub struct KitBoxProps {
    pub kit: Kit,
    /// Extra inline style (size, position).
    #[props(default)]
    pub style: String,
    /// Makes the box a button.
    #[props(default)]
    pub onclick: Option<EventHandler<MouseEvent>>,
    /// The box's own surface override (frost, fill opacity); see [`dodai_core::Surface`].
    #[props(default)]
    pub surface: dodai_core::Surface,
    pub children: Element,
}

/// A box that looks like `kit`; with `onclick` it is a button and hovers.
#[component]
pub fn KitBox(props: KitBoxProps) -> Element {
    let mut style = format!("{}{}", dodai_topcoat::kit_style(props.kit), props.style);
    let mut class = kit_class(props.kit);
    if let Some(g) = crate::provider::glass(props.kit.surface_class(), &props.surface) {
        class.push_str(" dodai-glass");
        style.push_str(&g);
    }
    match props.onclick {
        Some(h) => rsx! {
            button { class: "{class} dodai-kit--press", style, onclick: move |e| h.call(e), {props.children} }
        },
        None => rsx! { div { class, style, {props.children} } },
    }
}

/// Properties of [`Card`].
#[derive(Props, Clone, PartialEq)]
pub struct CardProps {
    #[props(default)]
    pub variant: CardVariant,
    #[props(default)]
    pub title: String,
    /// The card's own surface override (frost, fill opacity); see [`dodai_core::Surface`].
    #[props(default)]
    pub surface: dodai_core::Surface,
    pub children: Element,
}

/// Card: elevated, filled or outlined.
#[component]
pub fn Card(props: CardProps) -> Element {
    rsx! {
        KitBox { kit: Kit::Card(props.variant), surface: props.surface,
            if !props.title.is_empty() { span { class: "dodai-label", "{props.title}" } }
            {props.children}
        }
    }
}

/// Properties of [`Alert`].
#[derive(Props, Clone, PartialEq)]
pub struct AlertProps {
    pub tone: Tone,
    /// Full-width solid banner.
    #[props(default)]
    pub banner: bool,
    pub children: Element,
}

/// The icon an alert of `tone` shows.
pub fn alert_icon(tone: Tone) -> IconId {
    match tone {
        Tone::Danger | Tone::Warning => IconId::Warning,
        Tone::Success => IconId::CheckCircle,
        Tone::Info => IconId::Info,
    }
}

/// Message bar in a status tone.
#[component]
pub fn Alert(props: AlertProps) -> Element {
    rsx! {
        KitBox { kit: Kit::Alert { tone: props.tone, banner: props.banner }, style: "width:100%;".to_string(),
            Icon { icon: alert_icon(props.tone), size: 18 }
            span { {props.children} }
        }
    }
}

/// Properties of [`Snackbar`].
#[derive(Props, Clone, PartialEq)]
pub struct SnackbarProps {
    pub text: String,
    pub action: String,
    #[props(default)]
    pub onaction: EventHandler<MouseEvent>,
}

/// Snackbar with one action.
#[component]
pub fn Snackbar(props: SnackbarProps) -> Element {
    rsx! {
        KitBox { kit: Kit::Snackbar, style: "gap:var(--dodai-space-lg);width:fit-content;".to_string(),
            span { "{props.text}" }
            button { class: "dodai-btn dodai-btn--ghost", onclick: move |e| props.onaction.call(e), "{props.action}" }
        }
    }
}

/// Properties of [`Chip`].
#[derive(Props, Clone, PartialEq)]
pub struct ChipProps {
    pub label: String,
    #[props(default)]
    pub selected: bool,
    #[props(default)]
    pub icon: Option<IconId>,
    #[props(default)]
    pub onclick: EventHandler<MouseEvent>,
}

/// Filter chip.
#[component]
pub fn Chip(props: ChipProps) -> Element {
    let h = props.onclick;
    rsx! {
        KitBox { kit: Kit::Chip { selected: props.selected }, onclick: move |e| h.call(e),
            if let Some(i) = props.icon { Icon { icon: i, size: 16 } }
            span { "{props.label}" }
        }
    }
}

/// Round avatar with initials.
#[component]
pub fn Avatar(initials: String, #[props(default = 40)] size: u32) -> Element {
    rsx! {
        div { class: "dodai-kit dodai-avatar", style: "{dodai_topcoat::kit_style(Kit::Avatar)}width:{size}px;height:{size}px;",
            "{initials}"
        }
    }
}

/// Code listing.
#[component]
pub fn CodeBlock(code: String) -> Element {
    rsx! {
        KitBox { kit: Kit::Code, style: "display:block;".to_string(),
            pre { class: "dodai-code", "{code}" }
        }
    }
}

/// One step of [`Steps`].
#[derive(Clone, PartialEq)]
pub struct StepItem {
    pub title: String,
    pub detail: Option<String>,
}

/// Numbered step indicator; steps before `current` show a check.
#[component]
pub fn Steps(steps: Vec<StepItem>, current: usize) -> Element {
    let n = steps.len();
    rsx! {
        div { class: "dodai-steps",
            for (i, s) in steps.into_iter().enumerate() {
                KitBox { key: "{i}", kit: Kit::StepDot(StepState::of(i, current)), style: "width:28px;height:28px;padding:0;justify-content:center;".to_string(),
                    if StepState::of(i, current) == StepState::Done {
                        Icon { icon: IconId::Check, size: 16 }
                    } else {
                        span { class: "dodai-sm", "{i + 1}" }
                    }
                }
                div { class: "dodai-col-text",
                    span { class: "dodai-label", "{s.title}" }
                    if let Some(d) = s.detail { span { class: "dodai-caption", "{d}" } }
                }
                if i + 1 < n { span { class: "dodai-steps__line" } }
            }
        }
    }
}

/// Breadcrumb trail; the last item is the current page.
#[component]
pub fn Breadcrumb(items: Vec<String>) -> Element {
    let n = items.len();
    rsx! {
        nav { class: "dodai-crumbs", "aria-label": "Breadcrumb",
            for (i, it) in items.into_iter().enumerate() {
                span { key: "{i}", "aria-current": if i + 1 == n { "page" } else { "false" }, "{it}" }
                if i + 1 < n { Icon { icon: IconId::ChevronRight, size: 14 } }
            }
        }
    }
}

/// The page buttons to show for 0-based `page` of `pages`: the first, the last and a window
/// around `page`, gaps as `None` (at most seven slots).
pub fn page_slots(page: usize, pages: usize) -> Vec<Option<usize>> {
    dodai_core::paging::page_window(page + 1, pages, 7)
        .into_iter()
        .map(|it| match it {
            dodai_core::paging::PageItem::Page(p) => Some(p - 1),
            dodai_core::paging::PageItem::Ellipsis => None,
        })
        .collect()
}

/// Properties of [`Pagination`].
#[derive(Props, Clone, PartialEq)]
pub struct PaginationProps {
    pub pages: usize,
    pub page: usize,
    #[props(default)]
    pub onselect: EventHandler<usize>,
}

/// Page selector with previous and next.
#[component]
pub fn Pagination(props: PaginationProps) -> Element {
    let (pages, page, h) = (props.pages, props.page, props.onselect);
    rsx! {
        nav { class: "dodai-pager", "aria-label": "Pagination",
            button { class: "dodai-btn dodai-btn--ghost dodai-btn--sm dodai-btn--icon", disabled: page == 0,
                onclick: move |_| h.call(page.saturating_sub(1)), Icon { icon: IconId::ChevronLeft, size: 16 } }
            for (n, slot) in page_slots(page, pages).into_iter().enumerate() {
                if let Some(i) = slot {
                    KitBox { key: "{n}", kit: Kit::Page { selected: i == page }, onclick: move |_| h.call(i), "{i + 1}" }
                } else {
                    span { key: "{n}", class: "dodai-muted", "…" }
                }
            }
            button { class: "dodai-btn dodai-btn--ghost dodai-btn--sm dodai-btn--icon", disabled: page + 1 >= pages,
                onclick: move |_| h.call((page + 1).min(pages.saturating_sub(1))), Icon { icon: IconId::ChevronRight, size: 16 } }
        }
    }
}

/// A node of [`TreeView`].
#[derive(Clone, PartialEq)]
pub struct TreeItem {
    pub label: String,
    pub icon: Option<IconId>,
    pub children: Vec<TreeItem>,
    pub open: bool,
    /// Drawn as the selected row.
    pub selected: bool,
}

#[component]
fn TreeBranch(item: TreeItem) -> Element {
    let row = rsx! {
        KitBox { kit: Kit::Row { selected: item.selected },
            if !item.children.is_empty() { span { class: "dodai-tree__chev", Icon { icon: IconId::ChevronRight, size: 14 } } }
            else { span { style: "width:14px" } }
            if let Some(i) = item.icon { Icon { icon: i, size: 16 } }
            span { "{item.label}" }
        }
    };
    if item.children.is_empty() {
        return row;
    }
    rsx! {
        details { open: item.open,
            summary { {row} }
            div { class: "dodai-tree__children",
                for (i, c) in item.children.iter().enumerate() {
                    TreeBranch { key: "{i}", item: c.clone() }
                }
            }
        }
    }
}

/// Expandable tree.
#[component]
pub fn TreeView(items: Vec<TreeItem>) -> Element {
    rsx! {
        div { class: "dodai-tree",
            for (i, it) in items.into_iter().enumerate() { TreeBranch { key: "{i}", item: it } }
        }
    }
}

/// Data table with a header row and banded body rows.
#[component]
pub fn DataTable(columns: Vec<String>, rows: Vec<Vec<String>>) -> Element {
    let cols = columns.len();
    rsx! {
        div { class: "dodai-datatable", role: "table",
            KitBox { kit: Kit::TableHead, style: format!("--cols:{cols};display:grid;grid-template-columns:repeat({cols},1fr);"),
                for c in columns { span { class: "dodai-label", "{c}" } }
            }
            for (i, r) in rows.into_iter().enumerate() {
                KitBox { key: "{i}", kit: Kit::TableRow(i % 2 == 1), style: format!("display:grid;grid-template-columns:repeat({cols},1fr);"),
                    for c in r { span { "{c}" } }
                }
            }
        }
    }
}

/// One event of [`Timeline`].
#[derive(Clone, PartialEq)]
pub struct TimelineItem {
    pub title: String,
    pub time: String,
    pub detail: Option<String>,
    pub tone: Option<Tone>,
}

/// Events down a line.
#[component]
pub fn Timeline(events: Vec<TimelineItem>) -> Element {
    rsx! {
        div { class: "dodai-timeline",
            for (i, e) in events.into_iter().enumerate() {
                div { key: "{i}", class: "dodai-timeline__item",
                    KitBox { kit: Kit::Dot(e.tone), div {} }
                    div { class: "dodai-col-text",
                        span { "{e.title}" }
                        span { class: "dodai-caption", "{e.time}" }
                        if let Some(d) = e.detail { span { class: "dodai-muted", "{d}" } }
                    }
                }
            }
        }
    }
}

/// How each of `max` stars of a `value` rating is drawn: `true` = filled. Halves round up.
pub fn star_fills(value: f32, max: u8) -> Vec<bool> {
    let full = value.round().max(0.0) as u8;
    (0..max).map(|i| i < full).collect()
}

/// Star rating (display).
#[component]
pub fn Rating(value: f32, #[props(default = 5)] max: u8) -> Element {
    rsx! {
        span { class: "dodai-rating", role: "img", "aria-label": "{value} of {max}",
            for (i, on) in star_fills(value, max).into_iter().enumerate() {
                Icon { key: "{i}", icon: IconId::Star, size: 20, filled: on }
            }
        }
    }
}

/// The value after pressing a stepper button, kept inside `min..=max`.
pub fn stepped(value: i32, by: i32, min: i32, max: i32) -> i32 {
    (value + by).clamp(min, max)
}

/// Properties of [`Stepper`].
#[derive(Props, Clone, PartialEq)]
pub struct StepperProps {
    pub value: i32,
    #[props(default)]
    pub min: i32,
    #[props(default = 20)]
    pub max: i32,
    #[props(default)]
    pub onchange: EventHandler<i32>,
}

/// − / value / + counter.
#[component]
pub fn Stepper(props: StepperProps) -> Element {
    let (v, min, max, h) = (props.value, props.min, props.max, props.onchange);
    rsx! {
        span { class: "dodai-stepper",
            button { class: "dodai-btn dodai-btn--outline dodai-btn--sm dodai-btn--icon", "aria-label": "Decrease",
                disabled: v <= min, onclick: move |_| h.call(stepped(v, -1, min, max)), "−" }
            span { "{v}" }
            button { class: "dodai-btn dodai-btn--outline dodai-btn--sm dodai-btn--icon", "aria-label": "Increase",
                disabled: v >= max, onclick: move |_| h.call(stepped(v, 1, min, max)), "+" }
        }
    }
}

/// CSS class suffix and the inline style of each cell of a spinner `size` px across; no cells
/// for the ring.
pub fn spinner_cells(style: SpinnerStyle, size: f32) -> (&'static str, Vec<String>) {
    let kind = match style {
        SpinnerStyle::Ring => "ring",
        SpinnerStyle::Bars => "bars",
        SpinnerStyle::Orbit => "orbit",
        SpinnerStyle::Steps => "steps",
    };
    let cells = busy::cells(style, size)
        .iter()
        .map(|c| {
            // A negative delay starts the cell part-way through its cycle, as the head left it.
            let delay = -(-c.lag).rem_euclid(1.0) / busy::SPIN_RATE;
            format!(
                "left:{:.2}px;top:{:.2}px;width:{:.2}px;height:{:.2}px;animation-delay:{:.3}s",
                c.x, c.y, c.w, c.h, delay
            )
        })
        .collect();
    (kind, cells)
}

/// Spinner of `size` px, in the shape the active design gives it (ring, bars, orbiting dots
/// or stepping squares). Outside a [`crate::ThemeProvider`] it is a ring.
#[component]
pub fn Spinner(#[props(default = 24)] size: u32) -> Element {
    let style = try_use_context::<crate::ThemeHandle>()
        .map(|t| t.theme().look().spinner)
        .unwrap_or_default();
    let (kind, cells) = spinner_cells(style, size as f32);
    rsx! {
        span { class: "dodai-spinner dodai-spinner--{kind}", role: "progressbar", style: "--s:{size}px",
            for c in cells { i { style: "{c}" } }
        }
    }
}

/// Indeterminate bar.
#[component]
pub fn Indeterminate() -> Element {
    rsx! { div { class: "dodai-indeterminate", role: "progressbar", span {} } }
}

/// Loading placeholder; `width == 0` fills the width.
#[component]
pub fn Skeleton(#[props(default)] width: u32, #[props(default = 16)] height: u32) -> Element {
    let w = if width == 0 {
        "100%".to_string()
    } else {
        format!("{width}px")
    };
    rsx! { span { class: "dodai-skeleton", style: "width:{w};height:{height}px;{dodai_topcoat::kit_style(Kit::Skeleton)}background:var(--dodai-kit-skeleton-bg)" } }
}

/// Empty state.
#[component]
pub fn EmptyState(title: String, detail: String) -> Element {
    rsx! {
        div { class: "dodai-empty",
            Icon { icon: IconId::Folder, size: 32 }
            span { class: "dodai-label", "{title}" }
            span { class: "dodai-muted", "{detail}" }
        }
    }
}

/// Properties of [`Badged`].
#[derive(Props, Clone, PartialEq)]
pub struct BadgedProps {
    pub icon: IconId,
    #[props(default)]
    pub count: Option<(u32, Tone)>,
    #[props(default)]
    pub dot: Option<Tone>,
}

/// A ghost icon button with a count or dot badge.
#[component]
pub fn Badged(props: BadgedProps) -> Element {
    rsx! {
        span { class: "dodai-badged",
            button { class: "dodai-btn dodai-btn--ghost dodai-btn--md dodai-btn--icon", Icon { icon: props.icon, size: 18 } }
            if let Some((n, tone)) = props.count { KitBox { kit: Kit::Count(tone), "{n}" } }
            else if let Some(tone) = props.dot { KitBox { kit: Kit::Dot(Some(tone)), span {} } }
        }
    }
}

/// A tooltip bubble shown in place.
#[component]
pub fn TooltipBubble(text: String, #[props(default)] rich: bool) -> Element {
    rsx! { KitBox { kit: Kit::Tooltip { rich }, "{text}" } }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_spinner_style_has_its_class_and_one_styled_cell_per_shape() {
        let n = |st| spinner_cells(st, 24.0);
        assert_eq!(n(SpinnerStyle::Ring), ("ring", vec![]));
        assert_eq!(n(SpinnerStyle::Bars).1.len(), 8);
        assert_eq!(n(SpinnerStyle::Orbit).1.len(), 8);
        assert_eq!(n(SpinnerStyle::Steps).1.len(), 4);
        let (kind, cells) = n(SpinnerStyle::Steps);
        assert_eq!(kind, "steps");
        // The head starts on the first cell: its cycle is at the start (no delay), the rest are
        // part-way through.
        assert!(cells[0].ends_with("animation-delay:0.000s"), "{}", cells[0]);
        assert!(
            cells[1].ends_with("animation-delay:-0.833s"),
            "{}",
            cells[1]
        );
        assert!(cells
            .iter()
            .all(|c| c.contains("left:") && c.contains("animation-delay:")));
    }

    #[test]
    fn stepper_stays_inside_its_range() {
        assert_eq!(stepped(0, -1, 0, 20), 0);
        assert_eq!(stepped(20, 1, 0, 20), 20);
        assert_eq!(stepped(5, 1, 0, 20), 6);
        assert_eq!(stepped(5, -9, 0, 20), 0);
    }

    #[test]
    fn rating_fills_whole_stars_and_rounds_halves_up() {
        assert_eq!(star_fills(3.5, 5), vec![true, true, true, true, false]);
        assert_eq!(star_fills(0.0, 3), vec![false; 3]);
        assert_eq!(star_fills(9.0, 3), vec![true; 3]);
        assert_eq!(star_fills(-2.0, 2), vec![false; 2]);
    }

    #[test]
    fn pagination_shows_a_window_with_gaps() {
        assert!(page_slots(0, 0).is_empty());
        assert_eq!(page_slots(2, 7).len(), 7);
        let s = page_slots(4, 20);
        assert_eq!((s[0], s[s.len() - 1]), (Some(0), Some(19)));
        assert!(s.contains(&None) && s.contains(&Some(4)));
    }

    #[test]
    fn kit_classes_mark_cards_popups_and_dots() {
        assert!(kit_class(Kit::Card(CardVariant::Elevated)).contains("dodai-kit--elevated"));
        assert!(!kit_class(Kit::Card(CardVariant::Filled)).contains("dodai-kit--elevated"));
        assert!(kit_class(Kit::Popup).contains("dodai-kit--popup"));
        assert!(kit_class(Kit::Dot(None)).contains("dodai-kit--dot"));
        assert_eq!(kit_class(Kit::Code), "dodai-kit");
    }

    #[test]
    fn alert_icons_follow_the_tone() {
        assert_eq!(alert_icon(Tone::Danger), IconId::Warning);
        assert_eq!(alert_icon(Tone::Success), IconId::CheckCircle);
        assert_eq!(alert_icon(Tone::Info), IconId::Info);
    }

    fn render(el: fn() -> Element) -> String {
        let mut dom = VirtualDom::new(el);
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }

    #[test]
    fn components_render_their_markup() {
        let html = render(|| {
            rsx! {
                Card { title: "T".to_string(), variant: CardVariant::Elevated, "body" }
                Chip { label: "Filter".to_string(), selected: true }
                Pagination { pages: 3, page: 1 }
                DataTable { columns: vec!["A".into(), "B".into()], rows: vec![vec!["1".into(), "2".into()]] }
                Steps { steps: vec![StepItem { title: "One".into(), detail: None }, StepItem { title: "Two".into(), detail: Some("d".into()) }], current: 1 }
                Skeleton { width: 0 }
                Stepper { value: 0 }
            }
        });
        for want in [
            "dodai-kit--card",
            "--kb:var(--dodai-kit-card-elevated-bg)",
            "dodai-kit--press",
            "--dodai-kit-chip-on-bg",
            "dodai-pager",
            "dodai-datatable",
            "dodai-steps__line",
            "dodai-skeleton",
            "disabled",
        ] {
            assert!(html.contains(want), "missing {want} in {html}");
        }
        // 3 page buttons plus previous and next.
        assert_eq!(html.matches("dodai-kit--press").count(), 1 + 3);
    }
}

/// Segmented-control properties.
#[derive(Props, Clone, PartialEq)]
pub struct SegmentedProps {
    /// The options.
    pub labels: Vec<String>,
    /// The picked one.
    pub selected: usize,
    /// Called with the index of the option pressed.
    #[props(default)]
    pub onselect: EventHandler<usize>,
}

/// A segmented control: the options in one bordered strip ([`Kit::Segmented`]), the picked one
/// filled ([`Kit::Segment`]) with a check on tonal designs.
#[component]
pub fn Segmented(props: SegmentedProps) -> Element {
    let Some(theme) = try_use_context::<crate::ThemeHandle>().map(|h| h.theme()) else {
        return rsx! {};
    };
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
    let (sel, on) = (props.selected, props.onselect);
    rsx! {
        KitBox { kit: Kit::Segmented, style: "display:inline-flex;align-self:flex-start;flex:none;gap:{gap}px",
            for (i, l) in props.labels.iter().enumerate() {
                KitBox { key: "{i}", kit: Kit::Segment { selected: i == sel },
                    style: "height:{h}px;display:flex;align-items:center;white-space:nowrap;{upper}font-size:{size}px",
                    onclick: move |_| on.call(i),
                    if i == sel && tonal { "\u{2713} " }
                    "{l}"
                }
            }
        }
    }
}
