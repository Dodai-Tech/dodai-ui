//! The window around the sections: sidebar, control bar, scrolling page.

use dodai_core::{ButtonVariant, ControlSize, Kit, Surface};
use dodai_showcase_model::{Ctx, Showcase, CATEGORIES, SPEEDS};
use dodai_topcoat::html::{self, esc, Btn, Cx};

use crate::render::{self, R};
use crate::wire;

fn press(a: &str) -> String {
    format!("data-on=\"click\" data-a=\"{}\"", esc(a))
}

fn pick(a: &str) -> String {
    format!("data-on=\"change\" data-a=\"{}\"", esc(a))
}

fn field(name: &str, inner: &str) -> String {
    format!("<div class=\"dodai-topbar__field\"><span class=\"dodai-caption\">{name}</span>{inner}</div>")
}

fn off_on() -> Vec<String> {
    vec!["Off".to_string(), "On".to_string()]
}

fn owned(v: Vec<&str>) -> Vec<String> {
    v.into_iter().map(String::from).collect()
}

fn topbar(m: &Showcase, cx: &Cx) -> String {
    let colours = m.colours();
    let colour_at = colours.iter().position(|x| *x == m.colour()).unwrap_or(0);
    let colour_labels: Vec<String> = colours.iter().map(|x| x.label().to_string()).collect();
    let speeds: Vec<String> = SPEEDS.iter().map(|s| format!("{s}×")).collect();
    let dropdown = |items: Vec<String>, at: usize, kind: &str| {
        html::select(cx, &items, Some(at), "", &pick(&format!("{kind}|{{v}}")))
    };
    let seg = |labels: Vec<String>, at: usize, kind: &'static str| {
        html::segmented(cx, &labels, at, &|i| press(&format!("{kind}|{i}")))
    };
    let flag = |labels: Vec<String>, on: bool, kind: &'static str| {
        html::segmented(cx, &labels, usize::from(on), &|i| {
            press(&format!("{kind}|{}", i == 1))
        })
    };
    let mut o = String::from("<div class=\"dodai-topbar\">");
    o.push_str(&field(
        "Design",
        &dropdown(owned(m.design_names()), m.design_index(), "D"),
    ));
    o.push_str(&field(
        "Theme",
        &dropdown(owned(m.variants()), m.variant_index(), "M"),
    ));
    o.push_str(&field("Colour", &seg(colour_labels, colour_at, "K")));
    o.push_str(&field("Animate", &flag(off_on(), m.animate, "A")));
    o.push_str(&field("Speed", &seg(speeds, m.speed_index(), "E")));
    o.push_str(&field(
        "Direction",
        &flag(owned(vec!["LTR", "RTL"]), m.rtl, "R"),
    ));
    o.push_str(&field("Patterns", &flag(off_on(), m.patterns, "Y")));
    o.push_str("</div>");
    o
}

fn nav_item(
    cx: &Cx,
    label: &str,
    count: usize,
    on: bool,
    page: bool,
    listed: bool,
    action: &str,
) -> String {
    let ic = dodai_showcase_model::catalog::category_icon(label);
    let chevron = if listed {
        dodai_icons::IconId::ChevronDown
    } else {
        dodai_icons::IconId::ChevronRight
    };
    let tail = if page {
        format!(
            "<span class=\"dodai-caption\">{}</span>",
            html::icon(cx, chevron, 14, false)
        )
    } else {
        "<span style=\"width:14px\"></span>".to_string()
    };
    html::kit_box(
        cx,
        Kit::Row { selected: on },
        "width:100%;gap:8px;align-items:center;",
        &Surface::default(),
        true,
        &press(action),
        &format!(
            "{}<span class=\"dodai-label\" style=\"color:inherit;flex:1;text-align:start\">{}</span><span class=\"dodai-caption\">{count}</span>{tail}",
            html::icon(cx, ic, 16, false),
            esc(label)
        ),
    )
}

/// The sidebar.
pub fn sidebar(m: &Showcase, cx: &Cx) -> String {
    let cat = m.category();
    let rail = m.rail();
    let fold = html::button(
        cx,
        &Btn {
            variant: Some(ButtonVariant::Ghost),
            size: Some(ControlSize::Small),
            ..Btn::default()
        },
        &Surface::default(),
        &press(&wire::press(&Showcase::rail_toggle())),
        if rail { "»" } else { "«" },
    );
    if rail {
        let mut o = format!(
            "<aside id=\"side\" class=\"dodai-side\" style=\"width:72px;min-width:72px;padding:var(--dodai-space-xs);align-items:center\">{fold}"
        );
        for (i, name) in std::iter::once("All")
            .chain(CATEGORIES.iter().copied())
            .enumerate()
        {
            let on = if i == 0 {
                cat.is_none()
            } else {
                cat == Some(i - 1)
            };
            let a = i
                .checked_sub(1)
                .map_or("C".to_string(), |c| format!("C|{c}"));
            o.push_str(&html::kit_box(
                cx,
                Kit::Row { selected: on },
                "width:100%;justify-content:center",
                &Surface::default(),
                true,
                &press(&a),
                &html::icon(
                    cx,
                    dodai_showcase_model::catalog::category_icon(name),
                    20,
                    false,
                ),
            ));
        }
        o.push_str("</aside>");
        return o;
    }
    let active = m.active_section();
    let logo = dodai_showcase_model::media::logo_data_url();
    let at = m.current_category();
    let mut nav = nav_item(cx, "All", m.count(None), at.is_none(), false, false, "C");
    for (i, c) in CATEGORIES.iter().enumerate() {
        let listed = m.lists(i);
        nav.push_str(&nav_item(
            cx,
            c,
            m.count(Some(i)),
            at == Some(i),
            true,
            listed,
            &format!("C|{i}"),
        ));
        if listed {
            for t in m.listed(i).iter().map(|s| s.title) {
                nav.push_str(&format!(
                    "<div style=\"padding-left:16px;width:100%\">{}</div>",
                    html::kit_box(
                        cx,
                        Kit::Row {
                            selected: active == Some(t),
                        },
                        "width:100%;",
                        &Surface::default(),
                        true,
                        &format!("{} data-sub=\"{}\"", press(&format!("S|{t}")), esc(t)),
                        &format!(
                            "<span class=\"dodai-label\" style=\"color:inherit;font-size:var(--dodai-size-xs)\">{}</span>",
                            esc(t)
                        ),
                    )
                ));
            }
        }
    }
    format!(
        "<aside id=\"side\" class=\"dodai-side\"><div style=\"display:flex;align-items:center;justify-content:space-between\"><div style=\"display:flex;align-items:center;gap:var(--dodai-space-sm)\"><img src=\"{logo}\" alt=\"\" style=\"width:calc(var(--dodai-size-xl) * 1.6);height:calc(var(--dodai-size-xl) * 1.6);flex:none\"><h2>Dodai UI</h2></div>{fold}</div>{}<nav class=\"dodai-side__nav\">{nav}</nav><span class=\"dodai-caption\">{}</span></aside>",
        html::text_input(
            cx,
            m.filter(),
            "Filter components…",
            false,
            1,
            "data-id=\"filter\" data-on=\"input\" data-a=\"F|{v}\"",
        ),
        esc(dodai_showcase_model::catalog::KEYS_HINT)
    )
}

fn page(m: &Showcase, cx: &Cx) -> String {
    let theme = cx.theme;
    let title = m.category().map_or("All", |c| CATEGORIES[c]);
    let ctx = Ctx {
        theme,
        values: &m.values,
    };
    let r = R {
        cx: *cx,
        vals: &m.values,
    };
    let fx = m.resolved_surface();
    let bg = dodai_topcoat::surface_background(
        theme,
        fx,
        dodai_core::SurfaceClass::Panel,
        (900.0, 420.0),
    )
    .map_or(String::new(), |b| {
        format!("background-image:{b};background-size:100% 100%;border-radius:var(--dodai-radius)")
    });
    let mut o = format!("<main class=\"dodai-page\"><h1>{title}</h1>");
    if let Some(ov) = m.overview("topcoat") {
        o.push_str(&render::view(&ov, &r));
    }
    let shown = m.visible();
    if shown.is_empty() {
        o.push_str("<span class=\"dodai-muted\">No components match</span>");
    }
    for s in shown {
        let node = (s.build)(&ctx);
        let veil = if bg.is_empty() {
            String::new()
        } else {
            format!("<div style=\"position:absolute;inset:0;pointer-events:none;{bg}\"></div>")
        };
        o.push_str(&format!(
            "<div id=\"sec-{t}\" data-sec=\"{t}\" style=\"position:relative\">{}{veil}</div>",
            html::panel(
                cx,
                Some(s.heading),
                false,
                false,
                &Surface::default(),
                "",
                &render::view(&node, &r)
            ),
            t = esc(s.title)
        ));
    }
    o.push_str("</main>");
    o
}

fn pinned(m: &Showcase, cx: &Cx) -> String {
    let Some(node) = m.pinned() else {
        return String::new();
    };
    let r = R {
        cx: *cx,
        vals: &m.values,
    };
    format!(
        "<div style=\"padding:var(--dodai-space-md) var(--dodai-space-lg);border-bottom:1px solid var(--dodai-border);flex:none\">{}</div>",
        render::view(&node, &r)
    )
}

/// The whole app for `m`: the stylesheet of the current theme and the window.
pub fn app(m: &mut Showcase) -> String {
    let theme = m.theme();
    let cx = Cx {
        theme: &theme,
        surface: m.surface(),
    };
    let css = dodai_topcoat::multi_theme_stylesheet(std::slice::from_ref(&theme));
    let backdrop = dodai_topcoat::root_style(&theme);
    let wallpaper = if m.wallpaper() {
        format!(
            "background:url({}) center/cover fixed",
            dodai_showcase_model::media::wallpaper_url(&theme)
        )
    } else {
        backdrop
    };
    let jump = m
        .take_jump()
        .map_or(String::new(), |t| format!(" data-jump=\"{}\"", esc(t)));
    let redraw = m.redraw_every().map_or(String::new(), |d| {
        format!(" data-redraw=\"{}\"", d.as_millis())
    });
    format!(
        "<div id=\"app\" data-theme=\"{id}\"{jump}{redraw}><style>{css}</style><div class=\"dodai-app\" style=\"{wallpaper}\">{side}<div class=\"dodai-main\">{bar}{pin}{pg}</div></div></div>",
        id = theme.id(),
        side = sidebar(m, &cx),
        bar = topbar(m, &cx),
        pin = pinned(m, &cx),
        pg = page(m, &cx),
    )
}

/// Only the sidebar (the highlight follows the page's scroll).
pub fn side(m: &Showcase) -> String {
    let theme = m.theme();
    sidebar(
        m,
        &Cx {
            theme: &theme,
            surface: m.surface(),
        },
    )
}
