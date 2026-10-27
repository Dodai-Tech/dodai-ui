//! Theme installation: [`ThemeProvider`] + [`ThemeContext`].

use dodai_core::Theme;
use yew::prelude::*;

/// Shared theme state handed to descendants via context.
#[derive(Clone, PartialEq)]
pub struct ThemeContext {
    /// Active theme (glow intensity already applied).
    pub theme: Theme,
    /// Active glow intensity.
    pub glow: f32,
    /// Select a theme by id.
    pub select: Callback<AttrValue>,
    /// Cycle to the next theme.
    pub next: Callback<()>,
    /// Cycle to the previous theme.
    pub prev: Callback<()>,
    /// Set the glow intensity (clamped to ≥ 0).
    pub set_glow: Callback<f32>,
}

/// Full CSS for `theme` (variables + components), as injected by the provider.
pub fn theme_css(theme: &Theme) -> String {
    dodai_topcoat::stylesheet(theme)
}

#[derive(Properties, PartialEq)]
pub struct ThemeProviderProps {
    /// Initial theme id; unknown/empty → first theme.
    #[prop_or_default]
    pub theme_id: AttrValue,
    /// Initial glow intensity (1 = as designed).
    #[prop_or(1.0)]
    pub glow: f32,
    /// Emit the `<style>` element. Disable when the host page supplies CSS.
    #[prop_or(true)]
    pub inject_css: bool,
    #[prop_or_default]
    pub children: Html,
}

/// Applies a theme to its subtree: injects CSS, sets `data-theme`, and
/// provides a [`ThemeContext`].
#[function_component]
pub fn ThemeProvider(props: &ThemeProviderProps) -> Html {
    let id = props.theme_id.clone();
    let switcher = use_state(move || {
        let mut s = dodai_core::switcher();
        s.select(&id);
        s
    });
    let glow = use_state(|| props.glow.max(0.0));
    let theme = switcher.current().clone().with_glow_intensity(*glow);
    let ctx = ThemeContext {
        theme: theme.clone(),
        glow: *glow,
        select: {
            let s = switcher.clone();
            Callback::from(move |id: AttrValue| {
                let mut n = (*s).clone();
                if n.select(&id) {
                    s.set(n);
                }
            })
        },
        next: {
            let s = switcher.clone();
            Callback::from(move |()| {
                let mut n = (*s).clone();
                n.cycle_next();
                s.set(n);
            })
        },
        prev: {
            let s = switcher.clone();
            Callback::from(move |()| {
                let mut n = (*s).clone();
                n.cycle_prev();
                s.set(n);
            })
        },
        set_glow: {
            let g = glow.clone();
            Callback::from(move |v: f32| g.set(v.max(0.0)))
        },
    };
    html! {
        <ContextProvider<ThemeContext> context={ctx}>
            if props.inject_css { <style>{ theme_css(&theme) }</style> }
            <div class="dodai-root" data-theme={theme.id()}>{ props.children.clone() }</div>
        </ContextProvider<ThemeContext>>
    }
}

/// Access the nearest [`ThemeContext`], if any.
#[hook]
pub fn use_theme() -> Option<ThemeContext> {
    use_context::<ThemeContext>()
}

#[derive(Properties, PartialEq)]
pub struct ThemeSwitcherBarProps {
    /// Show `-` / `+` glow buttons.
    #[prop_or(true)]
    pub glow_controls: bool,
}

/// One button per theme plus glow ± controls; needs a [`ThemeProvider`].
#[function_component]
pub fn ThemeSwitcherBar(props: &ThemeSwitcherBarProps) -> Html {
    let Some(ctx) = use_theme() else {
        return html! {};
    };
    let buttons = dodai_core::themes::all().into_iter().map(|t| {
        let active = t.id() == ctx.theme.id();
        let cb = ctx.select.clone();
        let id = AttrValue::from(t.id());
        let onclick = Callback::from(move |_| cb.emit(id.clone()));
        let class = if active {
            "dodai-btn dodai-btn--primary"
        } else {
            "dodai-btn dodai-btn--ghost"
        };
        html! { <button key={t.id()} {class} {onclick}>{ t.name() }</button> }
    });
    let g = ctx.glow;
    let sg = ctx.set_glow.clone();
    let dec = Callback::from({
        let sg = sg.clone();
        move |_| sg.emit(g - 0.5)
    });
    let inc = Callback::from(move |_| sg.emit(g + 0.5));
    html! {
        <div class="dodai-row">
            { for buttons }
            if props.glow_controls {
                <button class="dodai-btn dodai-btn--outline" onclick={dec}>{ "Glow −" }</button>
                <span class="dodai-label">{ format!("{g:.1}") }</span>
                <button class="dodai-btn dodai-btn--outline" onclick={inc}>{ "Glow +" }</button>
            }
        </div>
    }
}
