//! Token-styled boxes and the composites built on them: card, alert, snackbar, chip, avatar,
//! code. Looks come from `dodai_core::Kit` through the stylesheet variables of `dodai-topcoat`
//! ([`dodai_topcoat::kit_style`]), so every design restyles them.

use dodai_core::{CardVariant, Kit, Tone};
use dodai_icons::IconId;
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
    fn an_alert_shows_its_tone_icon_outside_a_provider_as_nothing() {
        assert_eq!(alert_icon(Tone::Success), IconId::CheckCircle);
    }
}
