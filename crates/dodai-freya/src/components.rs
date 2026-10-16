//! Themed Freya components. All read the active theme via
//! [`use_dodai_theme`](crate::use_dodai_theme); shape, glow, hard shadow
//! and label casing come from tokens only.
//!
//! Boxes with a chamfered theme need explicit pixel `width`/`height` (the SVG
//! outline is generated for that size); components estimate a width from the
//! label when none is given.

use crate::style::*;
use crate::theme::{use_dodai, use_dodai_theme};
use dodai_core::{ButtonVariant, Color, ControlState, ShapeKind, Theme, Tone};
use freya::prelude::*;

fn c(col: Color) -> String {
    css(col)
}

/// Paint a shaped box of `w`×`h` containing `children` (centred).
pub fn shaped(
    theme: &Theme,
    style: &ShapeStyle,
    w: f32,
    h: f32,
    padding: f32,
    vertical: bool,
    children: Element,
) -> Element {
    let chamfer = style.is_chamfered();
    let svg = chamfer.then(|| chamfer_svg(theme, style, w, h));
    let pad = style.svg_pad();
    let (bg, border, radius, shadow) = if chamfer {
        (
            "transparent".to_string(),
            "none".to_string(),
            "0".to_string(),
            "none".to_string(),
        )
    } else {
        (
            style.background(),
            style.border(),
            style.corner_radius(),
            style.shadow(),
        )
    };
    let dir = if vertical { "vertical" } else { "horizontal" };
    let (main, cross) = if vertical {
        ("start", "start")
    } else {
        ("center", "center")
    };
    rsx! {
        rect {
            width: "{w}",
            height: "{h}",
            padding: "{padding}",
            direction: "{dir}",
            main_align: "{main}",
            cross_align: "{cross}",
            background: "{bg}",
            border: "{border}",
            corner_radius: "{radius}",
            shadow: "{shadow}",
            if let Some(svg) = svg {
                svg {
                    position: "absolute",
                    layer: "1",
                    position_top: "-{pad}",
                    position_left: "-{pad}",
                    width: "{w + 2.0 * pad}",
                    height: "{h + 2.0 * pad}",
                    svg_content: "{svg}",
                }
            }
            {children}
        }
    }
}

fn text(ts: &TextStyle, color: Option<Color>, s: &str) -> Element {
    let col = c(color.unwrap_or(ts.color));
    let t = ts.apply(s);
    rsx! {
        label {
            color: "{col}",
            font_size: "{ts.size}",
            font_family: "{ts.family}",
            font_weight: "{ts.weight}",
            letter_spacing: "{ts.letter_spacing}",
            "{t}"
        }
    }
}

// ── text ────────────────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
pub struct TextProps {
    #[props(into)]
    pub text: String,
    #[props(default)]
    pub size: Option<f32>,
}

/// Display heading (uppercased when the theme says so).
#[component]
pub fn DodaiHeading(props: TextProps) -> Element {
    let t = use_dodai_theme();
    text(
        &text_style(&t, TextRole::Heading, props.size),
        None,
        &props.text,
    )
}
/// Body label (uppercased when the theme says so).
#[component]
pub fn DodaiLabel(props: TextProps) -> Element {
    let t = use_dodai_theme();
    text(
        &text_style(&t, TextRole::Label, props.size),
        None,
        &props.text,
    )
}
/// Muted secondary text.
#[component]
pub fn DodaiMuted(props: TextProps) -> Element {
    let t = use_dodai_theme();
    text(
        &text_style(&t, TextRole::Muted, props.size),
        None,
        &props.text,
    )
}
/// Monospace text (never uppercased).
#[component]
pub fn DodaiMono(props: TextProps) -> Element {
    let t = use_dodai_theme();
    text(
        &text_style(&t, TextRole::Mono, props.size),
        None,
        &props.text,
    )
}

// ── interaction helper ──────────────────────────────────────────────────────

fn state_of(hover: bool, down: bool, disabled: bool) -> ControlState {
    if disabled {
        ControlState::Disabled
    } else if down {
        ControlState::Pressed
    } else if hover {
        ControlState::Hovered
    } else {
        ControlState::Idle
    }
}

// ── button ──────────────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
pub struct ButtonProps {
    #[props(into)]
    pub label: String,
    #[props(default = ButtonVariant::Primary)]
    pub variant: ButtonVariant,
    #[props(default)]
    pub disabled: bool,
    /// Fixed width in px; estimated from the label when `None`.
    #[props(default)]
    pub width: Option<f32>,
    #[props(default)]
    pub onclick: EventHandler<()>,
}

/// Themed button, every [`ButtonVariant`] plus disabled.
#[component]
pub fn DodaiButton(props: ButtonProps) -> Element {
    let t = use_dodai_theme();
    let mut hover = use_signal(|| false);
    let mut down = use_signal(|| false);
    let state = state_of(*hover.read(), *down.read(), props.disabled);
    let (style, fg) = button_style(&t, props.variant, state);
    let ts = text_style(&t, TextRole::Label, None);
    let w = props.width.unwrap_or_else(|| {
        estimate_text_width(&ts.apply(&props.label), ts.size, ts.letter_spacing)
            + 2.0 * t.spacing.lg
    });
    let h = t.shape.control_height;
    let disabled = props.disabled;
    let body = text(&ts, Some(fg), &props.label);
    let boxed = shaped(&t, &style, w, h, 0.0, false, body);
    rsx! {
        rect {
            onpointerenter: move |_| hover.set(true),
            onpointerleave: move |_| {
                hover.set(false);
                down.set(false);
            },
            onmousedown: move |_| down.set(true),
            onmouseup: move |_| down.set(false),
            onclick: move |_| {
                if !disabled {
                    props.onclick.call(());
                }
            },
            {boxed}
        }
    }
}

// ── panel ───────────────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
pub struct PanelProps {
    #[props(default)]
    pub title: Option<String>,
    #[props(default = 320.0)]
    pub width: f32,
    #[props(default = 160.0)]
    pub height: f32,
    /// Glows when focused (if the theme has glow).
    #[props(default)]
    pub focused: bool,
    pub children: Element,
}

/// Bordered container with optional title.
#[component]
pub fn DodaiPanel(props: PanelProps) -> Element {
    let t = use_dodai_theme();
    let style = panel_style(&t, props.focused);
    let title = props.title.as_ref().map(|s| {
        let mut ts = text_style(&t, TextRole::Label, Some(t.typography.size_lg));
        ts.color = t.panel(props.focused).title;
        ts.weight = "bold";
        text(&ts, None, s)
    });
    let (spacing, pad) = (t.spacing.sm, t.spacing.md);
    let inner = rsx! {
        rect { direction: "vertical", spacing: "{spacing}", margin: "{pad}", {title} {props.children} }
    };
    shaped(&t, &style, props.width, props.height, 0.0, true, inner)
}

// ── tag ─────────────────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
pub struct TagProps {
    #[props(into)]
    pub label: String,
    #[props(default = Tone::Info)]
    pub tone: Tone,
    #[props(default = true)]
    pub solid: bool,
}

/// Status tag.
#[component]
pub fn DodaiTag(props: TagProps) -> Element {
    let t = use_dodai_theme();
    let (style, fg) = tag_style(&t, props.tone, props.solid);
    let ts = text_style(&t, TextRole::Label, Some(t.typography.size_xs));
    let w = estimate_text_width(&ts.apply(&props.label), ts.size, ts.letter_spacing)
        + 2.0 * t.spacing.sm;
    let h = t.typography.size_xs + 2.0 * t.spacing.xs + 4.0;
    shaped(
        &t,
        &style,
        w,
        h,
        0.0,
        false,
        text(&ts, Some(fg), &props.label),
    )
}

// ── tabs ────────────────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
pub struct TabProps {
    #[props(into)]
    label: String,
    active: bool,
    onselect: EventHandler<()>,
}

#[component]
fn DodaiTab(props: TabProps) -> Element {
    let t = use_dodai_theme();
    let mut hover = use_signal(|| false);
    let (style, fg, ind) = tab_style(&t, props.active, *hover.read());
    let ts = text_style(&t, TextRole::Label, Some(t.typography.size_sm));
    let w = estimate_text_width(&ts.apply(&props.label), ts.size, ts.letter_spacing)
        + 2.0 * t.spacing.md;
    let h = t.shape.control_height;
    let line = ind.map(|i| {
        let col = c(i);
        rsx! { rect { position: "absolute", position_bottom: "0", width: "{w}", height: "2", background: "{col}" } }
    });
    let body = rsx! { {text(&ts, Some(fg), &props.label)} {line} };
    let boxed = shaped(&t, &style, w, h, 0.0, false, body);
    rsx! {
        rect {
            onpointerenter: move |_| hover.set(true),
            onpointerleave: move |_| hover.set(false),
            onclick: move |_| props.onselect.call(()),
            {boxed}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct TabsProps {
    pub labels: Vec<String>,
    #[props(default)]
    pub selected: usize,
    #[props(default)]
    pub onselect: EventHandler<usize>,
}

/// Tab strip.
#[component]
pub fn DodaiTabs(props: TabsProps) -> Element {
    let t = use_dodai_theme();
    let gap = t.spacing.xs;
    rsx! {
        rect {
            direction: "horizontal",
            spacing: "{gap}",
            for (i, l) in props.labels.iter().enumerate() {
                DodaiTab {
                    key: "{i}",
                    label: l.clone(),
                    active: i == props.selected,
                    onselect: move |_| props.onselect.call(i),
                }
            }
        }
    }
}

// ── toggle / checkbox / radio ───────────────────────────────────────────────

fn thumb(theme: &Theme, color: Color, size: f32, round: bool) -> Element {
    let col = c(color);
    let r = if round && theme.shape.kind() == ShapeKind::Rounded {
        format!("{}", (theme.shape.radius).min(size / 2.0))
    } else {
        "0".into()
    };
    rsx! { rect { width: "{size}", height: "{size}", background: "{col}", corner_radius: "{r}" } }
}

#[derive(Props, Clone, PartialEq)]
pub struct ToggleProps {
    pub on: bool,
    #[props(default)]
    pub disabled: bool,
    #[props(default)]
    pub onchange: EventHandler<bool>,
}

/// On/off switch.
#[component]
pub fn DodaiToggle(props: ToggleProps) -> Element {
    let t = use_dodai_theme();
    let mut hover = use_signal(|| false);
    let state = state_of(*hover.read(), false, props.disabled);
    let (style, thumb_c) = toggle_style(&t, props.on, state);
    let (w, h, pad) = (44.0, 24.0, 4.0);
    let align = if props.on { "end" } else { "start" };
    let inner = rsx! {
        rect { width: "{w - 2.0 * pad}", height: "{h - 2.0 * pad}", direction: "horizontal", main_align: "{align}", cross_align: "center",
            {thumb(&t, thumb_c, h - 2.0 * pad, true)} }
    };
    let boxed = shaped(&t, &style, w, h, pad, false, inner);
    let (on, disabled) = (props.on, props.disabled);
    rsx! {
        rect {
            onpointerenter: move |_| hover.set(true),
            onpointerleave: move |_| hover.set(false),
            onclick: move |_| if !disabled { props.onchange.call(!on) },
            {boxed}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct CheckProps {
    pub checked: bool,
    #[props(default, into)]
    pub label: Option<String>,
    #[props(default)]
    pub disabled: bool,
    #[props(default)]
    pub onchange: EventHandler<bool>,
}

fn mark_box(t: &Theme, on: bool, state: ControlState, round: bool) -> Element {
    let (mut style, mark) = toggle_style(t, on, state);
    if round && style.kind == ShapeKind::Rounded {
        style.radius = 9.0;
    }
    let inner = if on {
        thumb(t, mark, 8.0, round)
    } else {
        rsx! {}
    };
    shaped(t, &style, 18.0, 18.0, 0.0, false, inner)
}

/// Checkbox with optional label.
#[component]
pub fn DodaiCheckbox(props: CheckProps) -> Element {
    let t = use_dodai_theme();
    let mut hover = use_signal(|| false);
    let state = state_of(*hover.read(), false, props.disabled);
    let ts = text_style(&t, TextRole::Label, Some(t.typography.size_sm));
    let col = if props.disabled {
        Some(t.palette.text_disabled)
    } else {
        None
    };
    let lbl = props.label.as_ref().map(|l| text(&ts, col, l));
    let (on, disabled, gap) = (props.checked, props.disabled, t.spacing.sm);
    rsx! {
        rect {
            direction: "horizontal", cross_align: "center", spacing: "{gap}",
            onpointerenter: move |_| hover.set(true),
            onpointerleave: move |_| hover.set(false),
            onclick: move |_| if !disabled { props.onchange.call(!on) },
            {mark_box(&t, on, state, false)}
            {lbl}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct RadioProps {
    pub selected: bool,
    #[props(default, into)]
    pub label: Option<String>,
    #[props(default)]
    pub disabled: bool,
    #[props(default)]
    pub onselect: EventHandler<()>,
}

/// Radio button (round only when the theme's shape is `Rounded`).
#[component]
pub fn DodaiRadio(props: RadioProps) -> Element {
    let t = use_dodai_theme();
    let mut hover = use_signal(|| false);
    let state = state_of(*hover.read(), false, props.disabled);
    let ts = text_style(&t, TextRole::Label, Some(t.typography.size_sm));
    let col = if props.disabled {
        Some(t.palette.text_disabled)
    } else {
        None
    };
    let lbl = props.label.as_ref().map(|l| text(&ts, col, l));
    let (disabled, gap) = (props.disabled, t.spacing.sm);
    rsx! {
        rect {
            direction: "horizontal", cross_align: "center", spacing: "{gap}",
            onpointerenter: move |_| hover.set(true),
            onpointerleave: move |_| hover.set(false),
            onclick: move |_| if !disabled { props.onselect.call(()) },
            {mark_box(&t, props.selected, state, true)}
            {lbl}
        }
    }
}

// ── progress / slider ───────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
pub struct ProgressProps {
    /// 0..=100
    pub value: f32,
    #[props(default)]
    pub tone: Option<Tone>,
    #[props(default = 240.0)]
    pub width: f32,
    #[props(default = 12.0)]
    pub height: f32,
}

/// Progress bar.
#[component]
pub fn DodaiProgress(props: ProgressProps) -> Element {
    let t = use_dodai_theme();
    let (style, fill) = progress_style(&t, props.tone);
    let bw = t.shape.border_width;
    let inner_w = (props.width - 2.0 * bw) * props.value.clamp(0.0, 100.0) / 100.0;
    let inner_h = props.height - 2.0 * bw;
    let col = c(fill);
    let r = style.corner_radius();
    let bar = rsx! { rect { width: "{inner_w}", height: "{inner_h}", background: "{col}", corner_radius: "{r}" } };
    shaped(&t, &style, props.width, props.height, bw, false, bar)
}

#[derive(Props, Clone, PartialEq)]
pub struct SliderProps {
    /// 0..=100
    pub value: f64,
    #[props(default)]
    pub onmoved: EventHandler<f64>,
    #[props(default = "220".to_string(), into)]
    pub width: String,
}

/// Slider (wraps Freya's `Slider`, themed from tokens).
#[component]
pub fn DodaiSlider(props: SliderProps) -> Element {
    let t = use_dodai_theme();
    let p = &t.palette;
    let theme = theme_with!(SliderTheme {
        background: c(p.surface_raised).into(),
        thumb_background: c(p.primary).into(),
        thumb_inner_background: c(p.primary).into(),
        border_fill: c(p.border).into(),
    });
    rsx! {
        rect { width: "{props.width}",
            Slider { theme, size: "100%", value: props.value, onmoved: move |v| props.onmoved.call(v) }
        }
    }
}

// ── text input ──────────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
pub struct InputProps {
    pub value: String,
    #[props(default, into)]
    pub placeholder: Option<String>,
    #[props(default)]
    pub onchange: EventHandler<String>,
    #[props(default = "220".to_string(), into)]
    pub width: String,
}

/// Text input (wraps Freya's `Input`; chamfered themes render it square since
/// Freya's `Input` cannot be clipped to a polygon).
#[component]
pub fn DodaiTextInput(props: InputProps) -> Element {
    let t = use_dodai_theme();
    let idle = input_style(&t, false);
    let tok = t.input(false);
    let focus = t.input(true);
    let radius = idle.corner_radius();
    let theme = theme_with!(InputTheme {
        background: c(tok.background).into(),
        hover_background: c(tok.background).into(),
        border_fill: c(tok.border).into(),
        focus_border_fill: c(focus.border).into(),
        corner_radius: radius.into(),
        shadow: "none".into(),
        font_theme: theme_with!(FontTheme {
            color: c(tok.text).into()
        }),
        placeholder_font_theme: theme_with!(FontTheme {
            color: c(tok.placeholder).into()
        }),
    });
    rsx! {
        Input {
            theme,
            value: props.value.clone(),
            placeholder: props.placeholder.clone(),
            width: props.width.clone(),
            onchange: move |v| props.onchange.call(v),
        }
    }
}

// ── list / divider / tooltip ────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
pub struct ListItemProps {
    #[props(into)]
    pub label: String,
    #[props(default)]
    pub selected: bool,
    #[props(default = 260.0)]
    pub width: f32,
    #[props(default)]
    pub onclick: EventHandler<()>,
}

/// Selectable list row with marker.
#[component]
pub fn DodaiListItem(props: ListItemProps) -> Element {
    let t = use_dodai_theme();
    let mut hover = use_signal(|| false);
    let (bg, fg, marker) = list_item_style(&t, props.selected, *hover.read());
    let bg = bg.map_or("transparent".to_string(), c);
    let ts = text_style(&t, TextRole::Label, Some(t.typography.size_sm));
    let m = marker.map_or("transparent".to_string(), c);
    let h = t.shape.control_height;
    rsx! {
        rect {
            width: "{props.width}", height: "{h}", direction: "horizontal", cross_align: "center", background: "{bg}",
            onpointerenter: move |_| hover.set(true),
            onpointerleave: move |_| hover.set(false),
            onclick: move |_| props.onclick.call(()),
            rect { width: "3", height: "100%", background: "{m}" }
            rect { padding: "0 {t.spacing.md}", {text(&ts, Some(fg), &props.label)} }
        }
    }
}

/// Horizontal rule.
#[component]
pub fn DodaiDivider() -> Element {
    let t = use_dodai_theme();
    let col = c(t.divider());
    rsx! { rect { width: "100%", height: "{t.shape.border_width}", background: "{col}" } }
}

/// Tooltip bubble (static). Use [`DodaiTooltipHost`] for hover behaviour.
#[component]
pub fn DodaiTooltip(#[props(into)] text: String) -> Element {
    let t = use_dodai_theme();
    let style = ShapeStyle::from_tokens(
        &t,
        Some(t.palette.overlay),
        Some(t.palette.border_strong),
        false,
        false,
    );
    let ts = text_style(&t, TextRole::Muted, Some(t.typography.size_xs));
    let w = estimate_text_width(&text, ts.size, 0.0) + 2.0 * t.spacing.sm;
    let h = ts.size + 2.0 * t.spacing.sm;
    let body = self::text(&ts, Some(t.palette.text), &text);
    shaped(&t, &style, w, h, 0.0, false, body)
}

#[derive(Props, Clone, PartialEq)]
pub struct TooltipHostProps {
    #[props(into)]
    pub tip: String,
    /// Distance from the top of the host to the tooltip.
    #[props(default = 44.0)]
    pub offset: f32,
    pub children: Element,
}

/// Shows [`DodaiTooltip`] under its children while hovered.
#[component]
pub fn DodaiTooltipHost(props: TooltipHostProps) -> Element {
    let mut hover = use_signal(|| false);
    rsx! {
        rect {
            onpointerenter: move |_| hover.set(true),
            onpointerleave: move |_| hover.set(false),
            {props.children}
            if *hover.read() {
                rect { position: "absolute", position_top: "{props.offset}", position_left: "0", layer: "-50",
                    DodaiTooltip { text: props.tip.clone() } }
            }
        }
    }
}

// ── theme switcher bar ──────────────────────────────────────────────────────

/// One button per registered theme; the active one is `Primary`, others `Outline`.
#[component]
pub fn DodaiThemeSwitcher() -> Element {
    let dodai = use_dodai();
    let t = dodai.theme();
    let cur = dodai.current_index();
    let gap = t.spacing.sm;
    rsx! {
        rect { direction: "horizontal", spacing: "{gap}",
            for (i, (id, name)) in dodai.list().into_iter().enumerate() {
                DodaiButton {
                    key: "{id}",
                    label: name,
                    variant: if i == cur { ButtonVariant::Primary } else { ButtonVariant::Outline },
                    onclick: move |_| dodai.select_index(i),
                }
            }
        }
    }
}
