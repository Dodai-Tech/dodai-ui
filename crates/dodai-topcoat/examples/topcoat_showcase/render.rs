//! Walks the model's [`Node`] tree and writes HTML with the `dodai-topcoat` components.
//!
//! Interaction is a plain attribute: an element with `data-on="click|change|input"` and
//! `data-a="<action>"` sends that action to the server (`{v}` in it is the element's value).
//! The actions are decoded by `crate::wire`.

use dodai_core::{ButtonVariant, ControlSize, Kit, Surface, ToggleKind};
use dodai_icons::IconId;
use dodai_showcase_model::node::{Axis, Font, Ink, Place, TextRole as Role, TextStyle};
use dodai_showcase_model::{Node, Values};
use dodai_topcoat::html::{self, esc, Btn, Cx, SliderOpts};

use crate::wire;

/// Counter range of the number stepper demo.
pub const STEPPER_RANGE: (i32, i32) = (0, 20);

/// What the builder reads.
#[derive(Clone, Copy)]
pub struct R<'a> {
    /// Theme and surface mix.
    pub cx: Cx<'a>,
    /// The demo values.
    pub vals: &'a Values,
}

/// The text a slider readout shows.
pub fn num_text(v: f32, step: Option<f32>) -> String {
    if step.is_some_and(|s| s.fract() == 0.0) || v.fract() == 0.0 {
        format!("{v:.0}")
    } else {
        format!("{v:.1}")
    }
}

/// The inline style of a text role: size, family, colour and casing resolved by the model, so
/// text matches the gpui showcase.
pub fn text_style(r: Role, theme: &dodai_core::Theme) -> String {
    let st = r.resolve(theme);
    let family = match st.font {
        Font::Display => "var(--dodai-font-display)",
        Font::Body => "var(--dodai-font-body)",
        Font::Mono => "var(--dodai-font-mono)",
    };
    let caps = if st.caps && theme.typography.uppercase_labels {
        "text-transform:uppercase;"
    } else {
        ""
    };
    let lh = match r {
        Role::Type(t) => {
            let s = theme.type_style(t);
            format!("line-height:{}px;font-weight:{};", s.line_height, s.weight)
        }
        _ => String::new(),
    };
    format!(
        "font-size:{}px;font-family:{family};color:{};{caps}{lh}",
        st.size,
        st.ink.color(theme).to_css()
    )
}

fn styled(size: f32, font: Font, ink: Ink) -> Role {
    Role::Styled(TextStyle {
        size,
        font,
        ink,
        caps: false,
    })
}

fn span(style: &str, text: &str) -> String {
    format!("<span style=\"{style}\">{}</span>", esc(text))
}

fn svg_points(points: &[(f32, f32)]) -> String {
    points
        .iter()
        .map(|(x, y)| format!("{x:.1},{y:.1}"))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Pressing the element does `action`.
fn click(action: &str) -> String {
    format!("data-on=\"click\" data-a=\"{}\"", esc(action))
}

/// Changing the element does `action` (its `{v}` is the new value).
fn change(action: &str) -> String {
    format!("data-on=\"change\" data-a=\"{}\"", esc(action))
}

/// Typing in the element does `action` (its `{v}` is the text).
fn typing(action: &str) -> String {
    format!("data-on=\"input\" data-a=\"{}\"", esc(action))
}

fn icon(r: &R, i: IconId, size: u32) -> String {
    html::icon(&r.cx, i, size, false)
}

fn kit(r: &R, k: Kit, style: &str, press: Option<String>, inner: &str) -> String {
    html::kit_box(
        &r.cx,
        k,
        style,
        &Surface::default(),
        press.is_some(),
        &press.unwrap_or_default(),
        inner,
    )
}

/// Build `node`.
pub fn view(node: &Node, r: &R) -> String {
    let theme = r.cx.theme;
    let vals = r.vals;
    match node {
        Node::Stack(s) => {
            let axis = if s.axis == Axis::Row {
                "dodai-stack--row"
            } else {
                "dodai-stack--col"
            };
            let wrap = if s.wrap { "dodai-stack--wrap" } else { "" };
            let gap = s.gap.px(theme);
            let align = if let Some(w) = s.width {
                format!(
                    "width:{w}px;flex:none;align-items:{}",
                    if s.centre && s.axis == Axis::Row {
                        "center"
                    } else {
                        "flex-start"
                    }
                )
            } else if s.axis == Axis::Row && !s.centre {
                "align-items:flex-start".to_string()
            } else if s.axis == Axis::Column && s.centre {
                "align-items:center;width:auto".to_string()
            } else {
                String::new()
            };
            let items: String = s.items.iter().map(|n| view(n, r)).collect();
            format!("<div class=\"dodai-stack {axis} {wrap}\" style=\"gap:{gap}px;{align}\">{items}</div>")
        }
        Node::Text(t, role) => span(&text_style(*role, theme), t),
        Node::Space(px) => format!("<div style=\"width:{px}px;height:{px}px\"></div>"),
        Node::Grow => "<div style=\"flex:1 1 0\"></div>".to_string(),
        Node::Swatch {
            name,
            color,
            detail,
        } => {
            let xs = theme.typography.size_xs;
            format!(
                "<div style=\"display:flex;flex-direction:column;gap:4px;width:118px;flex:none\"><div style=\"height:44px;background:{};border:1px solid {}\"></div>{}{}</div>",
                color.to_css(),
                theme.palette.border.to_css(),
                span(&text_style(styled(xs, Font::Body, Ink::Text), theme), name),
                span(&text_style(styled(xs, Font::Mono, Ink::Muted), theme), detail),
            )
        }
        Node::Sample { w, h, label } => {
            let sm = theme.typography.size_sm;
            html::panel(
                &r.cx,
                None,
                false,
                false,
                &Surface::default(),
                &format!("width:{w}px;height:{h}px;padding:0;display:grid;place-items:center;flex:none"),
                &span(&text_style(styled(sm, Font::Body, Ink::Text), theme), label),
            )
        }
        Node::Logo(size) => format!(
            "<img src=\"{}\" alt=\"Dodai UI\" style=\"width:{size}px;height:{size}px;flex:none\">",
            dodai_showcase_model::media::logo_data_url()
        ),
        Node::Image {
            src: Some(src),
            video,
            contain,
            w,
            h,
            caption,
            ..
        } => {
            let url = dodai_showcase_model::media::data_url(src).unwrap_or_default();
            let fit = if *contain { "contain" } else { "cover" };
            let d = (h * 0.28).clamp(28.0, 56.0);
            let frame = frame_style(theme, *w, *h);
            let badge = format!(
                "<div style=\"position:absolute;width:{d}px;height:{d}px;border-radius:50%;display:grid;place-items:center;background:{};border:1px solid {};color:{};font-size:{}px\">▶</div>",
                theme.palette.background.with_alpha(0.75).to_css(),
                theme.palette.primary.to_css(),
                theme.palette.primary.to_css(),
                d * 0.45
            );
            let cap = caption
                .as_ref()
                .map_or(String::new(), |c| span(&text_style(Role::Muted, theme), c));
            format!(
                "<div style=\"display:flex;flex-direction:column;gap:{}px;width:{w}px;flex:none\"><div style=\"{frame}\"><img src=\"{url}\" style=\"width:100%;height:100%;object-fit:{fit}\">{}</div>{cap}</div>",
                theme.spacing.xs,
                if *video { badge } else { String::new() }
            )
        }
        Node::Image {
            w,
            h,
            caption,
            on_image,
            ..
        } => {
            let (sm, md) = (theme.typography.size_sm, theme.typography.size_md);
            let shadow = theme.palette.roles.shadow;
            let ink = theme.on(shadow.with_alpha(1.0)).to_css();
            let fade_h = (h * 0.5).max(theme.sizes().touch_target);
            let stops = theme
                .fade_stops()
                .iter()
                .rev()
                .map(|s| {
                    format!(
                        "{} {:.1}%",
                        shadow.with_alpha(s.alpha).to_css(),
                        100.0 - s.at * 100.0
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            let over = *on_image && caption.is_some();
            let cap_text = caption.clone().unwrap_or_default();
            let mut inner = span(
                &text_style(styled(md, Font::Body, Ink::Muted), theme),
                "image unavailable",
            );
            if over {
                inner.push_str(&format!(
                    "<div style=\"position:absolute;left:0;right:0;bottom:0;height:{fade_h}px;background:linear-gradient(to bottom,{stops})\"></div><span style=\"position:absolute;left:{}px;right:{}px;bottom:{}px;font-size:{sm}px;color:{ink};font-family:var(--dodai-font-body)\">{}</span>",
                    theme.spacing.md,
                    theme.spacing.md,
                    theme.spacing.sm,
                    esc(&cap_text)
                ));
            }
            let below = if !over {
                caption
                    .as_ref()
                    .map_or(String::new(), |c| span(&text_style(Role::Muted, theme), c))
            } else {
                String::new()
            };
            format!(
                "<div style=\"display:flex;flex-direction:column;gap:{}px;width:{w}px;flex:none\"><div style=\"{}\">{inner}</div>{below}</div>",
                theme.spacing.xs,
                frame_style(theme, *w, *h)
            )
        }
        Node::Block {
            w,
            h,
            fill,
            border,
            radius,
            label,
        } => {
            let size = format!(
                "{}{}",
                if *w > 0.0 {
                    format!("width:{w}px;")
                } else {
                    String::new()
                },
                if *h > 0.0 {
                    format!("height:{h}px;")
                } else {
                    String::new()
                }
            );
            let bd = border.map_or("none".to_string(), |b| format!("1px solid {}", b.to_css()));
            let l = label.as_ref().map_or(String::new(), |l| {
                span(
                    &text_style(styled(theme.typography.size_xs, Font::Body, Ink::Text), theme),
                    l,
                )
            });
            format!(
                "<div style=\"{size}background:{};border:{bd};border-radius:{radius}px;display:grid;place-items:center;flex:none\">{l}</div>",
                fill.to_css()
            )
        }
        Node::Button(b) if b.elevated => {
            let size = theme.control_metrics(ControlSize::Medium).text_size;
            let press = b.counter.as_ref().map(|k| click(&wire::press(k)));
            kit(
                r,
                Kit::Elevated,
                &format!("flex:none;font-size:{size}px"),
                press,
                &esc(&b.label),
            )
        }
        Node::Button(b) => {
            let crop = (!b.plain)
                .then(|| {
                    b.crop.or(dodai_showcase_model::tune_state::default_crop(
                        vals, theme,
                    ))
                })
                .flatten()
                .and_then(|end| theme.button_crop(end));
            let opts = Btn {
                variant: Some(b.variant),
                size: Some(b.size),
                square: b.square,
                icon_only: b.label.is_empty(),
                loading: b.loading,
                disabled: b.disabled,
                shimmer: b.shimmer,
                crop,
            };
            let attrs = b.counter.as_ref().map_or(String::new(), |k| click(&wire::press(k)));
            let mut inner = b.icon.map_or(String::new(), |i| icon(r, i, 16));
            if !b.label.is_empty() {
                inner.push_str(&esc(&b.label));
            }
            html::button(&r.cx, &opts, &b.surface, &attrs, &inner)
        }
        Node::Split {
            label,
            variant,
            open,
            crop,
        } => {
            let arrow = if *open {
                IconId::ChevronUp
            } else {
                IconId::ChevronDown
            };
            let end = crop.and_then(|c| theme.button_crop(c));
            let main = Btn {
                variant: Some(*variant),
                size: Some(ControlSize::Medium),
                crop: end,
                ..Btn::default()
            };
            let side = Btn {
                variant: Some(*variant),
                size: Some(ControlSize::Medium),
                icon_only: true,
                ..Btn::default()
            };
            format!(
                "<span class=\"dodai-split\">{}{}</span>",
                html::button(&r.cx, &main, &Surface::default(), "", &esc(label)),
                html::button(&r.cx, &side, &Surface::default(), "", &icon(r, arrow, 16))
            )
        }
        Node::Fab { label, .. } => {
            let k = Kit::Fab {
                extended: label.is_some(),
            };
            let l = label
                .as_ref()
                .map_or(String::new(), |l| format!("<span>{}</span>", esc(l)));
            kit(
                r,
                k,
                "flex:none;font-size:var(--dodai-size-md)",
                Some(String::new()),
                &format!("{}{l}", icon(r, IconId::Plus, 24)),
            )
        }
        Node::Toolbar {
            floating,
            vibrant,
            icons,
        } => {
            let k = Kit::Toolbar {
                floating: *floating,
                vibrant: *vibrant,
            };
            let width = if *floating { "fit-content" } else { "100%" };
            let btn = Btn {
                variant: Some(ButtonVariant::Ghost),
                icon_only: true,
                ..Btn::default()
            };
            let inner: String = icons
                .iter()
                .map(|ic| html::button(&r.cx, &btn, &Surface::default(), "", &icon(r, *ic, 18)))
                .collect();
            kit(
                r,
                k,
                &format!("width:{width};gap:var(--dodai-space-xs);flex:none"),
                None,
                &inner,
            )
        }
        Node::Choice(ch) if ch.connected => {
            let key = ch.bind.key.clone();
            html::segmented(&r.cx, &ch.labels, vals.index(&ch.bind), &|i| {
                click(&wire::index(&key, i))
            })
        }
        Node::Choice(ch) => {
            let at = vals.index(&ch.bind);
            let mut o = String::from("<div class=\"dodai-seg\">");
            for (i, l) in ch.labels.iter().enumerate() {
                let b = Btn {
                    size: Some(ControlSize::Medium),
                    variant: Some(if i == at {
                        ButtonVariant::Primary
                    } else {
                        ButtonVariant::Outline
                    }),
                    ..Btn::default()
                };
                o.push_str(&html::button(
                    &r.cx,
                    &b,
                    &Surface::default(),
                    &click(&wire::index(&ch.bind.key, i)),
                    &esc(l),
                ));
            }
            o.push_str("</div>");
            o
        }
        Node::Tabs(ch) => html::tabs(
            &r.cx,
            &ch.labels,
            vals.index(&ch.bind),
            ch.secondary,
            &ch.disabled,
            &ch.icons,
            &|i| click(&wire::index(&ch.bind.key, i)),
        ),
        Node::Tag { label, tone, solid } => html::tag(*tone, !*solid, label),
        Node::Chip(ch) => {
            let on = vals.flag(&ch.bind);
            let k = if ch.elevated {
                Kit::Elevated
            } else {
                Kit::Chip { selected: on }
            };
            let lead = if ch.filter && on {
                Some(IconId::Check)
            } else {
                ch.icon
            };
            let mut inner = lead.map_or(String::new(), |i| icon(r, i, 16));
            inner.push_str(&format!("<span>{}</span>", esc(&ch.label)));
            if ch.removable {
                inner.push_str(&icon(r, IconId::Close, 14));
            }
            kit(
                r,
                k,
                &format!("flex:none;font-size:{}px;gap:6px", theme.typography.size_sm),
                Some(click(&wire::flag(&ch.bind.key, !on))),
                &inner,
            )
        }
        Node::Toggle(t) => {
            let key = t.bind.key.clone();
            match t.kind {
                ToggleKind::Switch => html::switch(
                    vals.flag(&t.bind),
                    t.disabled,
                    &t.label,
                    &change(&wire::flag(&key, "{v}")),
                ),
                ToggleKind::Checkbox => html::checkbox(
                    vals.flag(&t.bind),
                    t.disabled,
                    t.mixed,
                    &t.label,
                    &change(&wire::flag(&key, "{v}")),
                ),
                ToggleKind::Radio => {
                    let (group, idx) = t.radio.clone().unwrap_or((key, 0));
                    let picked = vals.index_of(&group, usize::MAX);
                    let on = if picked == usize::MAX {
                        t.bind.default
                    } else {
                        picked == idx
                    };
                    html::radio(
                        &group,
                        on,
                        t.disabled,
                        &t.label,
                        &change(&wire::index(&group, idx)),
                    )
                }
            }
        }
        Node::Knob {
            bind,
            min,
            max,
            size,
            disabled,
        } => {
            let attrs = format!(
                "data-knob=\"{}|{min}|{max}\" data-v=\"{}\"",
                esc(&bind.key),
                vals.num(bind)
            );
            html::knob(theme, vals.num(bind), *min, *max, *size, *disabled, &attrs)
        }
        Node::Select {
            key,
            labels,
            selected,
            placeholder,
        } => format!(
            "<span style=\"min-width:320px;display:inline-block\">{}</span>",
            html::select(
                labels,
                *selected,
                placeholder,
                &format!("style=\"min-width:320px\" {}", change(&wire::index(key, "{v}")))
            )
        ),
        Node::Shader(spec) => crate::shader::view(spec, theme),
        Node::Plot(p) => {
            let lines: String = p
                .lines
                .iter()
                .map(|l| {
                    format!(
                        "<polyline fill=\"none\" stroke=\"{}\" stroke-width=\"{}\" stroke-linejoin=\"round\" points=\"{}\"/>",
                        l.color.to_css(),
                        l.width,
                        svg_points(&l.points)
                    )
                })
                .collect();
            format!(
                "<svg width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\" style=\"flex:none\">{lines}</svg>",
                p.w, p.h, p.w, p.h
            )
        }
        Node::Scroll { height, body } => format!(
            "<div class=\"dodai-scroll\" style=\"height:{height}px;overflow:auto;width:100%;max-width:520px\">{}</div>",
            view(body, r)
        ),
        Node::Slider(s) if s.vertical => {
            let v = vals.num(&s.bind);
            let o = SliderOpts {
                min: s.min,
                max: s.max,
                step: s.step.unwrap_or((s.max - s.min) / 100.0),
                disabled: s.disabled,
                vertical: true,
                length: s.length.unwrap_or(120.0) as u32,
                ..SliderOpts::default()
            };
            html::slider(v, &o, &slide(&s.bind.key, s.min, s.max, s.step))
        }
        Node::Range(rg) => {
            let step = rg.step.unwrap_or((rg.max - rg.min) / 100.0);
            let lo_attrs = format!(
                "data-on=\"input\" data-a=\"{}\"",
                esc(&wire::range(&rg.lo.key, &rg.hi.key, false, "{v}", rg.min, rg.max, rg.step))
            );
            let hi_attrs = format!(
                "data-on=\"input\" data-a=\"{}\"",
                esc(&wire::range(&rg.lo.key, &rg.hi.key, true, "{v}", rg.min, rg.max, rg.step))
            );
            html::range_slider(
                [vals.num(&rg.lo), vals.num(&rg.hi), rg.min, rg.max, step],
                rg.vertical.map(|l| l as u32),
                &lo_attrs,
                &hi_attrs,
            )
        }
        Node::Slider(s) => {
            let v = vals.num(&s.bind);
            let shown = match s.format {
                dodai_showcase_model::node::SliderFormat::Plain => num_text(v, s.step),
                f => f.text(v),
            };
            let o = SliderOpts {
                min: s.min,
                max: s.max,
                step: s.step.unwrap_or((s.max - s.min) / 100.0),
                disabled: s.disabled,
                ticks: s.ticks,
                centered: s.centered,
                bubble: s.bubble.map(|_| shown.clone()),
                bubble_below: s.bubble == Some(false),
                ..SliderOpts::default()
            };
            let lead = s.icon.map_or(String::new(), |ic| icon(r, ic, 20));
            let readout = if s.readout {
                format!("<code class=\"dodai-mono\">{}</code>", esc(&shown))
            } else {
                String::new()
            };
            format!(
                "<div class=\"dodai-stack dodai-stack--row\" style=\"width:100%\">{lead}<div style=\"flex:1\">{}</div>{readout}</div>",
                html::slider(v, &o, &slide(&s.bind.key, s.min, s.max, s.step))
            )
        }
        Node::Progress { value, tone } => {
            format!("<div style=\"width:100%\">{}</div>", html::progress(*value, *tone))
        }
        Node::Input(i) => {
            let label = i.label.as_ref().map_or(String::new(), |l| {
                format!("<span class=\"dodai-label\">{}</span>", esc(l))
            });
            let lead = i.icon.map_or(String::new(), |ic| icon(r, ic, 18));
            format!(
                "<div class=\"dodai-field\" style=\"width:100%\">{label}<div class=\"dodai-stack dodai-stack--row\">{lead}<div style=\"flex:1\">{}</div></div></div>",
                html::text_input(
                    vals.text(&i.bind),
                    &i.placeholder,
                    i.password,
                    i.lines,
                    &format!("data-id=\"{}\" {}", esc(&i.bind.key), typing(&wire::text(&i.bind.key, "{v}"))),
                )
            )
        }
        Node::List(rows) => {
            let mut o = String::from("<div class=\"dodai-list dodai-list--plain\">");
            for (n, row) in rows.iter().enumerate() {
                if n > 0 {
                    o.push_str("<hr class=\"dodai-divider\" style=\"margin:0\">");
                }
                let lead = if let Some(a) = &row.avatar {
                    html::avatar(a, 40)
                } else {
                    row.icon.map_or(String::new(), |i| icon(r, i, 20))
                };
                let mut text = String::new();
                if let Some(ov) = &row.overline {
                    text.push_str(&format!("<span class=\"dodai-caption\">{}</span>", esc(ov)));
                }
                text.push_str(&format!("<span>{}</span>", esc(&row.title)));
                if let Some(d) = &row.detail {
                    text.push_str(&format!("<span class=\"dodai-muted dodai-sm\">{}</span>", esc(d)));
                }
                let trail = row.trailing.as_ref().map_or(String::new(), |t| {
                    format!("<span class=\"dodai-caption\">{}</span>", esc(t))
                });
                o.push_str(&kit(
                    r,
                    Kit::Row {
                        selected: row.selected,
                    },
                    "gap:var(--dodai-space-md)",
                    None,
                    &format!("{lead}<div class=\"dodai-grow dodai-col-text\">{text}</div>{trail}"),
                ));
            }
            o.push_str("</div>");
            o
        }
        Node::Divider => "<hr class=\"dodai-divider\">".to_string(),
        Node::Framed { border, body } => {
            let t = theme;
            let frame = t.framed(t.panel(false).border, *border);
            let vars: String = dodai_topcoat::panel_frame_vars(t, &frame)
                .iter()
                .map(|(k, v)| format!("--dodai-{k}:{v};"))
                .collect();
            format!(
                "<div style=\"{vars}\">{}</div>",
                html::panel(&r.cx, None, false, false, &Surface::default(), "", &view(body, r))
            )
        }
        Node::Card {
            title,
            body,
            variant,
            surface,
        } => {
            let t = if title.is_empty() {
                String::new()
            } else {
                format!("<span class=\"dodai-label\">{}</span>", esc(title))
            };
            html::kit_box(
                &r.cx,
                Kit::Card(*variant),
                "",
                surface,
                false,
                "",
                &format!("{t}{}", view(body, r)),
            )
        }
        Node::Alert {
            tone,
            text,
            banner,
            action,
            dismissible,
        } => {
            let mut inner = String::new();
            if !*banner {
                inner.push_str(&icon(r, html::alert_icon(*tone), 18));
            }
            inner.push_str(&format!("<span style=\"flex:1\">{}</span>", esc(text)));
            if let Some(a) = action {
                inner.push_str(&format!(
                    "<span style=\"text-decoration:underline;cursor:pointer\">{}</span>",
                    esc(a)
                ));
            }
            if *dismissible {
                inner.push_str(&icon(r, IconId::Close, 18));
            }
            kit(
                r,
                Kit::Alert {
                    tone: *tone,
                    banner: *banner,
                },
                "width:100%;",
                None,
                &inner,
            )
        }
        Node::Snackbar { text, action } => kit(
            r,
            Kit::Snackbar,
            "gap:var(--dodai-space-lg);width:fit-content;",
            None,
            &format!(
                "<span>{}</span><button class=\"dodai-btn dodai-btn--ghost\">{}</button>",
                esc(text),
                esc(action)
            ),
        ),
        Node::Popup(inner) => kit(
            r,
            Kit::Popup,
            "flex-direction:column;align-items:stretch;min-width:200px;width:fit-content;",
            None,
            &view(inner, r),
        ),
        Node::Frame {
            h,
            body,
            scrim,
            layer,
            place,
        } => {
            let at = match place {
                Place::Centre => "top:50%;left:50%;transform:translate(-50%,-50%)",
                Place::Start => "top:0;bottom:0;left:0;display:flex",
                Place::End => "top:0;bottom:0;right:0;display:flex",
                Place::Bottom => "left:0;right:0;bottom:0",
            };
            let scrim = scrim.map_or(String::new(), |c| {
                format!("<div style=\"position:absolute;inset:0;background:{}\"></div>", c.to_css())
            });
            format!(
                "<div style=\"position:relative;height:{h}px;width:100%;overflow:hidden;border:1px solid var(--dodai-border);padding:var(--dodai-space-md);box-sizing:border-box\">{}{scrim}<div style=\"position:absolute;{at}\">{}</div></div>",
                view(body, r),
                view(layer, r)
            )
        }
        Node::Glyph {
            icon: ic,
            size,
            filled,
            ink,
        } => {
            let svg = ic
                .glyph(theme, *filled)
                .map(|g| g.svg(*size as u32, &ink.to_css()))
                .unwrap_or_default();
            format!("<span style=\"display:inline-flex;flex:none;width:{size}px;height:{size}px\">{svg}</span>")
        }
        Node::IconTile {
            icon: ic,
            filled,
            pick,
            selected,
        } => {
            let svg = ic
                .glyph(theme, *filled)
                .map(|g| g.svg(28, &theme.palette.primary.to_css()))
                .unwrap_or_default();
            let name = dodai_showcase_model::icons::shortened(
                ic.name(),
                dodai_showcase_model::icons::TILE_NAME_CHARS,
            );
            let bd = if *selected {
                theme.palette.primary
            } else {
                theme.palette.border
            }
            .to_css();
            let press = pick.as_ref().map_or(String::new(), |k| click(&wire::press(k)));
            format!(
                "<div style=\"width:92px;box-sizing:border-box;display:flex;flex-direction:column;align-items:center;gap:4px;padding:{}px 4px;border:1px solid {bd};cursor:pointer\" {press}><span style=\"display:inline-flex;width:28px;height:28px\">{svg}</span><span style=\"font-size:{}px;color:{};white-space:nowrap\">{}</span></div>",
                theme.spacing.sm,
                theme.typography.size_xs,
                theme.palette.text_muted.to_css(),
                esc(&name)
            )
        }
        Node::Frosted { w, h, panels } => {
            let rects: Vec<[f32; 4]> = panels.iter().map(|p| p.rect).collect();
            let scene = dodai_showcase_model::media::frosted(theme, *w as u32, *h as u32, &rects);
            let sharp = dodai_showcase_model::media::picture_url(&scene.sharp);
            let pad = theme.spacing.md;
            let edge = theme.frost().edge.map_or("none".to_string(), |(c, wd)| {
                format!("{wd}px solid {}", c.to_css())
            });
            let mut inner = String::new();
            for (p, pane) in panels.iter().zip(&scene.panes) {
                let [x, y, pw, ph] = p.rect;
                inner.push_str(&format!(
                    "<div style=\"position:absolute;box-sizing:border-box;left:{x}px;top:{y}px;width:{pw}px;height:{ph}px;border-radius:{pad}px;border:{edge};padding:{pad}px;background:url({}) 0 0/100% 100%;display:flex;flex-direction:column\">{}</div>",
                    dodai_showcase_model::media::picture_url(pane),
                    view(&p.body, r)
                ));
            }
            format!(
                "<div style=\"position:relative;flex:none;overflow:hidden;width:{w}px;height:{h}px;background:url({sharp}) 0 0/100% 100%\">{inner}</div>"
            )
        }
        Node::Gizmo { yaw, pitch, size } => {
            let attrs = format!(
                "data-gizmo=\"{}|{}|{size}\" data-yaw=\"{}\" data-pitch=\"{}\"",
                esc(&yaw.key),
                esc(&pitch.key),
                vals.num(yaw),
                vals.num(pitch)
            );
            html::gizmo(theme, *size, vals.num(yaw), vals.num(pitch), &attrs)
        }
        Node::Anchor {
            open,
            trigger,
            panel,
        } => {
            let shown = vals.flag(open);
            let wrap_click = !matches!(**trigger, Node::Button(_));
            let attrs = if wrap_click {
                click(&wire::flag(&open.key, !shown))
            } else {
                String::new()
            };
            let pop = if shown {
                format!(
                    "<div data-stop style=\"position:absolute;top:calc(100% + 4px);left:0;z-index:50\">{}</div>",
                    view(panel, r)
                )
            } else {
                String::new()
            };
            format!(
                "<div style=\"position:relative;display:inline-flex\" {attrs}>{}{pop}</div>",
                view(trigger, r)
            )
        }
        Node::Icon(i, size) => icon(r, *i, *size as u32),
        Node::Rating { value, max } => html::rating(&r.cx, *value, *max),
        Node::Stepper(b) => {
            let v = vals.int(b);
            html::stepper(
                v,
                STEPPER_RANGE.0,
                STEPPER_RANGE.1,
                &click(&wire::int(&b.key, (v - 1).max(STEPPER_RANGE.0))),
                &click(&wire::int(&b.key, (v + 1).min(STEPPER_RANGE.1))),
            )
        }
        Node::Steps { steps, current } => {
            let items: Vec<html::StepItem> = steps
                .iter()
                .map(|s| html::StepItem {
                    title: &s.title,
                    detail: s.detail.as_deref(),
                })
                .collect();
            html::steps(&r.cx, &items, *current)
        }
        Node::Breadcrumb(items) => html::breadcrumb(&r.cx, items),
        Node::Pagination { pages, bind } => html::pagination(&r.cx, *pages, vals.index(bind), &|i| {
            click(&wire::index(&bind.key, i))
        }),
        Node::Tree(nodes) => {
            let items: Vec<html::TreeItem> = nodes.iter().map(tree_item).collect();
            html::tree(&r.cx, &items)
        }
        Node::Table(tb) => table(r, tb),
        Node::Pick { bind, items } => {
            let at = vals.index(bind);
            let rows: String = items
                .iter()
                .enumerate()
                .map(|(i, title)| {
                    kit(
                        r,
                        Kit::Row { selected: i == at },
                        "",
                        Some(click(&wire::index(&bind.key, i))),
                        &format!("<span>{}</span>", esc(title)),
                    )
                })
                .collect();
            format!("<div style=\"display:flex;flex-direction:column;width:100%\">{rows}</div>")
        }
        Node::Rich { spans, role } => rich(spans, *role, theme),
        Node::Quote(body) => format!(
            "<div style=\"padding-left:var(--dodai-space-md);border-left:2px solid {}\">{}</div>",
            theme.palette.primary.to_css(),
            view(body, r)
        ),
        Node::AvatarGroup { names, max, size } => {
            let shown = max.unwrap_or(names.len()).min(names.len());
            let extra = names.len() - shown;
            let overlap = size * 0.25;
            let px = *size as u32;
            let mut o = String::from("<div style=\"display:flex;flex:none\">");
            for (i, n) in names[..shown].iter().enumerate() {
                let m = if i > 0 {
                    format!("margin-left:-{overlap}px")
                } else {
                    String::new()
                };
                o.push_str(&format!("<div style=\"{m}\">{}</div>", html::avatar(n, px)));
            }
            if extra > 0 {
                o.push_str(&format!(
                    "<div style=\"margin-left:-{overlap}px\">{}</div>",
                    html::avatar(&format!("+{extra}"), px)
                ));
            }
            o.push_str("</div>");
            o
        }
        Node::Overlay(items) => {
            let inner: String = items
                .iter()
                .map(|n| format!("<div style=\"grid-area:1/1\">{}</div>", view(n, r)))
                .collect();
            format!("<div style=\"display:grid;flex:none;place-items:center\">{inner}</div>")
        }
        Node::Avatar(s) => html::avatar(s, 40),
        Node::Tooltip {
            text,
            rich,
            title,
            action,
        } => {
            let ty = &theme.typography;
            let mut inner = String::new();
            if let Some(h) = title {
                inner.push_str(&format!("<span style=\"font-size:{}px\">{}</span>", ty.size_md, esc(h)));
            }
            inner.push_str(&format!("<span>{}</span>", esc(text)));
            if let Some(a) = action {
                inner.push_str(&format!(
                    "<span style=\"color:{}\">{}</span>",
                    theme.palette.primary.to_css(),
                    esc(a)
                ));
            }
            kit(
                r,
                Kit::Tooltip { rich: *rich },
                &format!(
                    "flex-direction:column;align-items:flex-start;width:fit-content;font-size:{}px",
                    ty.size_sm
                ),
                None,
                &inner,
            )
        }
        Node::Code { text, lang } => code_block(r, text, lang),
        Node::Timeline(events) => {
            let items: Vec<html::TimelineItem> = events
                .iter()
                .map(|e| html::TimelineItem {
                    title: &e.title,
                    time: &e.time,
                    detail: e.detail.as_deref(),
                    tone: e.tone,
                })
                .collect();
            html::timeline(&r.cx, &items)
        }
        Node::Badged { icon: ic, count, dot } => html::badged(&r.cx, *ic, *count, *dot),
        Node::Spinner(size) => html::spinner(&r.cx, *size as u32),
        Node::Indeterminate => html::indeterminate(),
        Node::Skeleton { w, h } => html::skeleton(*w as u32, *h as u32),
        Node::Empty {
            icon: glyph,
            title,
            detail,
        } => {
            let ty = &theme.typography;
            format!(
                "<div style=\"display:flex;flex-direction:column;align-items:center;gap:var(--dodai-space-sm);padding:var(--dodai-space-lg)\">{}{}{}</div>",
                span(&text_style(styled(ty.size_xxl, Font::Body, Ink::Muted), theme), glyph),
                span(&text_style(styled(ty.size_lg, Font::Body, Ink::Text), theme), title),
                span(&text_style(styled(ty.size_sm, Font::Body, Ink::Muted), theme), detail),
            )
        }
    }
}

/// The attributes of a slider input: its value goes to the model snapped to the step.
fn slide(key: &str, min: f32, max: f32, step: Option<f32>) -> String {
    format!(
        "data-on=\"input\" data-a=\"{}\"",
        esc(&wire::snapped(key, "{v}", min, max, step))
    )
}

fn frame_style(theme: &dodai_core::Theme, w: f32, h: f32) -> String {
    format!(
        "position:relative;overflow:hidden;width:{w}px;height:{h}px;background:{};border:{}px solid {};border-radius:{}px;display:grid;place-items:center",
        theme.palette.surface_raised.to_css(),
        theme.shape.border_width,
        theme.palette.border.to_css(),
        theme.shape.radius,
    )
}

/// The model's tree node as the component's.
pub fn tree_item(n: &dodai_showcase_model::node::TreeNode) -> html::TreeItem {
    html::TreeItem {
        label: n.label.clone(),
        icon: n.icon,
        children: n.children.iter().map(tree_item).collect(),
        open: n.open,
        selected: n.selected,
    }
}

/// A data table: header row, banded rows; fixed-width columns keep their width, the others
/// share the rest; scrolls inside `height` when set.
fn table(r: &R, tb: &dodai_showcase_model::node::Table) -> String {
    use dodai_showcase_model::node::Align;
    let tracks = tb
        .columns
        .iter()
        .map(|c| c.width.map_or("1fr".to_string(), |w| format!("{w}px")))
        .collect::<Vec<_>>()
        .join(" ");
    let align = |a: Align| match a {
        Align::Left => "left",
        Align::Center => "center",
        Align::Right => "right",
    };
    let aligns: Vec<&str> = tb.columns.iter().map(|c| align(c.align)).collect();
    let label = text_style(Role::Label, r.cx.theme);
    let head: String = tb
        .columns
        .iter()
        .enumerate()
        .map(|(i, c)| span(&format!("{label}text-align:{}", aligns[i]), &c.title))
        .collect();
    let scroll = tb.height.map_or(String::new(), |h| {
        format!("max-height:{h}px;overflow:auto;")
    });
    let mut rows = String::new();
    for (i, row) in tb.rows.iter().enumerate() {
        let cells: String = row
            .iter()
            .enumerate()
            .map(|(j, c)| {
                span(
                    &format!("text-align:{}", aligns.get(j).copied().unwrap_or("left")),
                    c,
                )
            })
            .collect();
        rows.push_str(&kit(
            r,
            Kit::TableRow(i % 2 == 1),
            &format!("display:grid;grid-template-columns:{tracks};"),
            None,
            &cells,
        ));
    }
    format!(
        "<div class=\"dodai-datatable\" role=\"table\" style=\"width:100%\">{}<div style=\"{scroll}\">{rows}</div></div>",
        kit(
            r,
            Kit::TableHead,
            &format!("display:grid;grid-template-columns:{tracks};"),
            None,
            &head
        )
    )
}

/// A code listing: the language over syntax-coloured lines.
fn code_block(r: &R, code: &str, lang: &str) -> String {
    use dodai_core::syntax::{highlight_lines, CodeToken as K};
    let theme = r.cx.theme;
    let p = &theme.palette;
    let colour = |k: K| match k {
        K::Keyword => p.primary,
        K::Str => p.success,
        K::Comment => p.text_muted,
        K::Number => p.warning,
        K::Type => p.info,
    };
    let mut lines = String::new();
    for (l, spans) in code.split('\n').zip(highlight_lines(code, lang)) {
        let mut out = String::new();
        let mut at = 0;
        for (rg, k) in spans {
            if rg.start > at {
                out.push_str(&esc(&l[at..rg.start]));
            }
            out.push_str(&format!(
                "<span style=\"color:{}\">{}</span>",
                colour(k).to_css(),
                esc(&l[rg.clone()])
            ));
            at = rg.end;
        }
        out.push_str(&esc(&l[at..]));
        lines.push_str(&format!("<div style=\"min-height:1.3em\">{out}</div>"));
    }
    let cap = if lang.is_empty() {
        String::new()
    } else {
        format!(
            "<div style=\"{}\">{}</div>",
            text_style(Role::Caption, theme),
            esc(&lang.to_uppercase())
        )
    };
    kit(
        r,
        Kit::Code,
        "display:block;width:100%",
        None,
        &format!("{cap}<pre class=\"dodai-code\" style=\"margin:0\">{lines}</pre>"),
    )
}

/// Inline-styled runs as one paragraph.
fn rich(
    spans: &[dodai_showcase_model::node::Span],
    role: Role,
    theme: &dodai_core::Theme,
) -> String {
    let p = &theme.palette;
    let (code_fg, code_bg, link) = (
        p.primary.to_css(),
        p.surface_raised.to_css(),
        p.link.to_css(),
    );
    let inner: String = spans
        .iter()
        .map(|s| {
            let st = format!(
                "{}{}{}{}",
                if s.bold { "font-weight:700;" } else { "" },
                if s.italic { "font-style:italic;" } else { "" },
                if s.code {
                    format!(
                        "font-family:var(--dodai-font-mono);color:{code_fg};background:{code_bg};"
                    )
                } else {
                    String::new()
                },
                if s.link {
                    format!("color:{link};text-decoration:underline;")
                } else {
                    String::new()
                },
            );
            span(&st, &s.text)
        })
        .collect();
    format!("<span style=\"{}\">{inner}</span>", text_style(role, theme))
}

#[cfg(test)]
mod tests {
    use super::*;
    use dodai_showcase_model::node::TreeNode;

    #[test]
    fn readouts_drop_the_fraction_of_whole_numbers() {
        assert_eq!(num_text(3.0, None), "3");
        assert_eq!(num_text(0.25, Some(0.05)), "0.2");
    }

    #[test]
    fn tree_items_keep_labels_nesting_and_open_state() {
        let n = TreeNode {
            label: "a".into(),
            icon: None,
            children: vec![TreeNode {
                label: "b".into(),
                icon: None,
                children: vec![],
                open: true,
                selected: false,
            }],
            open: false,
            selected: true,
        };
        let t = tree_item(&n);
        assert_eq!((t.label.as_str(), t.open, t.selected), ("a", false, true));
        assert_eq!(t.children[0].label, "b");
    }
}
