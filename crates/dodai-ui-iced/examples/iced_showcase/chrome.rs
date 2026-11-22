//! The window around the sections: sidebar, control bar, scrolling page.

use dodai_ui_core::Kit;
use dodai_ui_showcase_model::{Action, Ctx, Showcase, CATEGORIES, SPEEDS};
use iced::widget::{column, container, pick_list, row, scrollable, text, text_input, Space};
use iced::{alignment, Element, Length};

use crate::render::{self, Cx};
use crate::{App, Msg};

pub const SIDEBAR_W: f32 = 260.0;
/// Width of the sidebar folded to its icon rail.
pub const RAIL_W: f32 = 72.0;

/// One labelled row of mutually exclusive choices.
fn segmented<'a>(
    app: &App,
    name: &str,
    labels: Vec<String>,
    selected: usize,
    act: impl Fn(usize) -> Action,
) -> Element<'a, Msg> {
    let lt = &app.lt;
    let sp = &lt.theme().spacing;
    row![
        lt.muted_text(name),
        lt.segmented(&labels, selected, move |i| Msg::Act(act(i)))
    ]
    .spacing(sp.sm)
    .align_y(alignment::Vertical::Center)
    .into()
}

/// A dropdown over `items`.
fn dropdown<'a>(
    app: &App,
    name: &str,
    items: Vec<String>,
    selected: usize,
    act: impl Fn(usize) -> Action + Clone + 'a,
) -> Element<'a, Msg> {
    let lt = &app.lt;
    let sp = &lt.theme().spacing;
    let chosen = items.get(selected).cloned();
    let list = items.clone();
    let pick = pick_list(items, chosen, move |s: String| {
        let at = list.iter().position(|x| *x == s).unwrap_or(0);
        Msg::Act(act(at))
    })
    .padding(sp.sm)
    .width(Length::Fixed(260.0))
    .style(lt.pick_list())
    .menu_style(lt.menu());
    row![lt.muted_text(name), lt.field(pick)]
        .spacing(sp.sm)
        .align_y(alignment::Vertical::Center)
        .into()
}

pub fn top_bar<'a>(app: &App) -> Element<'a, Msg> {
    let m = &app.model;
    let sp = &app.lt.theme().spacing;
    let owned = |v: Vec<&str>| v.into_iter().map(String::from).collect::<Vec<_>>();
    let colours = m.colours();
    let colour_at = colours.iter().position(|x| *x == m.colour()).unwrap_or(0);
    let colour_labels: Vec<String> = colours.iter().map(|x| x.label().to_string()).collect();
    let speeds: Vec<String> = SPEEDS.iter().map(|s| format!("{s}×")).collect();
    let on_off = || vec!["Off".to_string(), "On".to_string()];
    row![
        dropdown(
            app,
            "Design",
            owned(m.design_names()),
            m.design_index(),
            Action::Design
        ),
        dropdown(
            app,
            "Theme",
            owned(m.variants()),
            m.variant_index(),
            Action::Variant
        ),
        segmented(app, "Colour", colour_labels, colour_at, move |i| {
            Action::Colour(colours[i])
        }),
        segmented(app, "Animate", on_off(), usize::from(m.animate), |i| {
            Action::Animate(i == 1)
        }),
        segmented(app, "Speed", speeds, m.speed_index(), Action::Speed),
        segmented(
            app,
            "Direction",
            owned(vec!["LTR", "RTL"]),
            usize::from(m.rtl),
            |i| Action::Rtl(i == 1)
        ),
        segmented(app, "Patterns", on_off(), usize::from(m.patterns), |i| {
            Action::Patterns(i == 1)
        }),
    ]
    .spacing(sp.lg)
    .align_y(alignment::Vertical::Center)
    .wrap()
    .vertical_spacing(sp.sm)
    .into()
}

fn nav_item<'a>(
    app: &App,
    label: &str,
    count: usize,
    on: bool,
    cat: Option<usize>,
) -> Element<'a, Msg> {
    let lt = &app.lt;
    let th = lt.theme();
    let ink = th.kit(Kit::Row { selected: on }, false).fg;
    // A category shows whether its sections are listed under it.
    let chevron: Element<'a, Msg> = match cat {
        Some(c) => lt.icon(
            if app.model.lists(c) {
                dodai_ui_icons::IconId::ChevronDown
            } else {
                dodai_ui_icons::IconId::ChevronRight
            },
            14.0,
            Some(th.palette.text_muted),
        ),
        None => Space::new().width(14).into(),
    };
    let line = row![
        lt.icon(
            dodai_ui_showcase_model::catalog::category_icon(label),
            16.0,
            Some(ink)
        ),
        text(lt.label(label)).size(th.typography.size_sm),
        Space::new().width(Length::Fill),
        lt.muted_text(&count.to_string()),
        chevron,
    ]
    .spacing(th.spacing.sm)
    .align_y(alignment::Vertical::Center);
    container(lt.kit_button(
        Kit::Row { selected: on },
        line,
        Msg::Act(Action::Category(cat)),
    ))
    .width(Length::Fill)
    .into()
}

pub fn nav<'a>(app: &App) -> Element<'a, Msg> {
    let m = &app.model;
    let lt = &app.lt;
    let mut col = column![nav_item(
        app,
        "All",
        m.count(None),
        m.current_category().is_none(),
        None
    )];
    for (i, c) in CATEGORIES.iter().enumerate() {
        let on = m.current_category() == Some(i);
        col = col.push(nav_item(app, c, m.count(Some(i)), on, Some(i)));
        if m.lists(i) {
            let active = m.active_section();
            for s in m.listed(i) {
                col = col.push(
                    container(lt.kit_button(
                        Kit::Row {
                            selected: active == Some(s.title),
                        },
                        text(lt.label(s.title)).size(lt.theme().typography.size_xs),
                        Msg::Act(Action::Section(s.title)),
                    ))
                    .padding(iced::Padding::ZERO.left(16.0))
                    .width(Length::Fill),
                );
            }
        }
    }
    scrollable(col)
        .style(lt.scrollable())
        .height(Length::Fill)
        .into()
}

/// The button that folds the sidebar to its rail and back.
fn rail_toggle<'a>(app: &App) -> Element<'a, Msg> {
    let rail = app.model.rail();
    app.lt.button_compact(
        if rail { "»" } else { "«" },
        None,
        dodai_ui_core::ButtonVariant::Ghost,
        dodai_ui_core::ControlSize::Small,
        Some(Msg::Act(Action::Press(Showcase::rail_toggle()))),
    )
}

/// The sidebar folded to a rail: the page icons only.
fn rail<'a>(app: &App) -> Element<'a, Msg> {
    let (lt, m) = (&app.lt, &app.model);
    let th = lt.theme();
    let item = |label: &str, on: bool, cat: Option<usize>| -> Element<'a, Msg> {
        let ink = th.kit(Kit::Row { selected: on }, false).fg;
        container(
            lt.kit_button(
                Kit::Row { selected: on },
                container(lt.icon(
                    dodai_ui_showcase_model::catalog::category_icon(label),
                    20.0,
                    Some(ink),
                ))
                .center_x(Length::Fill),
                Msg::Act(Action::Category(cat)),
            ),
        )
        .width(Length::Fill)
        .into()
    };
    let mut col = column![
        container(rail_toggle(app)).center_x(Length::Fill),
        item("All", m.category().is_none(), None)
    ]
    .spacing(th.spacing.xs);
    for (i, c) in CATEGORIES.iter().enumerate() {
        col = col.push(item(c, m.category() == Some(i), Some(i)));
    }
    col.padding([th.spacing.lg, th.spacing.xs])
        .width(RAIL_W)
        .height(Length::Fill)
        .into()
}

pub fn sidebar<'a>(app: &App) -> Element<'a, Msg> {
    if app.model.rail() {
        return rail(app);
    }
    let lt = &app.lt;
    let sp = &lt.theme().spacing;
    let filter = lt.field(
        text_input("Filter components…", app.model.filter())
            .on_input(|s| Msg::Act(Action::Filter(s)))
            .padding(sp.md)
            .style(lt.text_input()),
    );
    column![
        row![
            crate::render::logo(lt.theme().typography.size_xl * 1.6),
            lt.heading_text("Dodai UI"),
            Space::new().width(Length::Fill),
            rail_toggle(app)
        ]
        .spacing(sp.sm)
        .align_y(alignment::Vertical::Center),
        filter,
        nav(app),
        lt.muted_text(dodai_ui_showcase_model::catalog::KEYS_HINT),
    ]
    .spacing(sp.md)
    .padding(sp.lg)
    .width(SIDEBAR_W)
    .height(Length::Fill)
    .into()
}

pub fn page<'a>(app: &App) -> Element<'a, Msg> {
    let lt = &app.lt;
    let m: &Showcase = &app.model;
    let sp = &lt.theme().spacing;
    let theme = m.theme();
    let ctx = Ctx {
        theme: &theme,
        values: &m.values,
    };
    let cx = Cx {
        lt,
        model: m,
        t: app.t,
        tree_toggled: &app.tree_toggled,
    };
    let title = m.category().map_or("All", |c| CATEGORIES[c]);
    let mut col = column![lt.heading_text(title)].spacing(sp.lg);
    if let Some(o) = m.overview("iced") {
        col = col.push(render::view(&o, &cx));
    }
    let sections = m.visible();
    if sections.is_empty() {
        col = col.push(lt.muted_text("No components match"));
    }
    // The app-wide surface of the Surface effects page paints every panel.
    let fx = m.resolved_surface();
    for s in sections {
        let node = (s.build)(&ctx);
        let body = render::view(&node, &cx);
        let panel = lt.panel_view(Some(s.heading), body, false);
        col = col.push(
            container(iced::widget::stack![panel, lt.surface_fx(fx)])
                .id(iced::widget::Id::from(format!("sec-{}", s.title))),
        );
    }
    scrollable(container(col.padding(sp.xl).width(Length::Fill)).id(app.content_id.clone()))
        .on_scroll(|v| {
            let max = v.content_bounds().height - v.bounds().height;
            let y = v.absolute_offset().y;
            Msg::Scrolled(y, max > 1.0 && y >= max - 1.0)
        })
        .id(app.scroll_id.clone())
        .style(lt.scrollable())
        .height(Length::Fill)
        .into()
}

/// The live-preview strip pinned above the Shape and Surface-effects pages.
fn pinned<'a>(app: &App) -> Option<Element<'a, Msg>> {
    let node = app.model.pinned()?;
    let lt = &app.lt;
    let sp = &lt.theme().spacing;
    let cx = Cx {
        lt,
        model: &app.model,
        t: app.t,
        tree_toggled: &app.tree_toggled,
    };
    Some(
        column![
            container(render::view(&node, &cx))
                .padding([sp.md, sp.lg])
                .width(Length::Fill),
            lt.divider(),
        ]
        .into(),
    )
}

pub fn window<'a>(app: &App) -> Element<'a, Msg> {
    let lt = &app.lt;
    let sp = &lt.theme().spacing;
    let divider_v = container("").width(1).height(Length::Fill).style({
        let c = dodai_ui_iced::color(lt.theme().divider());
        move |_| container::Style {
            background: Some(iced::Background::Color(c)),
            ..container::Style::default()
        }
    });
    row![
        sidebar(app),
        divider_v,
        column![
            container(top_bar(app)).padding(sp.md).width(Length::Fill),
            lt.divider(),
        ]
        .push(pinned(app))
        .push(page(app))
        .width(Length::Fill)
    ]
    .into()
}
