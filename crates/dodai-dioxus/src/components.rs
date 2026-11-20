//! Typed components emitting the `.dodai-*` classes styled by `dodai-topcoat` CSS.

use dioxus::prelude::*;
pub use dodai_topcoat::html::button_classes;

use dodai_core::{ButtonVariant, ControlSize, Tone};

/// CSS modifier class for a [`ButtonVariant`].
pub fn button_class(v: ButtonVariant) -> &'static str {
    match v {
        ButtonVariant::Primary => "dodai-btn dodai-btn--primary",
        ButtonVariant::Secondary => "dodai-btn dodai-btn--secondary",
        ButtonVariant::Outline => "dodai-btn dodai-btn--outline",
        ButtonVariant::OutlineSecondary => "dodai-btn dodai-btn--outline-secondary",
        ButtonVariant::Danger => "dodai-btn dodai-btn--danger",
        ButtonVariant::Ghost => "dodai-btn dodai-btn--ghost",
        ButtonVariant::Scrim => "dodai-btn dodai-btn--scrim",
    }
}

/// Tag colour: a status [`Tone`] or a brand colour.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum TagKind {
    Primary,
    Secondary,
    #[default]
    Info,
    Success,
    Warning,
    Danger,
}

impl From<Tone> for TagKind {
    fn from(t: Tone) -> Self {
        match t {
            Tone::Info => Self::Info,
            Tone::Success => Self::Success,
            Tone::Warning => Self::Warning,
            Tone::Danger => Self::Danger,
        }
    }
}

impl TagKind {
    fn class(self) -> &'static str {
        match self {
            Self::Primary => "dodai-tag--primary",
            Self::Secondary => "dodai-tag--secondary",
            Self::Info => "",
            Self::Success => "dodai-tag--success",
            Self::Warning => "dodai-tag--warning",
            Self::Danger => "dodai-tag--danger",
        }
    }
}

/// Button properties.
#[derive(Props, Clone, PartialEq)]
pub struct ButtonProps {
    #[props(default = ButtonVariant::Primary)]
    pub variant: ButtonVariant,
    #[props(default)]
    pub disabled: bool,
    /// Size step; `None` keeps the stylesheet's default height.
    #[props(default)]
    pub size: Option<ControlSize>,
    /// Square corners instead of the design's own shape.
    #[props(default)]
    pub square: bool,
    /// No label: the button is as wide as it is high.
    #[props(default)]
    pub icon_only: bool,
    /// Shows the busy ring before the label and ignores clicks.
    #[props(default)]
    pub loading: bool,
    /// Which end is cropped, drawn the way the design crops buttons (`None` = uncropped;
    /// needs a [`crate::ThemeProvider`]).
    #[props(default)]
    pub crop: Option<dodai_core::Crop>,
    /// The button's own surface override (frost, fill opacity); see [`dodai_core::Surface`].
    #[props(default)]
    pub surface: dodai_core::Surface,
    /// A busy shimmer band sweeps across it.
    #[props(default)]
    pub shimmer: bool,
    #[props(default)]
    pub onclick: EventHandler<MouseEvent>,
    pub children: Element,
}

#[component]
pub fn Button(props: ButtonProps) -> Element {
    let mut class = button_classes(props.variant, props.size, props.square, props.icon_only);
    if props.shimmer {
        class.push_str(" dodai-btn--shimmer");
    }
    let glass = crate::provider::glass(Some(dodai_core::SurfaceClass::Button), &props.surface);
    let off = props.disabled || props.loading;
    let cropped = props
        .crop
        .zip(try_use_context::<crate::ThemeHandle>())
        .and_then(|(end, h)| h.theme().button_crop(end).map(|c| (c, h)));
    let mut style = String::new();
    let mut tick = None;
    if let Some((crop, h)) = cropped {
        style = dodai_topcoat::crop_style(&h.theme(), crop);
        if let dodai_core::ButtonCrop::CutTicked { tick: edge, .. } = crop {
            let (side, cls) = match edge {
                dodai_core::CropEdge::Left => ("left", "dodai-tick dodai-tick--left"),
                _ => ("right", "dodai-tick dodai-tick--right"),
            };
            if !props.icon_only {
                style.push_str(&format!(
                    "padding-{side}:calc(var(--dodai-space-lg) + var(--dodai-space-md));"
                ));
            }
            tick = Some(cls);
        }
    }
    if let Some(g) = glass {
        class.push_str(" dodai-glass");
        style.push_str(&g);
    }
    rsx! {
        button { class, style, disabled: off, onclick: move |e| props.onclick.call(e),
            if props.loading { span { class: "dodai-spinner", style: "--s:14px" } }
            {props.children}
            if let Some(cls) = tick { i { class: cls } }
        }
    }
}

/// Panel properties.
#[derive(Props, Clone, PartialEq)]
pub struct PanelProps {
    #[props(default)]
    pub title: Option<String>,
    /// Highlight with the primary border + glow.
    #[props(default)]
    pub focus: bool,
    #[props(default)]
    pub raised: bool,
    /// The panel's own surface override (frost, fill opacity); see [`dodai_core::Surface`].
    #[props(default)]
    pub surface: dodai_core::Surface,
    pub children: Element,
}

/// Card / panel with optional title; chamfered per theme shape.
#[component]
pub fn Panel(props: PanelProps) -> Element {
    let mut class = String::from("dodai-panel");
    if props.focus {
        class.push_str(" dodai-panel--focus");
    }
    if props.raised {
        class.push_str(" dodai-panel--raised");
    }
    let style = crate::provider::glass(Some(dodai_core::SurfaceClass::Panel), &props.surface);
    if style.is_some() {
        class.push_str(" dodai-glass");
    }
    rsx! {
        div { class, style,
            span { class: "dodai-panel__shade", aria_hidden: "true" }
            if let Some(t) = props.title { div { class: "dodai-panel__title", "{t}" } }
            {props.children}
        }
    }
}

/// Tag properties.
#[derive(Props, Clone, PartialEq)]
pub struct TagProps {
    #[props(default)]
    pub kind: TagKind,
    #[props(default)]
    pub soft: bool,
    pub children: Element,
}

/// Status tag.
#[component]
pub fn Tag(props: TagProps) -> Element {
    let soft = if props.soft { "dodai-tag--soft" } else { "" };
    let class = format!("dodai-tag {} {soft}", props.kind.class());
    rsx! { span { class, {props.children} } }
}

/// Count badge.
#[component]
pub fn Badge(children: Element) -> Element {
    rsx! { span { class: "dodai-badge", {children} } }
}

/// Tabs properties.
#[derive(Props, Clone, PartialEq)]
pub struct TabsProps {
    pub tabs: Vec<String>,
    pub active: usize,
    /// An underline under the active tab instead of its fill.
    #[props(default)]
    pub secondary: bool,
    /// Tabs that cannot be picked.
    #[props(default)]
    pub disabled: Vec<usize>,
    /// An icon over each label.
    #[props(default)]
    pub icons: Vec<dodai_icons::IconId>,
    #[props(default)]
    pub onselect: EventHandler<usize>,
    /// A close cross on every tab; the app removes the tab in `onclose`.
    #[props(default)]
    pub closable: bool,
    #[props(default)]
    pub onclose: EventHandler<usize>,
    /// Shows a "+" after the last tab; the app adds the tab in `onadd`.
    #[props(default)]
    pub addable: bool,
    #[props(default)]
    pub onadd: EventHandler<()>,
}

/// The classes of one tab.
fn tab_class(active: bool, disabled: bool, icon: bool) -> String {
    let mut c = String::from("dodai-tab");
    for (on, name) in [
        (active, " dodai-tab--active"),
        (disabled, " dodai-tab--disabled"),
        (icon, " dodai-tab--icon"),
    ] {
        if on {
            c.push_str(name);
        }
    }
    c
}

/// Tab strip; active tab is controlled by `active`.
#[component]
pub fn Tabs(props: TabsProps) -> Element {
    rsx! {
        div { class: if props.secondary { "dodai-tabs dodai-tabs--secondary" } else { "dodai-tabs" }, role: "tablist",
            for (i, t) in props.tabs.iter().enumerate() {
                a {
                    key: "{i}",
                    class: tab_class(i == props.active, props.disabled.contains(&i), props.icons.get(i).is_some()),
                    role: "tab",
                    "aria-selected": "{i == props.active}",
                    "aria-disabled": "{props.disabled.contains(&i)}",
                    href: "#",
                    onclick: move |e| { e.prevent_default(); props.onselect.call(i) },
                    if let Some(ic) = props.icons.get(i) { Icon { icon: *ic, size: 18 } }
                    span { "{t}" }
                    if props.closable {
                        span {
                            class: "dodai-tab__close",
                            role: "button",
                            tabindex: "0",
                            "aria-label": "Close tab",
                            onclick: move |e| { e.prevent_default(); e.stop_propagation(); props.onclose.call(i) },
                            Icon { icon: dodai_icons::IconId::Close, size: 14 }
                        }
                    }
                }
            }
            if props.addable {
                button {
                    r#type: "button",
                    class: "dodai-tab dodai-tab--add",
                    "aria-label": "Add tab",
                    onclick: move |_| props.onadd.call(()),
                    Icon { icon: dodai_icons::IconId::Plus, size: 16 }
                }
            }
        }
    }
}

/// Toggle properties.
#[derive(Props, Clone, PartialEq)]
pub struct CheckProps {
    pub checked: bool,
    #[props(default)]
    pub disabled: bool,
    #[props(default)]
    pub label: String,
    /// A checkbox drawn half-way (a partly picked group).
    #[props(default)]
    pub mixed: bool,
    #[props(default)]
    pub onchange: EventHandler<bool>,
}

/// Switch.
#[component]
pub fn Toggle(props: CheckProps) -> Element {
    rsx! {
        label { class: "dodai-switch",
            input { r#type: "checkbox", role: "switch", checked: props.checked, disabled: props.disabled,
                onchange: move |e| props.onchange.call(e.checked()) }
            span { class: "dodai-switch__box" }
            "{props.label}"
        }
    }
}

/// Checkbox.
#[component]
pub fn Checkbox(props: CheckProps) -> Element {
    rsx! {
        label { class: if props.mixed { "dodai-check dodai-check--mixed" } else { "dodai-check" },
            input { r#type: "checkbox", checked: props.checked, disabled: props.disabled,
                onchange: move |e| props.onchange.call(e.checked()) }
            span { class: "dodai-check__box" }
            "{props.label}"
        }
    }
}

/// Radio properties.
#[derive(Props, Clone, PartialEq)]
pub struct RadioProps {
    pub checked: bool,
    #[props(default)]
    pub name: String,
    #[props(default)]
    pub disabled: bool,
    #[props(default)]
    pub label: String,
    #[props(default)]
    pub onselect: EventHandler<()>,
}

/// Radio button.
#[component]
pub fn Radio(props: RadioProps) -> Element {
    rsx! {
        label { class: "dodai-radio",
            input { r#type: "radio", name: props.name, checked: props.checked, disabled: props.disabled,
                onchange: move |_| props.onselect.call(()) }
            span { class: "dodai-radio__box" }
            "{props.label}"
        }
    }
}

/// Progress properties.
#[derive(Props, Clone, PartialEq)]
pub struct ProgressProps {
    /// 0.0..=1.0
    pub value: f32,
    #[props(default)]
    pub tone: Option<Tone>,
}

/// Progress bar.
#[component]
pub fn Progress(props: ProgressProps) -> Element {
    let pct = props.value.clamp(0.0, 1.0) * 100.0;
    let tone = match props.tone {
        Some(Tone::Success) => "dodai-progress--success",
        Some(Tone::Warning) => "dodai-progress--warning",
        Some(Tone::Danger) => "dodai-progress--danger",
        _ => "",
    };
    rsx! {
        div { class: "dodai-progress {tone}", role: "progressbar", "aria-valuenow": "{pct:.0}",
            span { style: "--v:{pct}%" }
        }
    }
}

/// Slider properties.
#[derive(Props, Clone, PartialEq)]
pub struct SliderProps {
    pub value: f32,
    #[props(default = 0.0)]
    pub min: f32,
    #[props(default = 1.0)]
    pub max: f32,
    #[props(default = 0.01)]
    pub step: f32,
    #[props(default)]
    pub disabled: bool,
    /// Runs bottom to top, `length` px tall.
    #[props(default)]
    pub vertical: bool,
    /// Height in px of a vertical slider.
    #[props(default = 120)]
    pub length: u32,
    /// Tick marks at every step (horizontal).
    #[props(default)]
    pub ticks: bool,
    /// Fills from the middle of the track to the thumb (a balance).
    #[props(default)]
    pub centered: bool,
    /// This text in a pill over the thumb (under it with `bubble_below`).
    #[props(default)]
    pub bubble: Option<String>,
    #[props(default)]
    pub bubble_below: bool,
    #[props(default)]
    pub onchange: EventHandler<f32>,
}

/// Range slider (filled track via `--v`).
#[component]
pub fn Slider(props: SliderProps) -> Element {
    let span = (props.max - props.min).max(f32::EPSILON);
    let pct = ((props.value - props.min) / span).clamp(0.0, 1.0) * 100.0;
    // The filled part runs `a..v`: from the start, or from the middle when centred.
    let (a, v) = if props.centered {
        (pct.min(50.0), pct.max(50.0))
    } else {
        (0.0, pct)
    };
    let (class, style) = if props.vertical {
        (
            "dodai-slider dodai-slider--v",
            format!("--a:{a}%;--v:{v}%;height:{}px", props.length),
        )
    } else {
        ("dodai-slider", format!("--a:{a}%;--v:{v}%"))
    };
    let input = rsx! {
        input { class, r#type: "range", min: "{props.min}", max: "{props.max}", step: "{props.step}",
            value: "{props.value}", disabled: props.disabled, style,
            oninput: move |e| if let Ok(v) = e.value().parse::<f32>() { props.onchange.call(v) } }
    };
    if props.vertical {
        return input;
    }
    // Centre of the thumb at a fraction: the track is inset by half a thumb each side.
    let at = |f: f32| format!("left:calc(8px + {f} * (100% - 16px))");
    let ticks = if props.ticks {
        dodai_core::slider_math::tick_fractions(props.min, props.max, Some(props.step))
    } else {
        Vec::new()
    };
    let room = match (&props.bubble, props.bubble_below) {
        (None, _) => "",
        (Some(_), false) => "padding-top:30px",
        (Some(_), true) => "padding-bottom:30px",
    };
    let bubble_y = if props.bubble_below {
        "top:26px"
    } else {
        "top:0"
    };
    let frac = pct / 100.0;
    rsx! {
        div { class: "dodai-slider-box", style: room,
            {input}
            for (i, f) in ticks.into_iter().enumerate() {
                span { key: "{i}", class: "dodai-slider__tick", style: at(f) }
            }
            if let Some(b) = &props.bubble {
                span { class: "dodai-slider__bubble", style: "{at(frac)};{bubble_y}", "{b}" }
            }
        }
    }
}

/// Range slider properties.
#[derive(Props, Clone, PartialEq)]
pub struct RangeSliderProps {
    /// One thumb.
    pub lo: f32,
    /// The other thumb.
    pub hi: f32,
    #[props(default = 0.0)]
    pub min: f32,
    #[props(default = 1.0)]
    pub max: f32,
    #[props(default = 0.01)]
    pub step: f32,
    /// Runs bottom to top, `length` px tall.
    #[props(default)]
    pub vertical: bool,
    /// Height in px of a vertical range slider.
    #[props(default = 120)]
    pub length: u32,
    /// The new `(low, high)`, always in order.
    #[props(default)]
    pub onchange: EventHandler<(f32, f32)>,
}

/// Two thumbs on one track picking a span; they never cross.
#[component]
pub fn RangeSlider(props: RangeSliderProps) -> Element {
    let (lo, hi) = (props.lo.min(props.hi), props.hi.max(props.lo));
    let span = (props.max - props.min).max(f32::EPSILON);
    let pct = |v: f32| ((v - props.min) / span).clamp(0.0, 1.0) * 100.0;
    let (class, style) = if props.vertical {
        (
            "dodai-range dodai-range--v",
            format!(
                "--a:{}%;--b:{}%;height:{}px",
                pct(lo),
                pct(hi),
                props.length
            ),
        )
    } else {
        ("dodai-range", format!("--a:{}%;--b:{}%", pct(lo), pct(hi)))
    };
    let thumb = if props.vertical {
        "dodai-slider dodai-slider--v"
    } else {
        "dodai-slider"
    };
    rsx! {
        div { class, style,
            input { class: thumb, r#type: "range", min: "{props.min}", max: "{props.max}", step: "{props.step}",
                value: "{lo}", aria_label: "From",
                oninput: move |e| if let Ok(v) = e.value().parse::<f32>() { props.onchange.call((v.min(hi), hi)) } }
            input { class: thumb, r#type: "range", min: "{props.min}", max: "{props.max}", step: "{props.step}",
                value: "{hi}", aria_label: "To",
                oninput: move |e| if let Ok(v) = e.value().parse::<f32>() { props.onchange.call((lo, v.max(lo))) } }
        }
    }
}

/// Text input properties.
#[derive(Props, Clone, PartialEq)]
pub struct TextInputProps {
    pub value: String,
    #[props(default)]
    pub placeholder: String,
    #[props(default)]
    pub disabled: bool,
    /// Hide the characters.
    #[props(default)]
    pub password: bool,
    /// Visible lines; more than one makes it a text area.
    #[props(default = 1)]
    pub lines: u8,
    #[props(default)]
    pub oninput: EventHandler<String>,
}

/// Text input (single line, password or multi-line).
#[component]
pub fn TextInput(props: TextInputProps) -> Element {
    // On the glass the app-wide surface gives input fields.
    let style = crate::provider::glass(
        Some(dodai_core::SurfaceClass::Input),
        &dodai_core::Surface::INHERIT,
    );
    let glassy = if style.is_some() { " dodai-glass" } else { "" };
    if props.lines > 1 {
        let class = format!("dodai-textarea{glassy}");
        return rsx! {
            textarea { class, style, rows: "{props.lines}", value: props.value, placeholder: props.placeholder,
                disabled: props.disabled, oninput: move |e| props.oninput.call(e.value()) }
        };
    }
    let kind = if props.password { "password" } else { "text" };
    let class = format!("dodai-input{glassy}");
    rsx! {
        input { class, style, r#type: kind, value: props.value, placeholder: props.placeholder,
            disabled: props.disabled, oninput: move |e| props.oninput.call(e.value()) }
    }
}

/// Wrapper `<ul class="dodai-list">` for [`ListItem`]s.
#[component]
pub fn List(children: Element) -> Element {
    let style = crate::provider::glass(
        Some(dodai_core::SurfaceClass::Card),
        &dodai_core::Surface::INHERIT,
    );
    let class = if style.is_some() {
        "dodai-list dodai-glass"
    } else {
        "dodai-list"
    };
    rsx! { ul { class, style, role: "listbox", {children} } }
}

/// List item properties.
#[derive(Props, Clone, PartialEq)]
pub struct ListItemProps {
    #[props(default)]
    pub selected: bool,
    #[props(default)]
    pub onclick: EventHandler<MouseEvent>,
    pub children: Element,
}

/// Selectable row.
#[component]
pub fn ListItem(props: ListItemProps) -> Element {
    rsx! {
        li { class: "dodai-list__item", role: "option", "aria-selected": "{props.selected}",
            onclick: move |e| props.onclick.call(e), {props.children} }
    }
}

/// Horizontal rule.
#[component]
pub fn Divider() -> Element {
    rsx! { hr { class: "dodai-divider" } }
}

/// Tooltip properties.
#[derive(Props, Clone, PartialEq)]
pub struct TooltipProps {
    pub tip: String,
    pub children: Element,
}

/// Hover tooltip around its children.
#[component]
pub fn Tooltip(props: TooltipProps) -> Element {
    rsx! { span { class: "dodai-tip", "data-tip": props.tip, tabindex: "0", {props.children} } }
}

/// Heading (`h1`..`h4`, honours `uppercase_labels` via CSS).
#[component]
pub fn Heading(#[props(default = 2)] level: u8, children: Element) -> Element {
    match level {
        1 => rsx! { h1 { {children} } },
        2 => rsx! { h2 { {children} } },
        3 => rsx! { h3 { {children} } },
        _ => rsx! { h4 { {children} } },
    }
}

/// Small label text (case follows the theme).
#[component]
pub fn Label(children: Element) -> Element {
    rsx! { span { class: "dodai-label", {children} } }
}

/// Muted body text.
#[component]
pub fn Muted(children: Element) -> Element {
    rsx! { span { class: "dodai-muted", {children} } }
}

/// Monospace text.
#[component]
pub fn Mono(children: Element) -> Element {
    rsx! { code { class: "dodai-mono", {children} } }
}

/// Icon properties.
#[derive(Props, Clone, PartialEq)]
pub struct IconProps {
    /// The icon, from the shared typed vocabulary (`IconId::Home`).
    pub icon: dodai_icons::IconId,
    #[props(default = 24)]
    pub size: u32,
    /// Use the filled form where the pack has one.
    #[props(default)]
    pub filled: bool,
    /// The theme to draw in; `None` takes the one of the surrounding [`ThemeProvider`](crate::provider::ThemeProvider).
    #[props(default)]
    pub theme: Option<dodai_core::Theme>,
}

/// Inline SVG icon in the current colour, drawn in the active design system's style.
#[component]
pub fn Icon(props: IconProps) -> Element {
    let Some(theme) = props
        .theme
        .clone()
        .or_else(|| try_use_context::<crate::ThemeHandle>().map(|h| h.theme()))
    else {
        return rsx! {};
    };
    match dodai_icons::glyph_for_theme(props.icon, &theme, props.filled) {
        Some(g) => {
            let svg = g.svg(props.size, "currentColor");
            rsx! { span { class: "dodai-icon", style: "display:inline-flex;line-height:0", dangerous_inner_html: "{svg}" } }
        }
        None => rsx! {},
    }
}
