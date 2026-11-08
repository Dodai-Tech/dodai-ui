//! iced (feature `iced`): a shader as an image widget.
//!
//! iced has no frame callback for widgets, so the app drives it: call [`ShaderView::tick`] from
//! a timer subscription every [`ShaderView::next_in`] and show [`ShaderView::view`].
//!
//! ```ignore
//! let mut shader = ShaderView::new(Spec::new("neon_grid").fx("crt"));
//! // on a tick message: shader.tick(width, 1.0);
//! // in view():        shader.view()
//! ```

use crate::player::{Live, Poll, Spec};
use iced::widget::image::Handle;
use iced::{Element, Length};

/// A playing shader.
pub struct ShaderView {
    live: Live,
    handle: Option<Handle>,
}

impl ShaderView {
    /// Nothing shows until the first [`tick`](Self::tick) produces a picture.
    pub fn new(spec: Spec) -> Self {
        Self {
            live: Live::new(spec),
            handle: None,
        }
    }

    /// What is playing.
    pub fn spec(&self) -> &Spec {
        self.live.spec()
    }

    /// Replace the spec (nothing happens if it is unchanged).
    pub fn set_spec(&mut self, spec: Spec) {
        self.live.set_spec(spec);
    }

    /// How long to wait between ticks (`None` for a still shader).
    pub fn next_in(&self) -> Option<std::time::Duration> {
        self.live.next_in()
    }

    /// Render a picture `width` logical px wide if one is due; true when the picture changed.
    pub fn tick(&mut self, width: f32, scale: f32) -> bool {
        match self.live.tick(width, scale) {
            Poll::New(f) => {
                let (w, h) = (f.width, f.height);
                self.handle = Some(Handle::from_rgba(w, h, f.rgba()));
                true
            }
            Poll::Pending | Poll::Keep => false,
        }
    }

    /// The picture, fluid width (empty until the first one).
    pub fn view<'a, M: 'a>(&'a self) -> Element<'a, M> {
        match &self.handle {
            Some(h) => iced::widget::image(h.clone()).width(Length::Fill).into(),
            None => iced::widget::Space::new().into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_view_gets_its_first_picture() {
        let mut v = ShaderView::new(Spec::new("neon_grid").animated(false));
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
        while !v.tick(64.0, 1.0) {
            assert!(std::time::Instant::now() < deadline, "no picture in 60 s");
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        assert!(v.handle.is_some());
        assert!(v.next_in().is_none());
    }
}
