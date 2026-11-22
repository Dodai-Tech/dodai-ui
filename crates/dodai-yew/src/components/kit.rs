//! Token-styled boxes and the composites built on them: card, alert, snackbar, chip, avatar,
//! code. Looks come from `dodai_core::Kit` through the stylesheet variables of `dodai-topcoat`
//! ([`dodai_topcoat::kit_style`]), so every design restyles them.

use crate::{use_glass, use_theme};
use dodai_core::{CardVariant, Kit, SurfaceClass, Tone};
use dodai_icons::IconId;
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
    fn an_alert_icon_follows_the_tone() {
        assert_eq!(alert_icon(Tone::Warning), IconId::Warning);
        assert_eq!(alert_icon(Tone::Info), IconId::Info);
    }
}
