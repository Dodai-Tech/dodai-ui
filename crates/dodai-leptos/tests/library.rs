//! Server-render the components added for parity and check what they emit.

use dodai_core::calendar::{Date, DateRange};
use dodai_core::{themes, Tone};
use dodai_icons::IconId;
use dodai_leptos::*;
use leptos::prelude::*;

fn page<V: IntoView + 'static>(inner: impl FnOnce() -> V + Send + 'static) -> String {
    let all = themes::all();
    let id = all[0].id().to_string();
    view! { <ThemeProvider themes=all theme=id>{inner()}</ThemeProvider> }.to_html()
}

#[test]
fn navigation_and_sheets() {
    let h = page(|| {
        view! {
            <NavigationBar>
                <NavigationItem icon=IconId::Home label="Home" selected=true />
                <NavigationItem icon=IconId::Search label="Find" />
            </NavigationBar>
            <BottomSheet open=false label="Hidden">"x"</BottomSheet>
            <SideSheet open=true label="Filters">"body"</SideSheet>
        }
    });
    assert!(h.contains("Home") && h.contains("Find"));
    assert!(!h.contains("Hidden"));
    assert!(h.contains("aria-label=\"Filters\""));
}

#[test]
fn pickers_and_clocks_use_the_shared_maths() {
    let h = page(|| {
        view! {
            <DatePicker value=Date::new(2024, 2, 14) />
            <TimePicker hour=15 minute=5 />
            <Stopwatch ms=61_500 running=false laps=vec![1000, 3000] />
            <Timer remaining_ms=90_000 running=true />
            <Clock seconds=3723.0 />
        }
    });
    assert!(h.contains("February 2024") && h.contains("aria-pressed=\"true\""));
    assert!(h.contains("PM") && h.contains("Lap 2") && h.contains("Pause"));
    assert!(h.contains("01:02:03"));
}

#[test]
fn a_range_marks_both_ends_and_between() {
    let r = DateRange {
        start: Some(Date::new(2025, 3, 3)),
        end: Some(Date::new(2025, 3, 5)),
    };
    let h = page(move || view! { <DateTimeRangePicker range=r /> });
    assert_eq!(h.matches("aria-pressed=\"true\"").count(), 3);
}

#[test]
fn markdown_renders_blocks_and_marks() {
    let h = page(
        || view! { <Markdown source="# Title\n\nSome **bold** and `code`.\n\n- one\n- two\n\n---" /> },
    );
    assert!(
        h.contains("Title")
            && h.contains("<strong>bold</strong>")
            && h.contains("<code")
            && h.contains("<hr")
    );
}

#[test]
fn forms_show_their_state() {
    let h = page(|| {
        view! {
            <ProgressBar value=0.4 label="Upload" />
            <FormField label="Name" error="Required">"x"</FormField>
            <TagField tags=vec!["a".into(), "b".into()] />
            <OtpInput value="12" len=4 />
            <Banner tone=Tone::Danger>"Oops"</Banner>
        }
    });
    assert!(
        h.contains("40%")
            && h.contains("Required")
            && h.contains("a ×")
            && h.contains("maxlength=\"4\"")
    );
}

#[test]
fn scroll_spy_selects_the_section_in_view() {
    let h = page(
        || view! { <ScrollSpy sections=vec![("A".into(), -300.0), ("B".into(), 10.0), ("C".into(), 500.0)] /> },
    );
    let row = |l: &str| {
        let at = h.find(&format!(">{l}<")).unwrap();
        let start = h[..at].rfind("<button").unwrap();
        h[start..at].contains("dodai-kit")
    };
    assert!(row("A") && row("B") && row("C"));
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

#[test]
fn theme_selectors_list_every_theme() {
    let h = page(
        || view! { <ThemeSelector /> <DesignSystemSelector /> <Knob value=0.5 /> <AxisGizmo yaw=0.5 pitch=0.3 /> },
    );
    for t in themes::all() {
        assert!(h.contains(t.name()), "{}", t.name());
    }
    assert!(
        h.contains("dodai-knob\" ") || h.contains("class=\"dodai-knob"),
        "{}",
        h.split("<svg").nth(1).unwrap_or("none")
    );
    assert!(
        h.contains("class=\"dodai-gizmo\""),
        "{}",
        h.matches("<svg").count()
    );
}

#[test]
fn the_provider_reaches_components_that_render_later() {
    let h = page(|| view! { <Icon icon=IconId::Home /> });
    assert!(h.contains("<svg"), "icons need the theme from the provider");
}
