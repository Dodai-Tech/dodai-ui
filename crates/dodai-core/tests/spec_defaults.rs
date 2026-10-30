//! A design spec that overrides nothing resolves every component exactly like a theme with no
//! spec: the default hooks return `None` and the theme falls back to its own tokens.

use dodai_core::{
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
