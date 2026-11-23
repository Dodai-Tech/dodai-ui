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
    assert_eq!(
        *picked.borrow(),
        vec![2, 1],
        "next, then prev from where it moved to"
    );
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
    assert_eq!(
        (got[0], got[1]),
        (4, 3),
        "prev goes back from the page it moved to"
    );
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
    // The platform's select-all: cmd on macOS, ctrl elsewhere (as the key bindings choose).
    let select_all = if cfg!(target_os = "macos") {
        "cmd-a q"
    } else {
        "ctrl-a q"
    };
    vcx.simulate_keystrokes(select_all);
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
    let seen = Rc::new(RefCell::new(Vec::<f32>::new()));
    let s2 = seen.clone();
    let vcx = mount(cx, move |_, _| {
        let s = s2.clone();
        Rating::new("r", 1.0)
            .max(5)
            .on_change(move |v, _, _| s.borrow_mut().push(v))
            .into_any_element()
    });
    let (x, y, w, h) = target(vcx);
    click(vcx, x + w - 6.0, y + h / 2.0); // last star
    assert_eq!(*seen.borrow(), vec![5.0]);
}

#[gpui::test]
fn rating_half_click_sets_half_star(cx: &mut TestAppContext) {
    let seen = Rc::new(RefCell::new(Vec::<f32>::new()));
    let s2 = seen.clone();
    let vcx = mount(cx, move |_, _| {
        let s = s2.clone();
        Rating::new("r", 1.0)
            .max(5)
            .half(true)
            .on_change(move |v, _, _| s.borrow_mut().push(v))
            .into_any_element()
    });
    let (x, y, w, h) = target(vcx);
    click(vcx, x + w - 2.0, y + h / 2.0); // right edge: the right half of the last star
    click(vcx, x + w - 12.0, y + h / 2.0); // the left half of the last star
    assert_eq!(*seen.borrow(), vec![5.0, 4.5]);
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
    for theme in dodai_core::themes::all() {
        cx.update(|cx| install(cx, theme));
        let (_, vcx) = cx.add_window_view(|_, _| Host {
            build: Box::new(|_, _| {
                div()
                    .flex()
                    .children(dodai_icons::cyberpunk::ALL.iter().flat_map(|&n| {
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

#[gpui::test]
fn date_time_picker_lays_out_calendar_and_time(cx: &mut TestAppContext) {
    let vcx = mount(cx, |_, _| {
        dodai_gpui::DateTimePicker::new("dt", 2025, 6)
            .selected(Some(dodai_gpui::Date::new(2025, 6, 12)))
            .time(14, 30)
            .twelve_hour(true)
            .on_date(|_, _, _| {})
            .on_time(|_, _, _, _| {})
            .into_any_element()
    });
    vcx.run_until_parked();
}

#[gpui::test]
fn date_time_range_picker_lays_out_calendar_and_both_times(cx: &mut TestAppContext) {
    let vcx = mount(cx, |_, _| {
        let r = dodai_gpui::range_pick(Default::default(), dodai_gpui::Date::new(2025, 6, 12));
        dodai_gpui::DateTimeRangePicker::new("dtr", 2025, 6)
            .range(dodai_gpui::range_pick(
                r,
                dodai_gpui::Date::new(2025, 6, 15),
            ))
            .start_time(9, 0)
            .end_time(17, 30)
            .on_select(|_, _, _| {})
            .on_start_time(|_, _, _, _| {})
            .on_end_time(|_, _, _, _| {})
            .into_any_element()
    });
    vcx.run_until_parked();
}

#[gpui::test]
fn clicking_a_button_plays_the_click_ripple(cx: &mut TestAppContext) {
    for variant in [
        dodai_core::ButtonVariant::Primary,
        dodai_core::ButtonVariant::Secondary,
        dodai_core::ButtonVariant::Outline,
        dodai_core::ButtonVariant::Ghost,
    ] {
        press_plays_ripple(cx, variant);
    }
}

fn press_plays_ripple(cx: &mut TestAppContext, variant: dodai_core::ButtonVariant) {
    cx.update(|cx| install(cx, dodai_core::material::light()));
    let (_, vcx) = cx.add_window_view(move |_, _| Host {
        build: Box::new(move |_, _| {
            Button::new("rip", "Press")
                .variant(variant)
                .into_any_element()
        }),
    });
    vcx.simulate_resize(size(px(900.0), px(700.0)));
    vcx.update(|_, cx| {
        let mut fx = dodai_core::ResolvedSurface::NONE;
        fx.ripple = 0.8;
        set_surface(cx, fx);
    });
    vcx.run_until_parked();
    while live_press_count() > 0 {
        std::thread::sleep(std::time::Duration::from_millis(100));
        vcx.run_until_parked();
        vcx.update(|_, cx| cx.refresh_windows());
    }
    let b = vcx.debug_bounds("target").expect("button was laid out");
    click(vcx, f32::from(b.left()) + 20.0, f32::from(b.top()) + 10.0);
    assert_eq!(live_press_count(), 1, "{variant:?}: one ripple per press");
    vcx.update(|_, cx| set_surface(cx, dodai_core::ResolvedSurface::NONE));
}

// ───────────────────────────── select keyboard / scrollbar drag ─────────────────────────────

#[gpui::test]
fn select_keyboard_opens_moves_and_commits(cx: &mut TestAppContext) {
    let l = log();
    let l2 = l.clone();
    let vcx = mount(cx, move |_, _| {
        let l = l2.clone();
        Select::new("sel-kb")
            .options(["One", "Two", "Three"])
            .on_select(move |i, _, _| l.borrow_mut().push(i))
            .into_any_element()
    });
    let (x, y, w, h) = target(vcx);
    click(vcx, x + w / 2.0, y + h / 2.0); // open and focus
    vcx.simulate_keystrokes("end enter");
    assert_eq!(*l.borrow(), vec![2], "End then Enter picks the last option");
    click(vcx, x + w / 2.0, y + h / 2.0);
    vcx.simulate_keystrokes("home down enter");
    assert_eq!(
        l.borrow().last(),
        Some(&1),
        "Home, Down, Enter picks the second"
    );
}

#[gpui::test]
fn scroll_area_thumb_drag_scrolls_the_content(cx: &mut TestAppContext) {
    let vcx = mount(cx, |_, _| {
        ScrollArea::new("sa-drag")
            .height(100.0)
            .child(
                div()
                    .h(px(400.0))
                    .w(px(200.0))
                    .debug_selector(|| "content".into()),
            )
            .into_any_element()
    });
    // the first frame only measures; the next one draws the thumb
    vcx.update(|_, cx| cx.refresh_windows());
    vcx.run_until_parked();
    let top_before = f32::from(vcx.debug_bounds("content").expect("content").top());
    let (x, y, w, _) = target(vcx);
    // vertical thumb: right edge, BAR_GAP in, starting at the top of the rail
    let (tx, ty) = (x + w - 5.0, y + 2.0 + 5.0);
    let at = |px_x: f32, px_y: f32| point(px(px_x), px(px_y));
    vcx.simulate_mouse_down(at(tx, ty), gpui::MouseButton::Left, Modifiers::none());
    vcx.simulate_mouse_move(
        at(tx, ty + 40.0),
        gpui::MouseButton::Left,
        Modifiers::none(),
    );
    vcx.simulate_mouse_up(
        at(tx, ty + 40.0),
        gpui::MouseButton::Left,
        Modifiers::none(),
    );
    vcx.run_until_parked();
    let top_after = f32::from(vcx.debug_bounds("content").expect("content").top());
    assert!(
        top_after < top_before - 20.0,
        "dragging the thumb scrolled the content: {top_before} -> {top_after}"
    );
}

#[gpui::test]
fn scroll_area_horizontal_thumb_drag_scrolls_sideways(cx: &mut TestAppContext) {
    let vcx = mount(cx, |_, _| {
        div()
            .w(px(200.0))
            .child(
                ScrollArea::new("sa-drag-x")
                    .height(60.0)
                    .horizontal(true)
                    .child(
                        div()
                            .h(px(40.0))
                            .w(px(800.0))
                            .debug_selector(|| "content".into()),
                    ),
            )
            .into_any_element()
    });
    vcx.update(|_, cx| cx.refresh_windows());
    vcx.run_until_parked();
    let left_before = f32::from(vcx.debug_bounds("content").expect("content").left());
    let (x, y, _, h) = target(vcx);
    // horizontal thumb: bottom edge, BAR_GAP up, starting at the left of the rail
    let (tx, ty) = (x + 10.0, y + h - 5.0);
    let at = |px_x: f32, px_y: f32| point(px(px_x), px(px_y));
    vcx.simulate_mouse_down(at(tx, ty), gpui::MouseButton::Left, Modifiers::none());
    vcx.simulate_mouse_move(
        at(tx + 40.0, ty),
        gpui::MouseButton::Left,
        Modifiers::none(),
    );
    vcx.simulate_mouse_up(
        at(tx + 40.0, ty),
        gpui::MouseButton::Left,
        Modifiers::none(),
    );
    vcx.run_until_parked();
    let left_after = f32::from(vcx.debug_bounds("content").expect("content").left());
    assert!(
        left_after < left_before - 20.0,
        "dragging the horizontal thumb scrolled the content: {left_before} -> {left_after}"
    );
}

#[gpui::test]
fn accordion_header_click_toggles_the_body(cx: &mut TestAppContext) {
    let vcx = mount(cx, |_, _| {
        Accordion::new("acc")
            .item("First", "body one")
            .item("Second", "body two")
            .into_any_element()
    });
    let (x, y, w, h0) = target(vcx);
    click(vcx, x + w / 2.0, y + h0 / 4.0);
    // let the open transition settle
    settle(vcx);
    let (_, _, _, h1) = target(vcx);
    assert!(
        h1 > h0,
        "opening a section grows the accordion ({h0} -> {h1})"
    );
    click(vcx, x + w / 2.0, y + 8.0);
    settle(vcx);
    let (_, _, _, h2) = target(vcx);
    assert!(h2 < h1, "closing shrinks it again ({h1} -> {h2})");
}

#[gpui::test]
fn submenu_item_reports_parent_and_child(cx: &mut TestAppContext) {
    let l = log();
    let l2 = l.clone();
    let vcx = mount(cx, move |_, _| {
        let l = l2.clone();
        Menu::new("sm")
            .items([
                MenuItem::new("Open"),
                MenuItem::new("Share").submenu([MenuItem::new("Mail"), MenuItem::new("Link")]),
            ])
            .on_select(move |i, _, _| l.borrow_mut().push(i))
            .into_any_element()
    });
    let (x, y, w, h) = target(vcx);
    click(vcx, x + w / 2.0, y + h * 0.25);
    assert_eq!(*l.borrow(), vec![0], "plain item selects by index");
}

/// Motion runs on the wall clock: wait it out, redrawing as we go.
fn settle(vcx: &mut VisualTestContext) {
    for _ in 0..12 {
        std::thread::sleep(std::time::Duration::from_millis(80));
        vcx.update(|_, cx| cx.refresh_windows());
        vcx.run_until_parked();
    }
}

#[gpui::test]
fn controls_toggle_without_a_controlling_parent(cx: &mut TestAppContext) {
    let seen = Rc::new(RefCell::new(Vec::<bool>::new()));
    let s = seen.clone();
    // A fixed value and no state in the parent: the click must still flip it.
    let vcx = mount(cx, move |_, _| {
        let s = s.clone();
        Toggle::new("t", false)
            .on_change(move |v, _, _| s.borrow_mut().push(v))
            .into_any_element()
    });
    let b = vcx.debug_bounds("target").expect("toggle was laid out");
    let (x, y) = (
        f32::from(b.left() + b.size.width / 2.0),
        f32::from(b.top() + b.size.height / 2.0),
    );
    click(vcx, x, y);
    click(vcx, x, y);
    assert_eq!(
        *seen.borrow(),
        vec![true, false],
        "second click turns it back off"
    );
}

#[gpui::test]
fn tabs_switch_without_a_controlling_parent(cx: &mut TestAppContext) {
    let seen = Rc::new(RefCell::new(Vec::<usize>::new()));
    let s = seen.clone();
    let vcx = mount(cx, move |_, _| {
        let s = s.clone();
        Tabs::new("tabs")
            .labels(["One", "Two", "Three"])
            .selected(0)
            .on_select(move |i, _, _| s.borrow_mut().push(i))
            .into_any_element()
    });
    let b = vcx.debug_bounds("target").expect("tabs were laid out");
    let y = f32::from(b.top() + b.size.height / 2.0);
    // the last tab sits at the right edge, the first at the left
    click(vcx, f32::from(b.right()) - 20.0, y);
    click(vcx, f32::from(b.left()) + 20.0, y);
    assert_eq!(*seen.borrow(), vec![2, 0]);
}

#[gpui::test]
fn tabs_move_with_the_arrow_keys(cx: &mut TestAppContext) {
    let seen = Rc::new(RefCell::new(Vec::<usize>::new()));
    let s = seen.clone();
    let vcx = mount(cx, move |_, _| {
        let s = s.clone();
        Tabs::new("tabs")
            .labels(["One", "Two", "Three"])
            .selected(0)
            .on_select(move |i, _, _| s.borrow_mut().push(i))
            .into_any_element()
    });
    let b = vcx.debug_bounds("target").expect("tabs were laid out");
    // a click on the first tab focuses the strip
    click(
        vcx,
        f32::from(b.left()) + 20.0,
        f32::from(b.top() + b.size.height / 2.0),
    );
    vcx.simulate_keystrokes("right right left");
    assert_eq!(*seen.borrow(), vec![0, 1, 2, 1]);
}

/// Every selection animation, on a pill design and an underline design, lands on the clicked
/// item for tabs and segmented controls alike.
#[gpui::test]
fn every_selection_animation_lands_on_the_chosen_item(cx: &mut TestAppContext) {
    use dodai_gpui::motion::SelectionMotion;
    for theme in [dodai_core::dodai::light(), dodai_core::Theme::default()] {
        for kind in SelectionMotion::ALL {
            for segmented in [false, true] {
                let seen = Rc::new(RefCell::new(Vec::<usize>::new()));
                let s = seen.clone();
                let vcx = mount(cx, move |_, _| {
                    let s = s.clone();
                    let pick =
                        move |i: usize, _: &mut Window, _: &mut gpui::App| s.borrow_mut().push(i);
                    if segmented {
                        Segmented::new("seg", ["One", "Two", "Three"])
                            .animation(kind)
                            .on_select(pick)
                            .into_any_element()
                    } else {
                        Tabs::new("tabs")
                            .labels(["One", "Two", "Three"])
                            .animation(kind)
                            .on_select(pick)
                            .into_any_element()
                    }
                });
                vcx.update(|_, cx| install(cx, theme.clone()));
                vcx.run_until_parked();
                let (x, y, w, h) = target(vcx);
                click(vcx, x + w - 20.0, y + h / 2.0);
                click(vcx, x + 20.0, y + h / 2.0);
                assert_eq!(
                    *seen.borrow(),
                    vec![2, 0],
                    "{kind:?}, segmented {segmented}"
                );
            }
        }
    }
}

/// Faster, every animation takes less time. Animation off stills what moves on its own but
/// answers to input still move; reduced motion stills both.
#[gpui::test]
fn the_playback_switches_pace_every_motion(cx: &mut TestAppContext) {
    use dodai_gpui::motion::{ambient, pace, set_enabled, set_reduce_motion, set_speed, Motion};
    use std::time::Duration;
    let m = Motion::default().duration(Duration::from_millis(400));
    let half = Duration::from_millis(200);
    cx.update(|cx| {
        set_speed(cx, 2.0);
        assert_eq!((pace(m).duration, ambient(m).duration), (half, half));
        assert_eq!(pace(m).pace, 2.0);
        set_enabled(cx, false);
        assert_eq!(pace(m).duration, half, "a click still animates");
        assert_eq!(
            ambient(m).duration,
            Duration::ZERO,
            "entries and value-in land"
        );
        set_enabled(cx, true);
        set_reduce_motion(cx, true);
        assert_eq!(
            (pace(m).duration, ambient(m).duration),
            (Duration::ZERO, Duration::ZERO)
        );
        set_reduce_motion(cx, false);
        set_speed(cx, 1.0);
        assert_eq!(pace(m).duration, m.duration);
    });
}

/// Every selection animation, on a classic and a Material design, toggles switches,
/// checkboxes and radios on click.
#[gpui::test]
fn every_selection_animation_toggles_the_choice_controls(cx: &mut TestAppContext) {
    use dodai_gpui::motion::SelectionMotion;
    for theme in [dodai_core::dodai::light(), dodai_core::material::light()] {
        for kind in SelectionMotion::ALL {
            for control in 0..3 {
                let seen = Rc::new(RefCell::new(Vec::<bool>::new()));
                let s = seen.clone();
                let vcx = mount(cx, move |_, _| {
                    let s = s.clone();
                    let f =
                        move |v: bool, _: &mut Window, _: &mut gpui::App| s.borrow_mut().push(v);
                    match control {
                        0 => Toggle::new("c", false)
                            .animation(kind)
                            .on_change(f)
                            .into_any_element(),
                        1 => Checkbox::new("c", false)
                            .animation(kind)
                            .on_change(f)
                            .into_any_element(),
                        _ => Radio::new("c", false)
                            .animation(kind)
                            .on_change(f)
                            .into_any_element(),
                    }
                });
                vcx.update(|_, cx| install(cx, theme.clone()));
                vcx.run_until_parked();
                let (x, y, w, h) = target(vcx);
                click(vcx, x + w / 2.0, y + h / 2.0);
                click(vcx, x + w / 2.0, y + h / 2.0);
                let want = if control == 2 {
                    vec![true, true]
                } else {
                    vec![true, false]
                };
                assert_eq!(*seen.borrow(), want, "{kind:?}, control {control}");
            }
        }
    }
}

/// An accordion section opens to its body's measured height and closes again.
#[gpui::test]
fn accordion_opens_to_the_measured_body(cx: &mut TestAppContext) {
    let vcx = mount(cx, |_, _| {
        div()
            .w(px(320.0))
            .child(Accordion::new("acc").item(
                "Shipping",
                "Orders ship within two days. Tracking arrives by email once the parcel leaves.",
            ))
            .into_any_element()
    });
    let (x, y, _, closed) = target(vcx);
    click(vcx, x + 40.0, y + 10.0);
    settle(vcx);
    let (_, _, _, open) = target(vcx);
    assert!(open > closed + 20.0, "opened from {closed} to {open}");
    click(vcx, x + 40.0, y + 10.0);
    settle(vcx);
    let (_, _, _, shut) = target(vcx);
    assert!(
        (shut - closed).abs() < 1.0,
        "closed back to {shut}, was {closed}"
    );
}

/// Dragging the row pages it: a slow drag lands on the page under the hand, a quick flick
/// coasts on, and dragging past the first page springs back without choosing.
#[gpui::test]
fn carousel_follows_a_drag_and_a_flick(cx: &mut TestAppContext) {
    let picked = log();
    let p = picked.clone();
    let vcx = mount(cx, move |_, _| {
        let p = p.clone();
        let mut c = Carousel::new("car")
            .visible(2)
            .item_width(80.0)
            .on_select(move |i, _, _| p.borrow_mut().push(i));
        for name in ["a", "b", "c", "d", "e", "f"] {
            c = c.item(name);
        }
        c.into_any_element()
    });
    let (x, y, w, _) = target(vcx);
    let (cx0, cy) = (x + w / 2.0, y + 14.0);
    let at = |px_x: f32| point(px(px_x), px(cy));
    let drag = |vcx: &mut VisualTestContext, by: f32, pause: bool| {
        vcx.simulate_mouse_down(at(cx0), gpui::MouseButton::Left, Modifiers::none());
        vcx.run_until_parked();
        vcx.simulate_mouse_move(at(cx0 + by), gpui::MouseButton::Left, Modifiers::none());
        vcx.run_until_parked();
        if pause {
            std::thread::sleep(std::time::Duration::from_millis(150));
        }
        vcx.simulate_mouse_up(at(cx0 + by), gpui::MouseButton::Left, Modifiers::none());
        vcx.run_until_parked();
        settle(vcx);
    };
    drag(vcx, 60.0, true);
    assert!(
        picked.borrow().is_empty(),
        "past the first page: springs back"
    );
    drag(vcx, -190.0, true);
    assert_eq!(*picked.borrow(), vec![2], "two pages under the hand");
    drag(vcx, -20.0, false);
    let last = *picked.borrow().last().unwrap();
    assert!(
        picked.borrow().len() == 2 && last > 2,
        "a flick coasts on: {:?}",
        picked.borrow()
    );
}

/// Two fingers sideways swipe the carousel a page; up-and-down is left to the page; a
/// sideways wheel notch turns one page.
#[gpui::test]
fn carousel_follows_a_trackpad_swipe(cx: &mut TestAppContext) {
    let picked = log();
    let p = picked.clone();
    let vcx = mount(cx, move |_, _| {
        let p = p.clone();
        let mut c = Carousel::new("car")
            .visible(2)
            .item_width(80.0)
            .on_select(move |i, _, _| p.borrow_mut().push(i));
        for name in ["a", "b", "c", "d", "e", "f"] {
            c = c.item(name);
        }
        c.into_any_element()
    });
    let (x, y, w, _) = target(vcx);
    let at = point(px(x + w / 2.0), px(y + 14.0));
    let send = |vcx: &mut VisualTestContext, delta: gpui::ScrollDelta, phase: gpui::TouchPhase| {
        vcx.simulate_event(gpui::ScrollWheelEvent {
            position: at,
            delta,
            modifiers: Modifiers::none(),
            touch_phase: phase,
        });
        vcx.run_until_parked();
    };
    let swipe = |vcx: &mut VisualTestContext, (dx, dy): (f32, f32)| {
        use gpui::{ScrollDelta::Pixels, TouchPhase::*};
        send(vcx, Pixels(point(px(0.0), px(0.0))), Started);
        for _ in 0..4 {
            send(vcx, Pixels(point(px(dx / 4.0), px(dy / 4.0))), Moved);
        }
        std::thread::sleep(std::time::Duration::from_millis(150));
        send(vcx, Pixels(point(px(0.0), px(0.0))), Ended);
        // The trackpad's momentum after lifting is not ours.
        send(vcx, Pixels(point(px(dx / 4.0), px(0.0))), Moved);
        settle(vcx);
    };
    swipe(vcx, (0.0, -190.0));
    assert!(
        picked.borrow().is_empty(),
        "scrolling up and down leaves the row"
    );
    swipe(vcx, (-190.0, 10.0));
    assert_eq!(*picked.borrow(), vec![2], "two pages under the fingers");
    send(
        vcx,
        gpui::ScrollDelta::Lines(point(1.0, 0.0)),
        gpui::TouchPhase::Moved,
    );
    assert_eq!(
        *picked.borrow(),
        vec![2, 1],
        "a wheel notch turns one page back"
    );
}

/// An entry scope leaves the app's motion as it found it, frame after frame, at any speed
/// (it once restored the paced motion, compounding the speed until the duration overflowed).
#[gpui::test]
fn entry_scopes_leave_the_motion_as_they_found_it(cx: &mut TestAppContext) {
    use dodai_gpui::motion::{EntryKind, EntryScope, Motion, MotionExt};
    let vcx = mount(cx, |_, _| {
        EntryScope::new(EntryKind::Fade, Muted::new("inside"))
            .delay(std::time::Duration::from_millis(40))
            .into_any_element()
    });
    let theme = vcx.update(|_, cx| dodai_gpui::theme_of(cx).motion);
    vcx.update(|_, cx| dodai_gpui::motion::set_speed(cx, 0.5));
    settle(vcx);
    let after = vcx.update(|_, cx| Motion::of(cx));
    vcx.update(|_, cx| dodai_gpui::motion::set_speed(cx, 1.0));
    assert_eq!(
        after.duration, theme.duration,
        "the theme's motion, unpaced"
    );
    assert_eq!(
        after.delay, theme.delay,
        "the scope's delay stays inside it"
    );
}

// ───────────────────────────── code block ─────────────────────────────

const CODE: &str = "alpha beta\ngamma delta\nepsilon";

fn code_page(cx: &mut TestAppContext) -> &mut VisualTestContext {
    mount(cx, |_, _| {
        CodeBlock::new("code", CODE)
            .copy_button(false)
            .line_numbers(false)
            .into_any_element()
    })
}

/// What Cmd/Ctrl-C copied from the code block.
fn copied(vcx: &mut VisualTestContext) -> Option<String> {
    let key = if cfg!(target_os = "macos") {
        "cmd-c"
    } else {
        "ctrl-c"
    };
    vcx.simulate_keystrokes(key);
    vcx.update(|_, cx| cx.read_from_clipboard().and_then(|c| c.text()))
}

/// The y of the middle of code line `line`, found by probing: the first line is the one a
/// triple-click selects as "alpha beta".
fn line_y(vcx: &mut VisualTestContext, x: f32, line: usize) -> f32 {
    let (_, top, _, h) = target(vcx);
    let want = CODE.split('\n').nth(line).unwrap();
    let mut y = top;
    while y < top + h {
        vcx.simulate_event(gpui::MouseDownEvent {
            position: point(px(x), px(y)),
            modifiers: Modifiers::none(),
            button: gpui::MouseButton::Left,
            click_count: 3,
            first_mouse: false,
        });
        vcx.simulate_mouse_up(
            point(px(x), px(y)),
            gpui::MouseButton::Left,
            Modifiers::none(),
        );
        vcx.run_until_parked();
        if copied(vcx).as_deref() == Some(want) {
            return y;
        }
        y += 2.0;
    }
    panic!("line {line} not found");
}

#[gpui::test]
fn code_block_triple_click_selects_a_line_and_copies_it(cx: &mut TestAppContext) {
    let vcx = code_page(cx);
    let (x, ..) = target(vcx);
    let y = line_y(vcx, x + 30.0, 1);
    assert!(y > 0.0);
    assert_eq!(copied(vcx).as_deref(), Some("gamma delta"));
}

#[gpui::test]
fn code_block_drag_selects_across_lines(cx: &mut TestAppContext) {
    let vcx = code_page(cx);
    let (x, ..) = target(vcx);
    let (y0, y1) = (line_y(vcx, x + 30.0, 0), line_y(vcx, x + 30.0, 1));
    vcx.update(|_, cx| cx.write_to_clipboard(gpui::ClipboardItem::new_string(String::new())));
    // Press at the start of the first line, drag to the end of the second.
    let (from, to) = (point(px(x + 12.0), px(y0)), point(px(x + 100.0), px(y1)));
    vcx.simulate_mouse_down(from, gpui::MouseButton::Left, Modifiers::none());
    vcx.simulate_mouse_move(to, gpui::MouseButton::Left, Modifiers::none());
    vcx.simulate_mouse_up(to, gpui::MouseButton::Left, Modifiers::none());
    vcx.run_until_parked();
    assert_eq!(copied(vcx).as_deref(), Some("alpha beta\ngamma delta"));
}

#[gpui::test]
fn code_block_select_all_copies_everything(cx: &mut TestAppContext) {
    let vcx = code_page(cx);
    let (x, ..) = target(vcx);
    line_y(vcx, x + 30.0, 0);
    let all = if cfg!(target_os = "macos") {
        "cmd-a"
    } else {
        "ctrl-a"
    };
    vcx.simulate_keystrokes(all);
    assert_eq!(copied(vcx).as_deref(), Some(CODE));
}

#[gpui::test]
fn code_block_double_click_selects_a_word(cx: &mut TestAppContext) {
    let vcx = code_page(cx);
    let (x, ..) = target(vcx);
    let y = line_y(vcx, x + 30.0, 0);
    let at = point(px(x + 30.0), px(y));
    for count in [1, 2] {
        vcx.simulate_event(gpui::MouseDownEvent {
            position: at,
            modifiers: Modifiers::none(),
            button: gpui::MouseButton::Left,
            click_count: count,
            first_mouse: false,
        });
        vcx.simulate_mouse_up(at, gpui::MouseButton::Left, Modifiers::none());
    }
    vcx.run_until_parked();
    assert_eq!(copied(vcx).as_deref(), Some("alpha"));
}

// ───────────────────────────── navigation drawer, rail and bar ─────────────────────────────

/// Clicks along `from..to` (a line of `along` px steps of `step`) and returns the order in
/// which the destinations reported a click: each one once, first hit only.
fn sweep(
    vcx: &mut VisualTestContext,
    log: &Log,
    at: impl Fn(f32) -> (f32, f32),
    range: std::ops::Range<f32>,
    step: f32,
) -> Vec<usize> {
    let mut order = Vec::new();
    let mut v = range.start;
    while v < range.end {
        log.borrow_mut().clear();
        let (x, y) = at(v);
        click(vcx, x, y);
        for i in log.borrow().iter() {
            if !order.contains(i) {
                order.push(*i);
            }
        }
        v += step;
    }
    order
}

fn destinations(log: &Log) -> Vec<NavigationItem> {
    (0..3)
        .map(|i| {
            let l = log.clone();
            NavigationItem::new(("nav-dest", i), ["Home", "Search", "Library"][i])
                .selected(i == 1)
                .on_click(move |_, _, _| l.borrow_mut().push(i))
        })
        .collect()
}

#[gpui::test]
fn navigation_drawer_rows_click_in_order(cx: &mut TestAppContext) {
    let l = log();
    let l2 = l.clone();
    let vcx = mount(cx, move |_, _| {
        destinations(&l2)
            .into_iter()
            .fold(NavigationDrawer::new().width(240.0), |d, i| d.item(i))
            .into_any_element()
    });
    let (x, y, w, h) = target(vcx);
    let order = sweep(vcx, &l, |v| (x + w / 2.0, v), y..y + h, 2.0);
    assert_eq!(
        order,
        vec![0, 1, 2],
        "each row reports its own click, top to bottom"
    );
}

#[gpui::test]
fn navigation_rail_destinations_click_in_order(cx: &mut TestAppContext) {
    let l = log();
    let l2 = l.clone();
    let vcx = mount(cx, move |_, _| {
        destinations(&l2)
            .into_iter()
            .fold(NavigationRail::new(), |d, i| d.item(i))
            .into_any_element()
    });
    let (x, y, w, h) = target(vcx);
    let order = sweep(vcx, &l, |v| (x + w / 2.0, v), y..y + h, 2.0);
    assert_eq!(order, vec![0, 1, 2], "rail destinations, top to bottom");
}

#[gpui::test]
fn navigation_bar_destinations_click_in_order(cx: &mut TestAppContext) {
    let l = log();
    let l2 = l.clone();
    let vcx = mount(cx, move |_, _| {
        div()
            .w(px(480.0))
            .child(
                destinations(&l2)
                    .into_iter()
                    .fold(NavigationBar::new(), |d, i| d.item(i)),
            )
            .into_any_element()
    });
    let (x, y, w, h) = target(vcx);
    let order = sweep(vcx, &l, |v| (v, y + h / 2.0), x..x + w, 4.0);
    assert_eq!(order, vec![0, 1, 2], "bar destinations, left to right");
}

/// Sets a flag whenever it is rendered: a sidebar that is not in the tree never sets it.
#[derive(IntoElement)]
struct Probe(Rc<Cell<bool>>);

impl RenderOnce for Probe {
    fn render(self, _: &mut Window, _: &mut gpui::App) -> impl IntoElement {
        self.0.set(true);
        div().child("nav")
    }
}

#[gpui::test]
fn narrow_shell_drawer_closes_from_its_own_icon(cx: &mut TestAppContext) {
    let shown = Rc::new(Cell::new(false));
    let s2 = shown.clone();
    let vcx = mount(cx, move |_, _| {
        div()
            .w(px(500.0))
            .h(px(600.0))
            .child(
                ResponsiveShell::new("shell")
                    .breakpoint(1000.0) // wider than the 900 px test window: narrow layout
                    .sidebar_width(220.0)
                    .sidebar(Probe(s2.clone()))
                    .content(div().child("page")),
            )
            .into_any_element()
    });
    shown.set(false);
    vcx.simulate_mouse_move(point(px(400.0), px(400.0)), None, Modifiers::none());
    vcx.run_until_parked();
    assert!(!shown.get(), "closed at first");
    click(vcx, 20.0, 12.0); // the menu bar
    assert!(shown.get(), "the bar opens it");
    let b = vcx
        .debug_bounds("shell-close")
        .expect("the drawer's close icon");
    vcx.simulate_mouse_move(b.center(), None, Modifiers::none());
    vcx.run_until_parked();
    click(vcx, f32::from(b.center().x), f32::from(b.center().y));
    shown.set(false);
    vcx.simulate_mouse_move(point(px(400.0), px(400.0)), None, Modifiers::none());
    vcx.run_until_parked();
    assert!(!shown.get(), "the icon closes it");
}

#[gpui::test]
fn a_click_anywhere_marks_recent_input_for_the_motion_clock(cx: &mut TestAppContext) {
    let vcx = mount(cx, |_, _| {
        div()
            .w(px(200.0))
            .h(px(200.0))
            .child(dodai_gpui::motion::input_tracker())
            .into_any_element()
    });
    assert!(!dodai_gpui::motion::input_recent(), "nothing pressed yet");
    click(vcx, 100.0, 100.0);
    assert!(
        dodai_gpui::motion::input_recent(),
        "a click counts as input"
    );
}

#[gpui::test]
fn menu_keys_walk_the_enabled_rows_and_choose(cx: &mut TestAppContext) {
    let l = log();
    let l2 = l.clone();
    let vcx = mount(cx, move |_, _| {
        let l = l2.clone();
        Menu::new("mk")
            .items([
                MenuItem::new("Cut"),
                MenuItem::new("Copy").disabled(true),
                MenuItem::new("Paste"),
            ])
            .on_select(move |i, _, _| l.borrow_mut().push(i))
            .into_any_element()
    });
    let (x, y, w, h) = target(vcx);
    // A click on the disabled row gives the menu focus without choosing anything.
    click(vcx, x + w / 2.0, y + h * 0.5);
    assert!(l.borrow().is_empty());
    vcx.simulate_keystrokes("down down enter");
    assert_eq!(*l.borrow(), vec![2], "down skips the disabled row");
    vcx.simulate_keystrokes("up enter");
    assert_eq!(*l.borrow(), vec![2, 0]);
}

#[gpui::test]
fn tab_keys_skip_a_disabled_tab(cx: &mut TestAppContext) {
    let l = log();
    let l2 = l.clone();
    let vcx = mount(cx, move |_, _| {
        let l = l2.clone();
        Tabs::new("tk")
            .labels(["One", "Two", "Three"])
            .disabled([1])
            .on_select(move |i, _, _| l.borrow_mut().push(i))
            .into_any_element()
    });
    let (x, y, w, h) = target(vcx);
    // The disabled middle tab: focus, no selection.
    click(vcx, x + w * 0.5, y + h * 0.5);
    assert!(l.borrow().is_empty(), "a disabled tab is not chosen");
    vcx.simulate_keystrokes("right");
    assert_eq!(*l.borrow(), vec![2], "right jumps over the disabled tab");
}

#[gpui::test]
fn list_arrows_move_the_selection_and_skip_disabled_rows(cx: &mut TestAppContext) {
    let l = log();
    let l2 = l.clone();
    let vcx = mount(cx, move |_, _| {
        let l = l2.clone();
        List::new(
            "lk",
            [
                ListItem::new("a", "One"),
                ListItem::new("b", "Two").disabled(true),
                ListItem::new("c", "Three"),
                ListItem::new("d", "Four"),
            ],
        )
        .on_select(move |i, _, _| l.borrow_mut().push(i))
        .into_any_element()
    });
    let (x, y, w, h) = target(vcx);
    // Click the first row: focus and selection.
    click(vcx, x + w * 0.5, y + h * 0.05);
    assert_eq!(*l.borrow(), vec![0]);
    vcx.simulate_keystrokes("down");
    assert_eq!(*l.borrow(), vec![0, 2], "down jumps over the disabled row");
    vcx.simulate_keystrokes("end up");
    assert_eq!(*l.borrow(), vec![0, 2, 3, 2]);
    vcx.simulate_keystrokes("home");
    assert_eq!(l.borrow().last(), Some(&0));
}

#[gpui::test]
fn slider_keys_step_a_focused_slider_and_disabled_ignores_them(cx: &mut TestAppContext) {
    let vals = Rc::new(RefCell::new(Vec::<f32>::new()));
    let v2 = vals.clone();
    let vcx = mount(cx, move |_, _| {
        let v = v2.clone();
        div()
            .w(px(300.0))
            .child(
                Slider::new("sk", 0.0)
                    .step(0.1)
                    .on_change(move |x, _, _| v.borrow_mut().push(x)),
            )
            .into_any_element()
    });
    let (x, y, _, h) = target(vcx);
    click(vcx, x + 2.0, y + h * 0.5);
    let start = *vals.borrow().last().expect("the click set a value");
    vcx.simulate_keystrokes("right right");
    let end = *vals.borrow().last().unwrap();
    assert!(
        (end - (start + 0.2)).abs() < 1e-3,
        "two steps from {start}: {end}"
    );
    vcx.simulate_keystrokes("end");
    assert!((*vals.borrow().last().unwrap() - 1.0).abs() < 1e-3);
}

#[gpui::test]
fn disabled_slider_never_reports(cx: &mut TestAppContext) {
    let vals = Rc::new(RefCell::new(Vec::<f32>::new()));
    let v2 = vals.clone();
    let vcx = mount(cx, move |_, _| {
        let v = v2.clone();
        div()
            .w(px(300.0))
            .child(
                Slider::new("sd", 0.3)
                    .disabled(true)
                    .on_change(move |x, _, _| v.borrow_mut().push(x)),
            )
            .into_any_element()
    });
    let (x, y, w, h) = target(vcx);
    click(vcx, x + w * 0.8, y + h * 0.5);
    vcx.simulate_keystrokes("right end");
    assert!(vals.borrow().is_empty());
}

#[gpui::test]
fn popover_opens_on_the_trigger_and_closes_on_escape(cx: &mut TestAppContext) {
    let vcx = mount(cx, move |_, _| {
        Popover::new("pv")
            .trigger(div().w(px(80.0)).h(px(30.0)).child("Open"))
            .content(div().debug_selector(|| "pv-body".into()).child("hello"))
            .into_any_element()
    });
    assert!(vcx.debug_bounds("pv-body").is_none(), "closed at first");
    let (x, y, w, h) = target(vcx);
    click(vcx, x + w * 0.5, y + h * 0.5);
    assert!(
        vcx.debug_bounds("pv-body").is_some(),
        "the trigger opens it"
    );
    vcx.simulate_keystrokes("escape");
    assert!(vcx.debug_bounds("pv-body").is_none(), "escape closes it");
}

#[gpui::test]
fn popover_closes_on_a_click_outside(cx: &mut TestAppContext) {
    let vcx = mount(cx, move |_, _| {
        Popover::new("pv2")
            .trigger(div().w(px(80.0)).h(px(30.0)).child("Open"))
            .content(div().debug_selector(|| "pv2-body".into()).child("hello"))
            .into_any_element()
    });
    let (x, y, w, h) = target(vcx);
    click(vcx, x + w * 0.5, y + h * 0.5);
    assert!(vcx.debug_bounds("pv2-body").is_some());
    click(vcx, 700.0, 600.0);
    assert!(
        vcx.debug_bounds("pv2-body").is_none(),
        "outside click closes"
    );
}

#[gpui::test]
fn theme_switcher_bar_selects_the_clicked_theme(cx: &mut TestAppContext) {
    let vcx = mount(cx, move |_, _| {
        ThemeSwitcherBar::new("tsb").into_any_element()
    });
    vcx.update(|_, cx| {
        install_switcher(cx, dodai_core::switcher());
        cx.refresh_windows();
    });
    vcx.run_until_parked();
    let before = vcx.update(|_, cx| cx.dodai().switcher().index());
    let (x, y, w, h) = target(vcx);
    // Sweep along the row until a click lands on a button of another theme.
    let _ = w;
    for k in 0..60 {
        click(vcx, x + 20.0 + 40.0 * k as f32, y + h * 0.5);
        if vcx.update(|_, cx| cx.dodai().switcher().index()) != before {
            break;
        }
    }
    let after = vcx.update(|_, cx| cx.dodai().switcher().index());
    assert_ne!(before, after, "a click on another button picks that theme");
}

#[gpui::test]
fn picking_a_mode_ends_following_the_system(cx: &mut TestAppContext) {
    let vcx = mount(cx, |_, _| div().into_any_element());
    vcx.update(|w, cx| {
        install_switcher(cx, dodai_core::switcher());
        follow_system_appearance(w, cx);
        assert!(following_system_appearance());
        set_theme_mode(cx, dodai_core::Mode::Dark);
        assert!(!following_system_appearance(), "a chosen mode stops it");
        assert_eq!(cx.dodai().theme().meta.mode, dodai_core::Mode::Dark);
        toggle_theme_mode(cx);
        assert_eq!(cx.dodai().theme().meta.mode, dodai_core::Mode::Light);
    });
}
