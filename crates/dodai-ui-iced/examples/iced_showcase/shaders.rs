//! Shader nodes: one cached [`ShaderView`] per distinct spec.
//!
//! `view` runs on the main thread only, so a thread-local cache is enough. A view the page no
//! longer asks for is dropped at the next tick.

use std::cell::RefCell;
use std::collections::HashMap;

use dodai_ui_shaders::iced::ShaderView;
use dodai_ui_showcase_model::ShaderSpec;
use iced::Element;

struct Entry {
    view: ShaderView,
    width: f32,
    wanted: bool,
}

thread_local! {
    static CACHE: RefCell<HashMap<String, Entry>> = RefCell::new(HashMap::new());
}

/// The picture for `spec` (blank until its first frame is ready).
pub fn view<'a, M: 'a>(spec: &ShaderSpec, theme: &dodai_ui_core::Theme) -> Element<'a, M> {
    let key = format!("{spec:?}{}", theme.name());
    CACHE.with(|c| {
        let mut c = c.borrow_mut();
        let e = c.entry(key).or_insert_with(|| Entry {
            view: ShaderView::new(spec.to_spec(theme)),
            width: spec.width,
            wanted: true,
        });
        e.wanted = true;
        let picture = e.view.view();
        iced::widget::container(picture).width(spec.width).into()
    })
}

/// Once per frame: forget unused views, give the rest the chance to draw.
pub fn tick() {
    CACHE.with(|c| {
        let mut c = c.borrow_mut();
        c.retain(|_, e| e.wanted);
        for e in c.values_mut() {
            e.wanted = false;
            e.view.tick(e.width, 1.0);
        }
    });
}
