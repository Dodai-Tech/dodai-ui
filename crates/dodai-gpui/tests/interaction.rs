//! Headless interaction tests: real clicks / keys against rendered components.

use dodai_gpui::*;
use gpui::{
    div, point, prelude::*, px, size, AnyElement, Context, Focusable, Modifiers, TestAppContext,
    VisualTestContext, Window,
};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

type Build = Box<dyn Fn(&mut Window, &mut Context<Host>) -> AnyElement>;

/// A view that rebuilds its single child from a closure on every render.
struct Host {
    build: Build,
}

impl Render for Host {
    fn render(&mut self, w: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // shrink-wrapped, top-left anchored, so bounds are those of the component
        div().flex().child(
            div()
                .debug_selector(|| "target".into())
                .flex_none()
                .child((self.build)(w, cx)),
        )
    }
}

fn mount(
    cx: &mut TestAppContext,
    build: impl Fn(&mut Window, &mut Context<Host>) -> AnyElement + 'static,
) -> &mut VisualTestContext {
    cx.update(|cx| install(cx, dodai_core::Theme::default()));
    let (_, vcx) = cx.add_window_view(|_, _| Host {
        build: Box::new(build),
    });
    vcx.simulate_resize(size(px(900.0), px(700.0)));
    vcx.run_until_parked();
    vcx
}

fn click(vcx: &mut VisualTestContext, x: f32, y: f32) {
    let p = point(px(x), px(y));
    vcx.simulate_click(p, Modifiers::none());
    vcx.run_until_parked();
}

#[gpui::test]
fn carousel_arrows_select_neighbouring_pages(cx: &mut TestAppContext) {
    let picked = Rc::new(RefCell::new(Vec::<usize>::new()));
    let p = picked.clone();
    let vcx = mount(cx, move |_, _| {
        let p = p.clone();
        Carousel::new("car")
            .visible(2)
            .item_width(80.0)
            .active(1)
            .on_select(move |i, _, _| p.borrow_mut().push(i))
            .item("a")
            .item("b")
            .item("c")
            .item("d")
            .into_any_element()
    });
    let b = vcx.debug_bounds("target").expect("carousel was laid out");
    let (l, r, t) = (
        f32::from(b.left()),
        f32::from(b.right()),
        f32::from(b.top()),
    );
    // arrows sit at both ends of the first row
    click(vcx, r - 14.0, t + 14.0);
    click(vcx, l + 14.0, t + 14.0);
    assert_eq!(*picked.borrow(), vec![2, 0], "next then prev");
}

type Log = Rc<RefCell<Vec<usize>>>;

fn log() -> Log {
    Rc::new(RefCell::new(Vec::new()))
}

fn target(vcx: &mut VisualTestContext) -> (f32, f32, f32, f32) {
    let b = vcx.debug_bounds("target").expect("laid out");
    (
        f32::from(b.left()),
        f32::from(b.top()),
        f32::from(b.size.width),
        f32::from(b.size.height),
    )
}

#[gpui::test]
fn menu_selects_the_clicked_item_and_skips_disabled(cx: &mut TestAppContext) {
    let l = log();
    let l2 = l.clone();
    let vcx = mount(cx, move |_, _| {
        let l = l2.clone();
        Menu::new("m")
            .items([
                MenuItem::new("Cut"),
                MenuItem::new("Copy").disabled(true),
                MenuItem::new("Paste"),
            ])
            .on_select(move |i, _, _| l.borrow_mut().push(i))
            .into_any_element()
    });
    let (x, y, w, h) = target(vcx);
    for i in [0usize, 1, 2] {
        click(vcx, x + w / 2.0, y + h * (i as f32 + 0.5) / 3.0);
    }
    assert_eq!(*l.borrow(), vec![0, 2], "disabled item must not fire");
}

#[gpui::test]
fn segmented_changes_selection(cx: &mut TestAppContext) {
    let l = log();
    let l2 = l.clone();
    let vcx = mount(cx, move |_, _| {
        let l = l2.clone();
        Segmented::new("s", ["AAAA", "BBBB", "CCCC"])
            .on_change(move |i, _, _| l.borrow_mut().push(i))
            .into_any_element()
    });
    let (x, y, w, h) = target(vcx);
    click(vcx, x + w * 0.83, y + h / 2.0);
    click(vcx, x + w * 0.5, y + h / 2.0);
    assert_eq!(*l.borrow(), vec![2, 1]);
}

#[gpui::test]
fn pagination_next_and_prev_stay_in_range(cx: &mut TestAppContext) {
    let l = log();
    let l2 = l.clone();
    let vcx = mount(cx, move |_, _| {
        let l = l2.clone();
        Pagination::new("pg", 3, 12)
            .on_change(move |p, _, _| l.borrow_mut().push(p))
            .into_any_element()
    });
    let (x, y, w, h) = target(vcx);
    click(vcx, x + w - 14.0, y + h / 2.0);
    click(vcx, x + 14.0, y + h / 2.0);
    let got = l.borrow().clone();
    assert_eq!(got.len(), 2, "both arrows fire: {got:?}");
    assert_eq!((got[0], got[1]), (4, 2));
}

#[gpui::test]
fn breadcrumb_reports_clicked_crumb(cx: &mut TestAppContext) {
    let l = log();
    let l2 = l.clone();
    let vcx = mount(cx, move |_, _| {
        let l = l2.clone();
        Breadcrumb::new("bc", ["Home", "Library", "Data"])
            .on_click(move |i, _, _| l.borrow_mut().push(i))
            .into_any_element()
    });
    let (x, y, _, h) = target(vcx);
    click(vcx, x + 12.0, y + h / 2.0);
    assert_eq!(*l.borrow(), vec![0]);
}

#[gpui::test]
fn dialog_scrim_click_dismisses(cx: &mut TestAppContext) {
    let hits = Rc::new(Cell::new(0));
    let h2 = hits.clone();
    let vcx = mount(cx, move |_, _| {
        let h = h2.clone();
        // the scrim is absolutely positioned: give it a full-size container
        div()
            .relative()
            .w(px(900.0))
            .h(px(700.0))
            .child(
                Dialog::new("d", DialogCard::new("Title"))
                    .on_dismiss(move |_, _| h.set(h.get() + 1)),
            )
            .into_any_element()
    });
    click(vcx, 880.0, 680.0);
    assert!(hits.get() >= 1, "scrim click should dismiss");
}

// ───────────────────────────── text input ─────────────────────────────

fn input_page(
    cx: &mut TestAppContext,
    make: impl FnOnce(&mut Context<TextInput>) -> TextInput,
) -> (gpui::Entity<TextInput>, &mut VisualTestContext) {
    cx.update(|cx| install(cx, dodai_core::Theme::default()));
    let (input, vcx) = cx.add_window_view(|_, cx| make(cx));
    vcx.simulate_resize(size(px(600.0), px(300.0)));
    vcx.update(|w, cx| input.focus_handle(cx).focus(w));
    vcx.run_until_parked();
    (input, vcx)
}

fn text(input: &gpui::Entity<TextInput>, vcx: &mut VisualTestContext) -> String {
    vcx.update(|_, cx| input.read(cx).text().to_string())
}

#[gpui::test]
fn text_input_edits_with_the_keyboard(cx: &mut TestAppContext) {
    let (input, vcx) = input_page(cx, TextInput::new);
    vcx.simulate_keystrokes("h e l l o");
    assert_eq!(text(&input, vcx), "hello");
    vcx.simulate_keystrokes("backspace");
    assert_eq!(text(&input, vcx), "hell");
    vcx.simulate_keystrokes("home x");
    assert_eq!(text(&input, vcx), "xhell");
    vcx.simulate_keystrokes("end y");
    assert_eq!(text(&input, vcx), "xhelly");
    vcx.simulate_keystrokes("left left delete");
    assert_eq!(text(&input, vcx), "xhely");
}

#[gpui::test]
fn text_input_selection_replaces_text(cx: &mut TestAppContext) {
    let (input, vcx) = input_page(cx, |cx| TextInput::new(cx).value("abcdef"));
    vcx.simulate_keystrokes("shift-left shift-left z");
    assert_eq!(text(&input, vcx), "abcdz");
    vcx.simulate_keystrokes("cmd-a q");
    assert_eq!(text(&input, vcx), "q");
}

#[gpui::test]
fn text_input_submit_and_multiline(cx: &mut TestAppContext) {
    let sent = Rc::new(RefCell::new(String::new()));
    let s2 = sent.clone();
    let (input, vcx) = input_page(cx, move |cx| {
        TextInput::new(cx).on_submit(move |t, _, _| *s2.borrow_mut() = t.to_string())
    });
    vcx.simulate_keystrokes("o k enter");
    assert_eq!(&*sent.borrow(), "ok");
    assert_eq!(text(&input, vcx), "ok", "enter must not insert a newline");

    let (area, vcx) = input_page(cx, |cx| TextInput::new(cx).multiline(3));
    vcx.simulate_keystrokes("a enter b");
    assert_eq!(text(&area, vcx), "a\nb");
}

#[gpui::test]
fn text_input_disabled_ignores_typing_and_change_fires(cx: &mut TestAppContext) {
    let (dis, vcx) = input_page(cx, |cx| TextInput::new(cx).disabled(true));
    vcx.simulate_keystrokes("x");
    assert_eq!(text(&dis, vcx), "");

    let seen = Rc::new(RefCell::new(Vec::<String>::new()));
    let s2 = seen.clone();
    let (_i, vcx) = input_page(cx, move |cx| {
        TextInput::new(cx).on_change(move |t, _, _| s2.borrow_mut().push(t.to_string()))
    });
    vcx.simulate_keystrokes("a b");
    assert_eq!(*seen.borrow(), vec!["a".to_string(), "ab".to_string()]);
}

// ───────────────────────────── palette / sheets ─────────────────────────────

#[gpui::test]
fn command_palette_runs_first_match_on_enter_and_clicked_row(cx: &mut TestAppContext) {
    cx.update(|cx| install(cx, dodai_core::Theme::default()));
    let ran = log();
    let r2 = ran.clone();
    let (host, vcx) = cx.add_window_view(|w, cx| {
        let input = cx.new(|cx| TextInput::new(cx).placeholder("type a command"));
        let _ = w;
        PaletteHost { input, ran: r2 }
    });
    vcx.simulate_resize(size(px(900.0), px(700.0)));
    let input = host.read_with(vcx, |h, _| h.input.clone());
    vcx.update(|w, cx| input.focus_handle(cx).focus(w));
    vcx.run_until_parked();
    vcx.simulate_keystrokes("p a");
    vcx.simulate_keystrokes("enter");
    assert_eq!(*ran.borrow(), vec![2], "'pa' matches Paste (index 2) first");
}

struct PaletteHost {
    input: gpui::Entity<TextInput>,
    ran: Log,
}

impl Render for PaletteHost {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let ran = self.ran.clone();
        div().relative().size_full().child(
            CommandPalette::new("cp", self.input.clone())
                .commands([
                    Command::new("Cut"),
                    Command::new("Copy"),
                    Command::new("Paste"),
                ])
                .on_run(move |i, _, _| ran.borrow_mut().push(i)),
        )
    }
}

#[gpui::test]
fn bottom_sheet_scrim_dismisses(cx: &mut TestAppContext) {
    let hits = Rc::new(Cell::new(0));
    let h2 = hits.clone();
    let vcx = mount(cx, move |_, _| {
        let h = h2.clone();
        div()
            .relative()
            .w(px(900.0))
            .h(px(700.0))
            .child(
                BottomSheet::new("bs")
                    .title("Sheet")
                    .on_dismiss(move |_, _| h.set(h.get() + 1)),
            )
            .into_any_element()
    });
    click(vcx, 450.0, 40.0); // above the sheet = on the scrim
    assert!(hits.get() >= 1);
}

// ───────────────────────────── tree / select ─────────────────────────────

#[gpui::test]
fn tree_row_click_selects_and_reports_the_path(cx: &mut TestAppContext) {
    let picked = Rc::new(RefCell::new(Vec::<Vec<usize>>::new()));
    let p2 = picked.clone();
    let vcx = mount(cx, move |_, _| {
        let p = p2.clone();
        TreeView::new(
            "t",
            vec![
                TreeItem::new("Root")
                    .expanded(true)
                    .child(TreeItem::new("Leaf")),
                TreeItem::new("Other"),
            ],
        )
        .on_select(move |path, _, _| p.borrow_mut().push(path.to_vec()))
        .into_any_element()
    });
    let (x, y, w, h) = target(vcx);
    // three visible rows: Root, Leaf, Other
    click(vcx, x + w * 0.6, y + h * (1.5 / 3.0));
    click(vcx, x + w * 0.6, y + h * (2.5 / 3.0));
    assert_eq!(*picked.borrow(), vec![vec![0, 0], vec![1]]);
}

#[gpui::test]
fn select_opens_and_picks_an_option(cx: &mut TestAppContext) {
    let l = log();
    let l2 = l.clone();
    let vcx = mount(cx, move |_, _| {
        let l = l2.clone();
        Select::new("sel")
            .options(["One", "Two", "Three"])
            .on_change(move |i, _, _| l.borrow_mut().push(i))
            .into_any_element()
    });
    let (x, y, w, h) = target(vcx);
    click(vcx, x + w / 2.0, y + h / 2.0); // open
                                          // options list drops below the trigger, one row per option
    click(vcx, x + w / 2.0, y + h + h * 1.5);
    assert_eq!(
        l.borrow().len(),
        1,
        "an option was chosen: {:?}",
        l.borrow()
    );
}
