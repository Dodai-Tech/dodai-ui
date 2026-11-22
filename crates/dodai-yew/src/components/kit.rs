//! Token-styled boxes and the composites built on them: card, alert, snackbar, chip, avatar,
//! code. Looks come from `dodai_core::Kit` through the stylesheet variables of `dodai-topcoat`
//! ([`dodai_topcoat::kit_style`]), so every design restyles them.

use crate::{use_glass, use_theme};
use dodai_core::{CardVariant, Kit, StepState, SurfaceClass, Tone};
use dodai_icons::IconId;
pub use dodai_topcoat::html::{page_slots, star_fills};
use yew::prelude::*;

/// The classes of a box that looks like `kit`.
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

#[derive(Properties, PartialEq)]
pub struct IconProps {
    pub icon: IconId,
    #[prop_or(18)]
    pub size: u32,
    #[prop_or_default]
    pub filled: bool,
}

/// An icon as inline SVG in the current colour, drawn in the active design's style. Empty
/// outside a [`ThemeProvider`](crate::ThemeProvider).
#[function_component]
pub fn Icon(props: &IconProps) -> Html {
    let svg = use_theme()
        .and_then(|c| dodai_icons::glyph_for_theme(props.icon, &c.theme, props.filled))
        .map(|g| g.svg(props.size, "currentColor"))
        .unwrap_or_default();
    html! { <span class="dodai-icon" style="display:inline-flex;line-height:0">{ Html::from_html_unchecked(AttrValue::from(svg)) }</span> }
}

#[derive(Properties, PartialEq)]
pub struct KitBoxProps {
    pub kit: Kit,
    /// Extra inline style (size, position).
    #[prop_or_default]
    pub style: AttrValue,
    /// Makes the box a button.
    #[prop_or_default]
    pub pressable: bool,
    /// The box's own surface override (frost, fill opacity).
    #[prop_or_default]
    pub surface: dodai_core::Surface,
    #[prop_or_default]
    pub children: Html,
}

/// A box that looks like `kit`; with `pressable` it is a button and hovers.
#[function_component]
pub fn KitBox(props: &KitBoxProps) -> Html {
    let (glass, vars) = use_glass(
        props.kit.surface_class().unwrap_or(SurfaceClass::Panel),
        &props.surface,
    );
    let (glass, vars) = if props.kit.surface_class().is_some() {
        (glass, vars)
    } else {
        ("", None)
    };
    let style = format!(
        "{}{}{}",
        dodai_topcoat::kit_style(props.kit),
        props.style,
        vars.unwrap_or_default()
    );
    let class = format!("{}{glass}", kit_class(props.kit));
    if props.pressable {
        html! { <button class={format!("{class} dodai-kit--press")} {style}>{ props.children.clone() }</button> }
    } else {
        html! { <div {class} {style}>{ props.children.clone() }</div> }
    }
}

#[derive(Properties, PartialEq)]
pub struct CardProps {
    #[prop_or_default]
    pub variant: CardVariant,
    #[prop_or_default]
    pub title: AttrValue,
    #[prop_or_default]
    pub surface: dodai_core::Surface,
    #[prop_or_default]
    pub children: Html,
}

/// Card: elevated, filled or outlined.
#[function_component]
pub fn Card(props: &CardProps) -> Html {
    html! {
        <KitBox kit={Kit::Card(props.variant)} surface={props.surface}>
            if !props.title.is_empty() { <span class="dodai-label">{ props.title.clone() }</span> }
            { props.children.clone() }
        </KitBox>
    }
}

/// The icon an alert of `tone` shows.
pub fn alert_icon(tone: Tone) -> IconId {
    match tone {
        Tone::Danger | Tone::Warning => IconId::Warning,
        Tone::Success => IconId::CheckCircle,
        Tone::Info => IconId::Info,
    }
}

#[derive(Properties, PartialEq)]
pub struct AlertProps {
    pub tone: Tone,
    /// Full-width solid banner.
    #[prop_or_default]
    pub banner: bool,
    #[prop_or_default]
    pub children: Html,
}

/// Message bar in a status tone.
#[function_component]
pub fn Alert(props: &AlertProps) -> Html {
    html! {
        <KitBox kit={Kit::Alert { tone: props.tone, banner: props.banner }} style="width:100%;">
            <Icon icon={alert_icon(props.tone)} size={18} />
            <span>{ props.children.clone() }</span>
        </KitBox>
    }
}

#[derive(Properties, PartialEq)]
pub struct SnackbarProps {
    pub text: AttrValue,
    pub action: AttrValue,
}

/// Snackbar with one action.
#[function_component]
pub fn Snackbar(props: &SnackbarProps) -> Html {
    html! {
        <KitBox kit={Kit::Snackbar} style="gap:var(--dodai-space-lg);width:fit-content;">
            <span>{ props.text.clone() }</span>
            <button class="dodai-btn dodai-btn--ghost">{ props.action.clone() }</button>
        </KitBox>
    }
}

#[derive(Properties, PartialEq)]
pub struct ChipProps {
    pub label: AttrValue,
    #[prop_or_default]
    pub selected: bool,
    #[prop_or_default]
    pub icon: Option<IconId>,
}

/// Filter chip.
#[function_component]
pub fn Chip(props: &ChipProps) -> Html {
    html! {
        <KitBox kit={Kit::Chip { selected: props.selected }} pressable=true>
            if let Some(i) = props.icon { <Icon icon={i} size={16} /> }
            <span>{ props.label.clone() }</span>
        </KitBox>
    }
}

#[derive(Properties, PartialEq)]
pub struct AvatarProps {
    pub initials: AttrValue,
    #[prop_or(40)]
    pub size: u32,
}

/// Round avatar with initials.
#[function_component]
pub fn Avatar(props: &AvatarProps) -> Html {
    let style = format!(
        "{}width:{s}px;height:{s}px;",
        dodai_topcoat::kit_style(Kit::Avatar),
        s = props.size
    );
    html! { <div class="dodai-kit dodai-avatar" {style}>{ props.initials.clone() }</div> }
}

#[derive(Properties, PartialEq)]
pub struct CodeBlockProps {
    pub code: AttrValue,
}

/// Code listing.
#[function_component]
pub fn CodeBlock(props: &CodeBlockProps) -> Html {
    html! {
        <KitBox kit={Kit::Code} style="display:block;">
            <pre class="dodai-code">{ props.code.clone() }</pre>
        </KitBox>
    }
}

/// One step of [`Steps`].
#[derive(Clone, PartialEq)]
pub struct StepItem {
    pub title: AttrValue,
    pub detail: Option<AttrValue>,
}

#[derive(Properties, PartialEq)]
pub struct StepsProps {
    pub steps: Vec<StepItem>,
    pub current: usize,
}

/// Numbered step indicator; steps before `current` show a check.
#[function_component]
pub fn Steps(props: &StepsProps) -> Html {
    let n = props.steps.len();
    html! {
        <div class="dodai-steps">
            { for props.steps.iter().enumerate().map(|(i, s)| {
                let state = StepState::of(i, props.current);
                html! {
                    <>
                        <KitBox kit={Kit::StepDot(state)} style="width:28px;height:28px;padding:0;justify-content:center;">
                            if state == StepState::Done {
                                <Icon icon={IconId::Check} size={16} />
                            } else {
                                <span class="dodai-sm">{ i + 1 }</span>
                            }
                        </KitBox>
                        <div class="dodai-col-text">
                            <span class="dodai-label">{ s.title.clone() }</span>
                            if let Some(d) = &s.detail { <span class="dodai-caption">{ d.clone() }</span> }
                        </div>
                        if i + 1 < n { <span class="dodai-steps__line"></span> }
                    </>
                }
            }) }
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct BreadcrumbProps {
    pub items: Vec<AttrValue>,
}

/// Breadcrumb trail; the last item is the current page.
#[function_component]
pub fn Breadcrumb(props: &BreadcrumbProps) -> Html {
    let n = props.items.len();
    html! {
        <nav class="dodai-crumbs" aria-label="Breadcrumb">
            { for props.items.iter().enumerate().map(|(i, it)| html! {
                <>
                    <span aria-current={ if i + 1 == n { "page" } else { "false" } }>{ it.clone() }</span>
                    if i + 1 < n { <Icon icon={IconId::ChevronRight} size={14} /> }
                </>
            }) }
        </nav>
    }
}

#[derive(Properties, PartialEq)]
pub struct PaginationProps {
    pub pages: usize,
    pub page: usize,
    /// Gets the page to show.
    #[prop_or_default]
    pub onselect: Callback<usize>,
}

/// Page selector with previous and next.
#[function_component]
pub fn Pagination(props: &PaginationProps) -> Html {
    let (pages, page) = (props.pages, props.page);
    let go = |to: usize| {
        let cb = props.onselect.clone();
        Callback::from(move |_: MouseEvent| cb.emit(to))
    };
    html! {
        <nav class="dodai-pager" aria-label="Pagination">
            <button class="dodai-btn dodai-btn--ghost dodai-btn--sm dodai-btn--icon" disabled={page == 0} onclick={go(page.saturating_sub(1))}>
                <Icon icon={IconId::ChevronLeft} size={16} />
            </button>
            { for page_slots(page, pages).into_iter().map(|slot| match slot {
                Some(i) => {
                    let kit = Kit::Page { selected: i == page };
                    html! { <button class={format!("{} dodai-kit--press", kit_class(kit))} style={dodai_topcoat::kit_style(kit)} onclick={go(i)}>{ i + 1 }</button> }
                }
                None => html! { <span class="dodai-muted">{ "…" }</span> },
            }) }
            <button class="dodai-btn dodai-btn--ghost dodai-btn--sm dodai-btn--icon" disabled={page + 1 >= pages} onclick={go((page + 1).min(pages.saturating_sub(1)))}>
                <Icon icon={IconId::ChevronRight} size={16} />
            </button>
        </nav>
    }
}

#[derive(Properties, PartialEq)]
pub struct DataTableProps {
    pub columns: Vec<AttrValue>,
    pub rows: Vec<Vec<AttrValue>>,
}

/// Data table with a header row and banded body rows.
#[function_component]
pub fn DataTable(props: &DataTableProps) -> Html {
    let cols = props.columns.len();
    let grid = format!("display:grid;grid-template-columns:repeat({cols},1fr);");
    html! {
        <div class="dodai-datatable" role="table">
            <KitBox kit={Kit::TableHead} style={format!("--cols:{cols};{grid}")}>
                { for props.columns.iter().map(|c| html! { <span class="dodai-label">{ c.clone() }</span> }) }
            </KitBox>
            { for props.rows.iter().enumerate().map(|(i, r)| html! {
                <KitBox kit={Kit::TableRow(i % 2 == 1)} style={grid.clone()}>
                    { for r.iter().map(|c| html! { <span>{ c.clone() }</span> }) }
                </KitBox>
            }) }
        </div>
    }
}

/// One event of [`Timeline`].
#[derive(Clone, PartialEq)]
pub struct TimelineItem {
    pub title: AttrValue,
    pub time: AttrValue,
    pub detail: Option<AttrValue>,
    pub tone: Option<Tone>,
}

#[derive(Properties, PartialEq)]
pub struct TimelineProps {
    pub events: Vec<TimelineItem>,
}

/// Events down a line.
#[function_component]
pub fn Timeline(props: &TimelineProps) -> Html {
    html! {
        <div class="dodai-timeline">
            { for props.events.iter().map(|e| html! {
                <div class="dodai-timeline__item">
                    <KitBox kit={Kit::Dot(e.tone)}><div></div></KitBox>
                    <div class="dodai-col-text">
                        <span>{ e.title.clone() }</span>
                        <span class="dodai-caption">{ e.time.clone() }</span>
                        if let Some(d) = &e.detail { <span class="dodai-muted">{ d.clone() }</span> }
                    </div>
                </div>
            }) }
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct RatingProps {
    pub value: f32,
    #[prop_or(5)]
    pub max: u8,
}

/// Star rating.
#[function_component]
pub fn Rating(props: &RatingProps) -> Html {
    html! {
        <span class="dodai-rating" role="img" aria-label={format!("{} of {}", props.value, props.max)}>
            { for star_fills(props.value, props.max).into_iter().map(|on| html! { <Icon icon={IconId::Star} size={20} filled={on} /> }) }
        </span>
    }
}

/// The value after pressing a stepper button, kept inside `min..=max`.
pub fn stepped(value: i32, by: i32, min: i32, max: i32) -> i32 {
    (value + by).clamp(min, max)
}

#[derive(Properties, PartialEq)]
pub struct StepperProps {
    pub value: i32,
    #[prop_or_default]
    pub min: i32,
    #[prop_or(20)]
    pub max: i32,
    #[prop_or_default]
    pub onchange: Callback<i32>,
}

/// − / value / + counter.
#[function_component]
pub fn Stepper(props: &StepperProps) -> Html {
    let (v, min, max) = (props.value, props.min, props.max);
    let go = |by: i32| {
        let cb = props.onchange.clone();
        Callback::from(move |_: MouseEvent| cb.emit(stepped(v, by, min, max)))
    };
    html! {
        <span class="dodai-stepper">
            <button class="dodai-btn dodai-btn--outline dodai-btn--sm dodai-btn--icon" aria-label="Decrease" disabled={v <= min} onclick={go(-1)}>{ "−" }</button>
            <span>{ v }</span>
            <button class="dodai-btn dodai-btn--outline dodai-btn--sm dodai-btn--icon" aria-label="Increase" disabled={v >= max} onclick={go(1)}>{ "+" }</button>
        </span>
    }
}

/// A node of [`TreeView`].
#[derive(Clone, PartialEq)]
pub struct TreeItem {
    pub label: AttrValue,
    pub icon: Option<IconId>,
    pub children: Vec<TreeItem>,
    pub open: bool,
    /// Drawn as the selected row.
    pub selected: bool,
}

fn tree_branch(item: &TreeItem) -> Html {
    let leaf = item.children.is_empty();
    let row = html! {
        <KitBox kit={Kit::Row { selected: item.selected }}>
            if leaf { <span style="width:14px"></span> }
            else { <span class="dodai-tree__chev"><Icon icon={IconId::ChevronRight} size={14} /></span> }
            if let Some(i) = item.icon { <Icon icon={i} size={16} /> }
            <span>{ item.label.clone() }</span>
        </KitBox>
    };
    if leaf {
        return row;
    }
    html! {
        <details open={item.open}>
            <summary>{ row }</summary>
            <div class="dodai-tree__children">{ for item.children.iter().map(tree_branch) }</div>
        </details>
    }
}

#[derive(Properties, PartialEq)]
pub struct TreeViewProps {
    pub items: Vec<TreeItem>,
}

/// Expandable tree.
#[function_component]
pub fn TreeView(props: &TreeViewProps) -> Html {
    html! { <div class="dodai-tree">{ for props.items.iter().map(tree_branch) }</div> }
}

#[derive(Properties, PartialEq)]
pub struct BadgeProps {
    #[prop_or_default]
    pub children: Html,
}

/// Count badge.
#[function_component]
pub fn Badge(props: &BadgeProps) -> Html {
    html! { <span class="dodai-badge">{ props.children.clone() }</span> }
}

#[derive(Properties, PartialEq)]
pub struct SkeletonProps {
    /// `0` fills the width.
    #[prop_or_default]
    pub width: u32,
    #[prop_or(16)]
    pub height: u32,
}

/// Loading placeholder.
#[function_component]
pub fn Skeleton(props: &SkeletonProps) -> Html {
    let w = if props.width == 0 {
        "100%".to_string()
    } else {
        format!("{}px", props.width)
    };
    let style = format!(
        "width:{w};height:{}px;{}background:var(--dodai-kit-skeleton-bg)",
        props.height,
        dodai_topcoat::kit_style(Kit::Skeleton)
    );
    html! { <span class="dodai-skeleton" {style}></span> }
}

#[derive(Properties, PartialEq)]
pub struct EmptyStateProps {
    pub title: AttrValue,
    pub detail: AttrValue,
}

/// Empty state.
#[function_component]
pub fn EmptyState(props: &EmptyStateProps) -> Html {
    html! {
        <div class="dodai-empty">
            <Icon icon={IconId::Folder} size={32} />
            <span class="dodai-label">{ props.title.clone() }</span>
            <span class="dodai-muted">{ props.detail.clone() }</span>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct BadgedProps {
    pub icon: IconId,
    #[prop_or_default]
    pub count: Option<(u32, Tone)>,
    #[prop_or_default]
    pub dot: Option<Tone>,
}

/// A ghost icon button with a count or dot badge.
#[function_component]
pub fn Badged(props: &BadgedProps) -> Html {
    html! {
        <span class="dodai-badged">
            <button class="dodai-btn dodai-btn--ghost dodai-btn--md dodai-btn--icon">
                <Icon icon={props.icon} size={18} />
            </button>
            if let Some((n, tone)) = props.count { <KitBox kit={Kit::Count(tone)}>{ n }</KitBox> }
            else if let Some(tone) = props.dot { <KitBox kit={Kit::Dot(Some(tone))}><span></span></KitBox> }
        </span>
    }
}

#[derive(Properties, PartialEq)]
pub struct SegmentedProps {
    pub labels: Vec<AttrValue>,
    pub selected: usize,
    /// Gets the index of the option pressed.
    #[prop_or_default]
    pub onselect: Callback<usize>,
}

/// A segmented control: the options in one bordered strip, the picked one filled with a check
/// on tonal designs. Empty outside a [`ThemeProvider`](crate::ThemeProvider).
#[function_component]
pub fn Segmented(props: &SegmentedProps) -> Html {
    let Some(ctx) = use_theme() else {
        return html! {};
    };
    let theme = &ctx.theme;
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
    html! {
        <KitBox kit={Kit::Segmented} style={format!("display:inline-flex;align-self:flex-start;flex:none;gap:{gap}px")}>
            { for props.labels.iter().enumerate().map(|(i, l)| {
                let kit = Kit::Segment { selected: i == props.selected };
                let style = format!(
                    "{}height:{h}px;display:flex;align-items:center;white-space:nowrap;{upper}font-size:{size}px",
                    dodai_topcoat::kit_style(kit)
                );
                let cb = props.onselect.clone();
                html! {
                    <button class={format!("{} dodai-kit--press", kit_class(kit))} {style} onclick={Callback::from(move |_: MouseEvent| cb.emit(i))}>
                        if i == props.selected && tonal { { "\u{2713} " } }
                        { l.clone() }
                    </button>
                }
            }) }
        </KitBox>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cards_and_popups_get_their_kit_classes() {
        assert!(kit_class(Kit::Card(CardVariant::Elevated)).contains("dodai-kit--elevated"));
        assert!(kit_class(Kit::Popup).contains("dodai-kit--popup"));
        assert_eq!(kit_class(Kit::Code), "dodai-kit");
    }

    #[test]
    fn a_stepper_stays_in_range() {
        assert_eq!(stepped(5, 1, 0, 5), 5);
        assert_eq!(stepped(0, -1, 0, 5), 0);
        assert_eq!(stepped(2, 1, 0, 5), 3);
    }

    #[test]
    fn an_alert_icon_follows_the_tone() {
        assert_eq!(alert_icon(Tone::Warning), IconId::Warning);
        assert_eq!(alert_icon(Tone::Info), IconId::Info);
    }
}
