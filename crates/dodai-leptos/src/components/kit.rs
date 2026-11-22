//! Token-styled boxes and the composites built on them: card, alert, snackbar, chip, avatar,
//! code. Looks come from `dodai_core::Kit` through the stylesheet variables of `dodai-topcoat`
//! ([`dodai_topcoat::kit_style`]), so every design restyles them.

use dodai_core::{CardVariant, Kit, StepState, Tone};
use dodai_icons::IconId;
pub use dodai_topcoat::html::{page_slots, star_fills};
use leptos::prelude::*;

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

/// An icon as inline SVG in the current colour, drawn in the active design's style. Empty
/// outside a [`ThemeProvider`](crate::ThemeProvider).
#[component]
pub fn Icon(
    icon: IconId,
    #[prop(default = 18)] size: u32,
    #[prop(optional)] filled: bool,
) -> impl IntoView {
    let svg = use_context::<crate::GlassContext>()
        .and_then(|g| dodai_icons::glyph_for_theme(icon, &g.theme, filled))
        .map(|g| g.svg(size, "currentColor"))
        .unwrap_or_default();
    view! { <span class="dodai-icon" style="display:inline-flex;line-height:0" inner_html=svg></span> }
}

/// A box that looks like `kit`; with `pressable` it is a button and hovers.
#[component]
pub fn KitBox(
    kit: Kit,
    /// Extra inline style (size, position).
    #[prop(optional, into)]
    style: String,
    #[prop(optional)] pressable: bool,
    /// The box's own surface override (frost, fill opacity).
    #[prop(optional)]
    surface: dodai_core::Surface,
    children: Children,
) -> impl IntoView {
    let (glass, vars) = kit
        .surface_class()
        .map_or(("", String::new()), |c| crate::glass(c, &surface));
    let style = format!("{}{style}{vars}", dodai_topcoat::kit_style(kit));
    let class = format!("{}{glass}", kit_class(kit));
    if pressable {
        view! { <button class=format!("{class} dodai-kit--press") style=style>{children()}</button> }
            .into_any()
    } else {
        view! { <div class=class style=style>{children()}</div> }.into_any()
    }
}

/// Card: elevated, filled or outlined.
#[component]
pub fn Card(
    #[prop(optional)] variant: CardVariant,
    #[prop(optional, into)] title: String,
    #[prop(optional)] surface: dodai_core::Surface,
    children: Children,
) -> impl IntoView {
    view! {
        <KitBox kit=Kit::Card(variant) surface=surface>
            {(!title.is_empty()).then(|| view! { <span class="dodai-label">{title}</span> })}
            {children()}
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

/// Message bar in a status tone.
#[component]
pub fn Alert(tone: Tone, #[prop(optional)] banner: bool, children: Children) -> impl IntoView {
    view! {
        <KitBox kit=Kit::Alert { tone, banner } style="width:100%;">
            <Icon icon=alert_icon(tone) size=18 />
            <span>{children()}</span>
        </KitBox>
    }
}

/// Snackbar with one action.
#[component]
pub fn Snackbar(#[prop(into)] text: String, #[prop(into)] action: String) -> impl IntoView {
    view! {
        <KitBox kit=Kit::Snackbar style="gap:var(--dodai-space-lg);width:fit-content;">
            <span>{text}</span>
            <button class="dodai-btn dodai-btn--ghost">{action}</button>
        </KitBox>
    }
}

/// Filter chip.
#[component]
pub fn Chip(
    #[prop(into)] label: String,
    #[prop(optional)] selected: bool,
    #[prop(optional)] icon: Option<IconId>,
) -> impl IntoView {
    view! {
        <KitBox kit=Kit::Chip { selected } pressable=true>
            {icon.map(|i| view! { <Icon icon=i size=16 /> })}
            <span>{label}</span>
        </KitBox>
    }
}

/// Round avatar with initials.
#[component]
pub fn Avatar(#[prop(into)] initials: String, #[prop(default = 40)] size: u32) -> impl IntoView {
    let style = format!(
        "{}width:{size}px;height:{size}px;",
        dodai_topcoat::kit_style(Kit::Avatar)
    );
    view! { <div class="dodai-kit dodai-avatar" style=style>{initials}</div> }
}

/// Code listing.
#[component]
pub fn CodeBlock(#[prop(into)] code: String) -> impl IntoView {
    view! {
        <KitBox kit=Kit::Code style="display:block;">
            <pre class="dodai-code">{code}</pre>
        </KitBox>
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
pub fn Steps(steps: Vec<StepItem>, current: usize) -> impl IntoView {
    let n = steps.len();
    let items = steps
        .into_iter()
        .enumerate()
        .map(|(i, s)| {
            let done = StepState::of(i, current) == StepState::Done;
            view! {
                <KitBox kit=Kit::StepDot(StepState::of(i, current)) style="width:28px;height:28px;padding:0;justify-content:center;">
                    {if done {
                        view! { <Icon icon=IconId::Check size=16 /> }.into_any()
                    } else {
                        view! { <span class="dodai-sm">{i + 1}</span> }.into_any()
                    }}
                </KitBox>
                <div class="dodai-col-text">
                    <span class="dodai-label">{s.title}</span>
                    {s.detail.map(|d| view! { <span class="dodai-caption">{d}</span> })}
                </div>
                {(i + 1 < n).then(|| view! { <span class="dodai-steps__line"></span> })}
            }
        })
        .collect_view();
    view! { <div class="dodai-steps">{items}</div> }
}

/// Breadcrumb trail; the last item is the current page.
#[component]
pub fn Breadcrumb(items: Vec<String>) -> impl IntoView {
    let n = items.len();
    let crumbs = items
        .into_iter()
        .enumerate()
        .map(|(i, it)| {
            view! {
                <span aria-current=if i + 1 == n { "page" } else { "false" }>{it}</span>
                {(i + 1 < n).then(|| view! { <Icon icon=IconId::ChevronRight size=14 /> })}
            }
        })
        .collect_view();
    view! { <nav class="dodai-crumbs" aria-label="Breadcrumb">{crumbs}</nav> }
}

/// Page selector with previous and next; `on_select` gets the page to show.
#[component]
pub fn Pagination(
    pages: usize,
    page: usize,
    #[prop(optional, into)] on_select: Option<Callback<usize>>,
) -> impl IntoView {
    let go = move |to: usize| {
        if let Some(cb) = on_select {
            cb.run(to);
        }
    };
    let slots = page_slots(page, pages)
        .into_iter()
        .map(|slot| match slot {
            Some(i) => view! {
                <button
                    class=format!("{} dodai-kit--press", kit_class(Kit::Page { selected: i == page }))
                    style=dodai_topcoat::kit_style(Kit::Page { selected: i == page })
                    on:click=move |_| go(i)
                >
                    {i + 1}
                </button>
            }
            .into_any(),
            None => view! { <span class="dodai-muted">"…"</span> }.into_any(),
        })
        .collect_view();
    view! {
        <nav class="dodai-pager" aria-label="Pagination">
            <button class="dodai-btn dodai-btn--ghost dodai-btn--sm dodai-btn--icon" disabled=page == 0 on:click=move |_| go(page.saturating_sub(1))>
                <Icon icon=IconId::ChevronLeft size=16 />
            </button>
            {slots}
            <button class="dodai-btn dodai-btn--ghost dodai-btn--sm dodai-btn--icon" disabled=page + 1 >= pages on:click=move |_| go((page + 1).min(pages.saturating_sub(1)))>
                <Icon icon=IconId::ChevronRight size=16 />
            </button>
        </nav>
    }
}

/// Data table with a header row and banded body rows.
#[component]
pub fn DataTable(columns: Vec<String>, rows: Vec<Vec<String>>) -> impl IntoView {
    let cols = columns.len();
    let grid = format!("display:grid;grid-template-columns:repeat({cols},1fr);");
    let head = columns
        .into_iter()
        .map(|c| view! { <span class="dodai-label">{c}</span> })
        .collect_view();
    let body = rows
        .into_iter()
        .enumerate()
        .map(|(i, r)| {
            let cells = r
                .into_iter()
                .map(|c| view! { <span>{c}</span> })
                .collect_view();
            view! { <KitBox kit=Kit::TableRow(i % 2 == 1) style=grid.clone()>{cells}</KitBox> }
        })
        .collect_view();
    view! {
        <div class="dodai-datatable" role="table">
            <KitBox kit=Kit::TableHead style=format!("--cols:{cols};{grid}")>{head}</KitBox>
            {body}
        </div>
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
pub fn Timeline(events: Vec<TimelineItem>) -> impl IntoView {
    let items = events
        .into_iter()
        .map(|e| {
            view! {
                <div class="dodai-timeline__item">
                    <KitBox kit=Kit::Dot(e.tone)>
                        <div></div>
                    </KitBox>
                    <div class="dodai-col-text">
                        <span>{e.title}</span>
                        <span class="dodai-caption">{e.time}</span>
                        {e.detail.map(|d| view! { <span class="dodai-muted">{d}</span> })}
                    </div>
                </div>
            }
        })
        .collect_view();
    view! { <div class="dodai-timeline">{items}</div> }
}

/// Star rating.
#[component]
pub fn Rating(value: f32, #[prop(default = 5)] max: u8) -> impl IntoView {
    let stars = star_fills(value, max)
        .into_iter()
        .map(|on| view! { <Icon icon=IconId::Star size=20 filled=on /> })
        .collect_view();
    view! { <span class="dodai-rating" role="img" aria-label=format!("{value} of {max}")>{stars}</span> }
}

/// The value after pressing a stepper button, kept inside `min..=max`.
pub fn stepped(value: i32, by: i32, min: i32, max: i32) -> i32 {
    (value + by).clamp(min, max)
}

/// − / value / + counter; `on_change` gets the new value.
#[component]
pub fn Stepper(
    value: i32,
    #[prop(default = 0)] min: i32,
    #[prop(default = 20)] max: i32,
    #[prop(optional, into)] on_change: Option<Callback<i32>>,
) -> impl IntoView {
    let go = move |by: i32| {
        if let Some(cb) = on_change {
            cb.run(stepped(value, by, min, max));
        }
    };
    view! {
        <span class="dodai-stepper">
            <button class="dodai-btn dodai-btn--outline dodai-btn--sm dodai-btn--icon" aria-label="Decrease" disabled=value <= min on:click=move |_| go(-1)>"−"</button>
            <span>{value}</span>
            <button class="dodai-btn dodai-btn--outline dodai-btn--sm dodai-btn--icon" aria-label="Increase" disabled=value >= max on:click=move |_| go(1)>"+"</button>
        </span>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn html(v: impl IntoView) -> String {
        v.to_html()
    }

    #[test]
    fn a_card_is_a_kit_box_with_its_title() {
        let h = html(view! { <Card title="Hi">"x"</Card> });
        assert!(h.contains("dodai-kit--card") && h.contains("Hi"));
    }

    #[test]
    fn a_chip_is_a_pressable_box() {
        let h = html(view! { <Chip label="Tag" selected=true /> });
        assert!(h.contains("dodai-kit--press") && h.contains("Tag"));
    }

    #[test]
    fn a_stepper_stays_in_range() {
        assert_eq!(stepped(5, 1, 0, 5), 5);
        assert_eq!(stepped(0, -1, 0, 5), 0);
        assert_eq!(stepped(2, 1, 0, 5), 3);
    }

    #[test]
    fn a_table_has_a_head_and_banded_rows() {
        let h = html(
            view! { <DataTable columns=vec!["A".into()] rows=vec![vec!["1".into()], vec!["2".into()]] /> },
        );
        assert!(h.contains("dodai-datatable") && h.contains("1") && h.contains("2"));
    }

    #[test]
    fn an_alert_shows_its_tone_icon_outside_a_provider_as_nothing() {
        assert_eq!(alert_icon(Tone::Success), IconId::CheckCircle);
    }
}
