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
            .selected(1)
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
            .on_select(move |i, _, _| l.borrow_mut().push(i))
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
            .on_select(move |i, _, _| l.borrow_mut().push(i))
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
            .on_select(move |i, _, _| l.borrow_mut().push(i))
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

// ───────────────────────────── popovers / tables / chips ─────────────────────────────

#[gpui::test]
fn popover_opens_on_trigger_click_and_closes_outside(cx: &mut TestAppContext) {
    let vcx = mount(cx, |_, _| {
        Popover::new("pop")
            .trigger(Button::new("pb", "Open"))
            .content(div().debug_selector(|| "pop-body".into()).child("hello"))
            .into_any_element()
    });
    assert!(vcx.debug_bounds("pop-body").is_none(), "closed initially");
    let (x, y, w, h) = target(vcx);
    click(vcx, x + w / 2.0, y + h / 2.0);
    assert!(vcx.debug_bounds("pop-body").is_some(), "opened by click");
    click(vcx, 850.0, 650.0);
    assert!(
        vcx.debug_bounds("pop-body").is_none(),
        "closed by outside click"
    );
}

#[gpui::test]
fn dropdown_menu_opens_and_selects(cx: &mut TestAppContext) {
    let l = log();
    let l2 = l.clone();
    let vcx = mount(cx, move |_, _| {
        let l = l2.clone();
        DropdownMenu::new("dd", "Actions")
            .items([MenuItem::new("Edit"), MenuItem::new("Delete")])
            .on_select(move |i, _, _| l.borrow_mut().push(i))
            .into_any_element()
    });
    let (x, y, w, h) = target(vcx);
    click(vcx, x + w / 2.0, y + h / 2.0);
    // the menu is below the trigger; probe rows until one fires
    for k in 1..8 {
        click(vcx, x + w / 2.0, y + h + 6.0 + 14.0 * k as f32);
        if !l.borrow().is_empty() {
            break;
        }
    }
    assert_eq!(l.borrow().len(), 1, "a menu item fired");
}

#[gpui::test]
fn table_row_click_and_header_sort(cx: &mut TestAppContext) {
    let picked = log();
    let sorted = log();
    let (p2, s2) = (picked.clone(), sorted.clone());
    let vcx = mount(cx, move |_, _| {
        let (p, s) = (p2.clone(), s2.clone());
        div()
            .w(px(400.0))
            .child(
                Table::new("tb")
                    .columns(["Name", "Score"])
                    .rows([["Ada", "9"], ["Bob", "7"], ["Cy", "5"]])
                    .on_select(move |i, _, _| p.borrow_mut().push(i))
                    .on_sort(move |c, _, _| s.borrow_mut().push(c)),
            )
            .into_any_element()
    });
    let (x, y, w, h) = target(vcx);
    // header is the first of four rows
    click(vcx, x + w * 0.9, y + 20.0); // header cells are text-height, centred
    click(vcx, x + w * 0.5, y + h - 10.0);
    assert_eq!(*sorted.borrow(), vec![1], "right column header sorts");
    assert_eq!(*picked.borrow(), vec![2], "third body row selected");
}

#[gpui::test]
fn chip_click_and_remove(cx: &mut TestAppContext) {
    let clicks = Rc::new(Cell::new(0));
    let removes = Rc::new(Cell::new(0));
    let (c2, r2) = (clicks.clone(), removes.clone());
    let vcx = mount(cx, move |_, _| {
        let (c, r) = (c2.clone(), r2.clone());
        Chip::new("chip", "Rust")
            .on_click(move |_, _, _| c.set(c.get() + 1))
            .on_remove(move |_, _, _| r.set(r.get() + 1))
            .into_any_element()
    });
    let (x, y, w, h) = target(vcx);
    click(vcx, x + 10.0, y + h / 2.0);
    click(vcx, x + w - 20.0, y + h / 2.0);
    assert_eq!(clicks.get(), 1, "remove must not count as a click");
    assert_eq!(removes.get(), 1);
}

#[gpui::test]
fn side_sheet_scrim_dismisses(cx: &mut TestAppContext) {
    let hits = Rc::new(Cell::new(0));
    let h2 = hits.clone();
    let vcx = mount(cx, move |_, _| {
        let h = h2.clone();
        div()
            .relative()
            .w(px(900.0))
            .h(px(700.0))
            .child(
                SideSheet::new("ss", "Side")
                    .width(200.0)
                    .on_dismiss(move |_, _| h.set(h.get() + 1)),
            )
            .into_any_element()
    });
    // far left of a right-edge sheet is the scrim
    click(vcx, 20.0, 350.0);
    assert!(hits.get() >= 1, "scrim click dismisses");
}

// ───────────────────────────── extras ─────────────────────────────

#[gpui::test]
fn number_stepper_buttons_clamp_to_range(cx: &mut TestAppContext) {
    let seen = Rc::new(RefCell::new(Vec::<i32>::new()));
    let s2 = seen.clone();
    let vcx = mount(cx, move |_, _| {
        let s = s2.clone();
        NumberStepper::new("ns", 9)
            .range(0, 10)
            .step(2)
            .on_change(move |v, _, _| s.borrow_mut().push(v))
            .into_any_element()
    });
    let (x, y, w, h) = target(vcx);
    click(vcx, x + w - 10.0, y + h / 2.0); // + : 9 + 2 clamps to 10
    click(vcx, x + 10.0, y + h / 2.0); // − : 9 - 2 = 7
    let got = seen.borrow().clone();
    assert_eq!(got, vec![10, 7], "plus clamps at max, minus steps down");
}

#[gpui::test]
fn rating_click_sets_star_count(cx: &mut TestAppContext) {
    let seen = Rc::new(RefCell::new(Vec::<u32>::new()));
    let s2 = seen.clone();
    let vcx = mount(cx, move |_, _| {
        let s = s2.clone();
        Rating::new("r", 1)
            .max(5)
            .on_change(move |v, _, _| s.borrow_mut().push(v))
            .into_any_element()
    });
    let (x, y, w, h) = target(vcx);
    click(vcx, x + w - 6.0, y + h / 2.0); // last star
    assert_eq!(*seen.borrow(), vec![5]);
}

#[gpui::test]
fn color_picker_selects_swatch(cx: &mut TestAppContext) {
    let l = log();
    let l2 = l.clone();
    let vcx = mount(cx, move |_, _| {
        let l = l2.clone();
        ColorPicker::new(
            "cp",
            [
                dodai_core::Color::hex(0xff0000),
                dodai_core::Color::hex(0x00ff00),
                dodai_core::Color::hex(0x0000ff),
            ],
        )
        .on_select(move |i, _, _| l.borrow_mut().push(i))
        .into_any_element()
    });
    let (x, y, w, h) = target(vcx);
    click(vcx, x + w - 8.0, y + h / 2.0);
    assert_eq!(*l.borrow(), vec![2], "last swatch");
}

// ───────────────────────────── scroll-spy ─────────────────────────────

/// Four 300px sections in a 500px viewport: only ~400px of scroll range, so
/// the last sections can never reach the top. The old highlight stuck on the
/// first one.
fn spy_page(cx: &mut TestAppContext) -> (gpui::ScrollHandle, &mut VisualTestContext) {
    let handle = gpui::ScrollHandle::new();
    let h = handle.clone();
    let vcx = mount(cx, move |_, _| {
        let mut col = div()
            .id("spy-scroll")
            .track_scroll(&h)
            .overflow_y_scroll()
            .w(px(300.0))
            .h(px(500.0));
        for _ in 0..4 {
            col = col.child(div().h(px(300.0)).flex_none());
        }
        col.into_any_element()
    });
    (handle, vcx)
}

const SPY_ITEMS: [(char, usize); 4] = [('a', 0), ('b', 1), ('c', 2), ('d', 3)];

#[gpui::test]
fn scroll_spy_follows_scroll_and_reaches_the_last_section(cx: &mut TestAppContext) {
    let (handle, vcx) = spy_page(cx);
    let spy = ScrollSpy::new();
    assert_eq!(spy.current(&handle, &SPY_ITEMS, 0.0), Some('a'));
    handle.set_offset(point(px(0.0), px(-350.0)));
    vcx.run_until_parked();
    assert_eq!(spy.current(&handle, &SPY_ITEMS, 0.0), Some('b'));
    // scrolled to the very end: 'd' top (y=900) never reaches the viewport top
    let max = handle.max_offset().height;
    handle.set_offset(point(px(0.0), -max));
    vcx.run_until_parked();
    assert_eq!(
        spy.current(&handle, &SPY_ITEMS, 0.0),
        Some('d'),
        "last section must highlight at the end of the range"
    );
}

#[gpui::test]
fn scroll_spy_pin_survives_until_released(cx: &mut TestAppContext) {
    let (handle, _vcx) = spy_page(cx);
    let mut spy = ScrollSpy::new();
    spy.pin('c'); // clicked, but the page can't scroll it to the top
    assert_eq!(spy.current(&handle, &SPY_ITEMS, 0.0), Some('c'));
    spy.release();
    assert_eq!(spy.current(&handle, &SPY_ITEMS, 0.0), Some('a'));
}

// ───────────────────────────── overscroll ─────────────────────────────

struct Stretchy {
    scroll: gpui::ScrollHandle,
    over: Overscroll,
}

impl Render for Stretchy {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().w(px(300.0)).h(px(300.0)).overflow_hidden().child(
            div()
                .id("os")
                .track_scroll(&self.scroll)
                .relative()
                .top(px(self.over.offset()))
                .overflow_y_scroll()
                .size_full()
                .on_scroll_wheel(cx.listener(|s, e: &gpui::ScrollWheelEvent, _, cx| {
                    let dy = dodai_gpui::overscroll::wheel_dy(&e.delta);
                    let (top, bottom) = dodai_gpui::overscroll::edges(&s.scroll);
                    s.over.on_wheel(dy, top, bottom, std::time::Instant::now());
                    cx.notify();
                }))
                .child(
                    div()
                        .debug_selector(|| "os-first".into())
                        .h(px(200.0))
                        .flex_none(),
                )
                .child(div().h(px(200.0)).flex_none()),
        )
    }
}

fn wheel(vcx: &mut VisualTestContext, dy: f32) {
    vcx.simulate_event(gpui::ScrollWheelEvent {
        position: point(px(100.0), px(100.0)),
        delta: gpui::ScrollDelta::Pixels(point(px(0.0), px(dy))),
        modifiers: Modifiers::none(),
        touch_phase: gpui::TouchPhase::Moved,
    });
    vcx.run_until_parked();
}

#[gpui::test]
fn wheel_past_the_top_stretches_and_mid_page_does_not(cx: &mut TestAppContext) {
    cx.update(|cx| install(cx, dodai_core::Theme::default()));
    let (view, vcx) = cx.add_window_view(|_, _| Stretchy {
        scroll: gpui::ScrollHandle::new(),
        over: Overscroll::default(),
    });
    vcx.simulate_resize(size(px(400.0), px(400.0)));
    vcx.run_until_parked();
    // at the top, pulling down stretches
    wheel(vcx, 30.0);
    let pulled = view.read_with(vcx, |v, _| v.over.offset());
    assert!(pulled > 0.0, "stretched past the top: {pulled}");
    assert!(
        vcx.debug_bounds("os-first").unwrap().top() > px(0.0),
        "content moved"
    );
    // scrolling down (towards content) unwinds the stretch, no scrolling yet
    wheel(vcx, -300.0);
    let after = view.read_with(vcx, |v, _| v.over.offset());
    assert!(
        after < pulled,
        "wheel the other way relaxes: {after} < {pulled}"
    );
}

/// `Icon` follows the theme:
/// every pack/theme combination must paint without panicking.
#[gpui::test]
fn icons_paint_in_every_pack_and_theme(cx: &mut TestAppContext) {
    use dodai_gpui::{Icon, IconPack};
    for theme in dodai::themes::all() {
        cx.update(|cx| install(cx, theme));
        let (_, vcx) = cx.add_window_view(|_, _| Host {
            build: Box::new(|_, _| {
                div()
                    .flex()
                    .children(dodai_cyberpunk_icons::ALL.iter().flat_map(|&n| {
                        [
                            Icon::new(n).size(20.0).into_any_element(),
                            Icon::new(n)
                                .pack(IconPack::Material)
                                .filled(true)
                                .into_any_element(),
                            Icon::new(n).pack(IconPack::Stroked).into_any_element(),
                        ]
                    }))
                    .child(Icon::Home.size(0.0))
                    .child(Icon::Heart.modifier(Modifier::Add).size(24.0))
                    .child(Icon::Acorn)
                    .child(Icon::HouseLine.filled(true))
                    .into_any_element()
            }),
        });
        vcx.simulate_resize(size(px(900.0), px(700.0)));
        vcx.run_until_parked();
    }
}

#[gpui::test]
fn lazy_blocks_far_from_the_window_are_not_laid_out(cx: &mut TestAppContext) {
    let vcx = mount(cx, move |_, _| {
        div()
            .flex()
            .flex_col()
            .child(lazy(
                "lz-near",
                div().h(px(100.0)).debug_selector(|| "lz-near-body".into()),
            ))
            .child(div().h(px(4000.0)))
            .child(div().debug_selector(|| "lz-far-slot".into()).child(lazy(
                "lz-far",
                div().h(px(200.0)).debug_selector(|| "lz-far-body".into()),
            )))
            .into_any_element()
    });
    for _ in 0..2 {
        vcx.update(|w, _| w.refresh());
        vcx.run_until_parked();
    }
    assert!(
        vcx.debug_bounds("lz-near-body").is_some(),
        "near stays realised"
    );
    assert!(
        vcx.debug_bounds("lz-far-body").is_none(),
        "far block is replaced by a spacer"
    );
    let slot = vcx.debug_bounds("lz-far-slot").expect("slot laid out");
    assert_eq!(
        slot.size.height,
        px(200.0),
        "the spacer keeps the measured height"
    );
}
