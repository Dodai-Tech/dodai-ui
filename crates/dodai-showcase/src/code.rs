//! The code toggle under a section.

use super::*;
use dodai_core::Theme;
use dodai_gpui::hsla;
use gpui::SharedString;
use gpui::{div, px, Window};

use catalog::section_id;

impl Showcase {
    /// A section plus, when it has a usage snippet, a code-icon toggle under it (closed by default).
    pub(crate) fn with_code(
        &self,
        title: &'static str,
        key: Option<&'static str>,
        el: gpui::AnyElement,
        t: &Theme,
        me: &gpui::Entity<Showcase>,
    ) -> gpui::AnyElement {
        // What the panel can show: the implementation of each component the section uses, then
        // (when there is one) the usage snippet.
        // Sections that share a key (the vehicle instruments) still list their own components.
        let mut comps = impl_src::components(title);
        if comps.is_empty() {
            comps = key.map(impl_src::components).unwrap_or_default();
        }
        let usage = key.and_then(snippets::snippet);
        let mut tabs: Vec<String> = comps.clone();
        if usage.is_some() {
            tabs.push("Usage".into());
        }
        if tabs.is_empty() {
            return el;
        }
        let pick = self
            .code_pick
            .get(title)
            .copied()
            .unwrap_or(0)
            .min(tabs.len() - 1);
        let code = match comps.get(pick) {
            Some(name) => impl_src::implementation(name).unwrap_or_default(),
            None => usage.unwrap_or_default(),
        };
        let open = self.code_open.contains(title) || std::env::var_os("DODAI_CODE").is_some();
        let me = me.clone();
        let tid = section_id(title);
        let toggle = code_toggle(("code-toggle", tid), t, open, {
            let me = me.clone();
            move |_, _, cx| {
                me.update(cx, |s, cx| {
                    if !s.code_open.remove(title) {
                        s.code_open.insert(title);
                    }
                    cx.notify();
                })
            }
        });
        let max_h = t.typography.size_md * 26.0;
        div()
            .flex()
            .flex_col()
            .gap(px(t.spacing.sm - t.spacing.xs / 2.0))
            .child(el)
            .child(div().flex().justify_end().child(toggle))
            .when(open, |d| {
                d.when(tabs.len() > 1, |d| {
                    let me = me.clone();
                    d.child(
                        div().flex().child(
                            dodai_gpui::Segmented::new(("code-tabs", tid), tabs.clone())
                                .selected(pick)
                                .on_select(move |i, _, cx| {
                                    me.update(cx, |s, cx| {
                                        s.code_pick.insert(title, i);
                                        cx.notify();
                                    })
                                }),
                        ),
                    )
                })
                .child(
                    dodai_gpui::CodeBlock::new(
                        SharedString::from(format!("snippet-{tid}-{pick}")),
                        code,
                    )
                    .language("rust")
                    .max_height(max_h)
                    .scrollbar(true),
                )
            })
            .into_any_element()
    }
}

/// The "Show code" switch every section uses (on = code visible).
pub(crate) fn code_toggle(
    id: impl Into<gpui::ElementId>,
    t: &Theme,
    open: bool,
    on_change: impl Fn(bool, &mut Window, &mut gpui::App) + 'static,
) -> gpui::Div {
    div()
        .flex()
        .items_center()
        .gap(px(8.0))
        .text_size(px(11.0))
        .text_color(hsla(t.palette.text_muted))
        .child(dodai_gpui::Icon::Code.size(14.0))
        .child("Show code")
        .child(dodai_gpui::Toggle::new(id, open).on_change(on_change))
}
