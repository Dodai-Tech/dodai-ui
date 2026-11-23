//! Server-render the library components added for parity and check what they emit.

use dioxus::prelude::*;
use dodai_core::calendar::{Date, DateRange};
use dodai_dioxus::*;
use dodai_icons::IconId;

fn html(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[test]
fn navigation_marks_the_selected_item_and_sheets_hide_when_closed() {
    let out = html(|| {
        rsx! {
            ThemeProvider {
                NavigationBar {
                    NavigationItem { icon: IconId::Home, label: "Home", selected: true }
                    NavigationItem { icon: IconId::Search, label: "Find" }
                }
                BottomSheet { open: false, label: "Hidden", "x" }
                SideSheet { open: true, label: "Filters", "body" }
            }
        }
    });
    assert!(out.contains("Home") && out.contains("Find"));
    assert!(!out.contains("Hidden"));
    assert!(out.contains("aria-label=\"Filters\""));
}

#[test]
fn date_picker_shows_the_month_and_marks_the_picked_day() {
    let out = html(|| {
        rsx! { ThemeProvider { DatePicker { value: Date::new(2024, 2, 14) } } }
    });
    assert!(out.contains("February 2024"));
    assert!(out.contains("aria-pressed=\"true\""));
    assert!(out.contains(">29<"), "leap day present");
}

#[test]
fn range_picker_marks_both_ends() {
    let out = html(|| {
        let r = DateRange {
            start: Some(Date::new(2025, 3, 3)),
            end: Some(Date::new(2025, 3, 5)),
        };
        rsx! { ThemeProvider { DateTimeRangePicker { range: r } } }
    });
    assert_eq!(out.matches("aria-pressed=\"true\"").count(), 3);
}

#[test]
fn time_and_clock_faces_format_from_the_shared_maths() {
    let out = html(|| {
        rsx! {
            ThemeProvider {
                TimePicker { hour: 15, minute: 5 }
                Stopwatch { ms: 61_500, running: false, laps: vec![1000, 3000] }
                Timer { remaining_ms: 90_000, running: true }
                Clock { seconds: 3723.0 }
            }
        }
    });
    assert!(out.contains(">03<") && out.contains(">05<") && out.contains("PM"));
    assert!(out.contains("Lap 2"));
    assert!(out.contains("Pause"));
    assert!(out.contains("01:02:03"));
}

#[test]
fn markdown_renders_blocks_and_inline_marks() {
    let out = html(|| {
        rsx! { ThemeProvider { Markdown { source: "# Title\n\nSome **bold** and `code`.\n\n- one\n- two\n\n---".to_string() } } }
    });
    assert!(out.contains("Title"));
    assert!(out.contains("<strong>bold</strong>"));
    assert!(out.contains("<code"));
    assert!(out.contains("<hr"));
}

#[test]
fn forms_and_progress_render_their_state() {
    let out = html(|| {
        rsx! {
            ThemeProvider {
                ProgressBar { value: 0.4, label: "Upload".to_string() }
                NumberStepper { value: 3, min: 0, max: 3 }
                OtpInput { value: "12".to_string(), len: 4 }
                FormField { label: "Name", error: "Required".to_string(), TextInput { value: String::new() } }
                TagField { tags: vec!["a".to_string(), "b".to_string()] }
            }
        }
    });
    assert!(out.contains("40%"));
    assert!(out.contains("Required"));
    assert!(out.contains("a ×") && out.contains("b ×"));
    assert!(out.contains("maxlength=\"4\""));
}

#[test]
fn scroll_spy_follows_the_shared_rule() {
    let out = html(|| {
        rsx! {
            ThemeProvider {
                ScrollSpy { sections: vec![("A".to_string(), -300.0), ("B".to_string(), 10.0), ("C".to_string(), 500.0)] }
            }
        }
    });
    // B is the last section whose top reached the viewport top, so only its row is selected.
    let row = |label: &str| {
        let at = out.find(&format!(">{label}<")).unwrap();
        let start = out[..at].rfind("<button").unwrap();
        out[start..at].contains("row-on")
    };
    assert!(row("B") && !row("A") && !row("C"));
}

#[test]
fn live_samples_share_one_window() {
    let a = Live::new(3);
    let b = a.clone();
    b.push(1.0);
    b.push(2.0);
    assert_eq!(a.samples(), vec![0.0, 1.0, 2.0]);
    assert_eq!(a.pushed() - Live::new(3).pushed(), 2);
    assert!(a == b);
}
