//! A design spec that overrides nothing resolves every component exactly like a theme with no
//! spec: the default hooks return `None` and the theme falls back to its own tokens.

use dodai_ui_core::{
    themes, ButtonVariant, ControlSize, ControlState, DesignSpec, ItemState, SpecRef, Theme,
};

#[derive(Debug)]
struct Plain;

impl DesignSpec for Plain {
    fn name(&self) -> &'static str {
        "plain"
    }
}

static PLAIN: Plain = Plain;

fn pair() -> (Theme, Theme) {
    let base = themes::neutral();
    let mut with = base.clone();
    with.spec = SpecRef::new(&PLAIN);
    (base, with)
}

#[test]
fn an_empty_spec_changes_no_component_token() {
    let (a, b) = pair();
    assert_eq!(b.spec.name(), "plain");
    assert_ne!(a.spec, b.spec);
    for s in [
        ControlState::Idle,
        ControlState::Hovered,
        ControlState::Pressed,
        ControlState::Focused,
        ControlState::Disabled,
    ] {
        for v in [
            ButtonVariant::Primary,
            ButtonVariant::Secondary,
            ButtonVariant::Ghost,
            ButtonVariant::Danger,
        ] {
            assert_eq!(a.button(v, s), b.button(v, s), "{v:?} {s:?}");
        }
    }
    for st in [
        ItemState::IDLE,
        ItemState::HOVERED,
        ItemState::SELECTED,
        ItemState::SELECTED_HOVERED,
    ] {
        assert_eq!(a.chip(st), b.chip(st), "chip {st:?}");
        assert_eq!(a.tab(st), b.tab(st), "tab {st:?}");
        assert_eq!(a.list_item(st), b.list_item(st), "list {st:?}");
        assert_eq!(a.navigation_item(st), b.navigation_item(st), "nav {st:?}");
    }
    for size in [ControlSize::Small, ControlSize::Medium, ControlSize::Large] {
        assert_eq!(a.control_metrics(size), b.control_metrics(size), "{size:?}");
    }
    assert_eq!(a.button_padding(), b.button_padding());
    assert_eq!(a.frost(), b.frost());
}

#[test]
fn item_state_constants_spell_out_the_flags() {
    assert_eq!(ItemState::IDLE, ItemState::default());
    assert_eq!(ItemState::SELECTED, ItemState::of(true, false));
    assert_eq!(ItemState::HOVERED, ItemState::of(false, true));
    assert_eq!(ItemState::SELECTED_HOVERED, ItemState::of(true, true));
}

#[test]
fn an_empty_spec_changes_no_other_accessor() {
    use dodai_ui_core::{AppBarKind, CardVariant, StepState, ToggleKind, Tone, TypeRole};
    let (a, b) = pair();
    let tones = [Tone::Info, Tone::Success, Tone::Warning, Tone::Danger];
    for tone in tones {
        assert_eq!(a.badge(tone), b.badge(tone), "badge {tone:?}");
        for flag in [false, true] {
            assert_eq!(a.alert(tone, flag), b.alert(tone, flag), "alert {tone:?}");
            assert_eq!(a.tag(tone, flag), b.tag(tone, flag), "tag {tone:?}");
        }
        assert_eq!(a.progress(Some(tone)), b.progress(Some(tone)));
    }
    assert_eq!(a.progress(None), b.progress(None));
    for v in [
        CardVariant::Elevated,
        CardVariant::Filled,
        CardVariant::Outlined,
    ] {
        assert_eq!(a.card(v), b.card(v), "card {v:?}");
    }
    for s in [StepState::Done, StepState::Current, StepState::Todo] {
        assert_eq!(a.step(s), b.step(s), "step {s:?}");
    }
    for kind in [ToggleKind::Switch, ToggleKind::Checkbox, ToggleKind::Radio] {
        for on in [false, true] {
            for st in [
                ControlState::Idle,
                ControlState::Hovered,
                ControlState::Disabled,
            ] {
                assert_eq!(
                    a.choice_toggle(kind, on, st),
                    b.choice_toggle(kind, on, st),
                    "{kind:?} {on} {st:?}"
                );
            }
        }
    }
    for kind in [
        AppBarKind::CenterAligned,
        AppBarKind::Small,
        AppBarKind::Medium,
        AppBarKind::Large,
        AppBarKind::Bottom,
    ] {
        for scrolled in [false, true] {
            assert_eq!(a.app_bar(kind, scrolled), b.app_bar(kind, scrolled));
        }
    }
    for role in [
        TypeRole::DisplayLarge,
        TypeRole::TitleMedium,
        TypeRole::LabelSmall,
    ] {
        assert_eq!(a.type_style(role), b.type_style(role), "{role:?}");
    }
    for focused in [false, true] {
        assert_eq!(a.panel(focused), b.panel(focused));
        assert_eq!(a.input(focused), b.input(focused));
    }
    for rich in [false, true] {
        assert_eq!(a.tooltip(rich), b.tooltip(rich));
    }
    assert_eq!(a.sizes(), b.sizes());
    assert_eq!(a.navigation(), b.navigation());
    assert_eq!(a.dialog(), b.dialog());
    assert_eq!(a.sheet(), b.sheet());
    assert_eq!(a.popup(), b.popup());
    assert_eq!(a.snackbar(), b.snackbar());
    assert_eq!(a.scrim(), b.scrim());
    assert_eq!(a.divider(), b.divider());
    assert_eq!(a.focus_ring(), b.focus_ring());
    assert_eq!(a.list_row_height(true), b.list_row_height(true));
    assert_eq!(a.list_lines_height(2), b.list_lines_height(2));
    assert_eq!(a.menu_row_height(), b.menu_row_height());
    assert_eq!(a.page_grid(900.0), b.page_grid(900.0));
    assert_eq!(a.hard_shadow_color(), b.hard_shadow_color());
    assert_eq!(a.fade_stops(), b.fade_stops());
}
