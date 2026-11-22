//! Server-render the components added for parity and check what they emit.

use dodai_ui_core::calendar::{Date, DateRange};
use dodai_ui_core::Tone;
use dodai_ui_icons::IconId;
use dodai_ui_yew::*;
use yew::prelude::*;

fn render<C: BaseComponent>(props: C::Properties) -> String
where
    C::Properties: Send,
{
    tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(yew::ServerRenderer::<C>::with_props(move || props).render())
}

#[function_component]
fn Nav() -> Html {
    html! { <ThemeProvider>
        <NavigationBar>
            <NavigationItem icon={IconId::Home} label="Home" selected=true />
            <NavigationItem icon={IconId::Search} label="Find" />
        </NavigationBar>
        <BottomSheet open=false label="Hidden">{ "x" }</BottomSheet>
        <SideSheet open=true label="Filters">{ "body" }</SideSheet>
    </ThemeProvider> }
}

#[test]
fn navigation_and_sheets() {
    let h = render::<Nav>(());
    assert!(h.contains("Home") && h.contains("Find"));
    assert!(!h.contains("Hidden"));
    assert!(h.contains("aria-label=\"Filters\""));
}

#[function_component]
fn Pickers() -> Html {
    html! { <ThemeProvider>
        <DatePicker value={Date::new(2024, 2, 14)} />
        <TimePicker hour=15 minute=5 />
        <Stopwatch ms=61_500 running=false laps={vec![1000, 3000]} />
        <Timer remaining_ms=90_000 running=true />
        <Clock seconds=3723.0 />
    </ThemeProvider> }
}

#[test]
fn pickers_and_clocks_use_the_shared_maths() {
    let h = render::<Pickers>(());
    assert!(h.contains("February 2024") && h.contains("aria-pressed=\"true\""));
    assert!(h.contains("PM") && h.contains("Lap 2") && h.contains("Pause"));
    assert!(h.contains("01:02:03"));
}

#[function_component]
fn Range() -> Html {
    let r = DateRange {
        start: Some(Date::new(2025, 3, 3)),
        end: Some(Date::new(2025, 3, 5)),
    };
    html! { <ThemeProvider><DateTimeRangePicker range={r} /></ThemeProvider> }
}

#[test]
fn a_range_marks_both_ends_and_between() {
    assert_eq!(
        render::<Range>(()).matches("aria-pressed=\"true\"").count(),
        3
    );
}

#[function_component]
fn Md() -> Html {
    html! { <ThemeProvider><Markdown source="# Title\n\nSome **bold** and `code`.\n\n- one\n- two\n\n---" /></ThemeProvider> }
}

#[test]
fn markdown_renders_blocks_and_marks() {
    let h = render::<Md>(());
    assert!(
        h.contains("Title")
            && h.contains("<strong>bold</strong>")
            && h.contains("<code")
            && h.contains("<hr")
    );
}

#[function_component]
fn Forms() -> Html {
    html! { <ThemeProvider>
        <ProgressBar value=0.4 label="Upload" />
        <FormField label="Name" error="Required"><TextInput value="" /></FormField>
        <TagField tags={vec![AttrValue::from("a"), AttrValue::from("b")]} />
        <OtpInput value="12" len=4 />
        <Banner tone={Tone::Danger}>{ "Oops" }</Banner>
        <NumberStepper value=3 min=0 max=3 />
    </ThemeProvider> }
}

#[test]
fn forms_show_their_state() {
    let h = render::<Forms>(());
    assert!(
        h.contains("40%")
            && h.contains("Required")
            && h.contains("a ×")
            && h.contains("maxlength=\"4\"")
    );
    assert!(h.contains("disabled"), "the stepper stops at its maximum");
}

#[function_component]
fn Spy() -> Html {
    html! { <ThemeProvider>
        <ScrollSpy sections={vec![(AttrValue::from("A"), -300.0), (AttrValue::from("B"), 10.0), (AttrValue::from("C"), 500.0)]} />
    </ThemeProvider> }
}

#[test]
fn scroll_spy_renders_every_section() {
    let h = render::<Spy>(());
    assert!(h.contains(">A<") && h.contains(">B<") && h.contains(">C<"));
}

#[test]
fn live_samples_share_one_window() {
    let a = Live::new(3);
    let b = a.clone();
    b.push(1.0);
    b.push(2.0);
    assert_eq!(a.samples(), vec![0.0, 1.0, 2.0]);
    assert!(a == b);
}

#[function_component]
fn Selectors() -> Html {
    html! { <ThemeProvider><ThemeSelector /><DesignSystemSelector /><Knob value=0.5 /><AxisGizmo yaw=0.5 pitch=0.3 /></ThemeProvider> }
}

#[test]
fn selectors_list_every_theme_and_the_svgs_render() {
    let h = render::<Selectors>(());
    for t in dodai_ui_core::themes::all() {
        assert!(h.contains(t.name()), "{}", t.name());
    }
    assert!(h.contains("class=\"dodai-knob\"") && h.contains("class=\"dodai-gizmo\""));
}
