//! Builders named after the spec (`dist/design/spec`): the same component, property and event
//! names as every other toolkit. Each builder turns into a bundle; the free functions
//! (`badge`, `avatar`, ...) stay for existing callers.

use bevy::prelude::*;
use dodai_core::Tone;

use crate::components::{alert, avatar, badge, breadcrumb};
use crate::overlay::accordion;

/// A count or short label in a tone.
pub struct Badge {
    text: String,
    tone: Tone,
}

impl Badge {
    /// A badge showing `text` in `tone`.
    pub fn new(text: impl Into<String>, tone: Tone) -> Self {
        Self {
            text: text.into(),
            tone,
        }
    }
    /// The bundle to spawn.
    pub fn bundle(self) -> impl Bundle {
        badge(self.text, self.tone)
    }
}

/// A round avatar with initials.
pub struct Avatar {
    initials: String,
    size: f32,
}

impl Avatar {
    /// An avatar showing `initials`.
    pub fn new(initials: impl Into<String>) -> Self {
        Self {
            initials: initials.into(),
            size: 32.0,
        }
    }
    /// Size in px.
    #[must_use]
    pub fn size(mut self, s: f32) -> Self {
        self.size = s;
        self
    }
    /// The bundle to spawn.
    pub fn bundle(self) -> impl Bundle {
        avatar(self.initials, self.size)
    }
}

/// A trail of places, the last one current.
pub struct Breadcrumb {
    items: Vec<String>,
}

impl Breadcrumb {
    /// A breadcrumb over `items`.
    pub fn new<S: Into<String>>(items: impl IntoIterator<Item = S>) -> Self {
        Self {
            items: items.into_iter().map(Into::into).collect(),
        }
    }
    /// The bundle to spawn.
    pub fn bundle(self) -> impl Bundle {
        let items: Vec<&str> = self.items.iter().map(String::as_str).collect();
        breadcrumb(&items)
    }
}

/// Sections that fold: a header over a body.
#[derive(Default)]
pub struct Accordion {
    items: Vec<(String, String)>,
    expanded: Option<usize>,
}

impl Accordion {
    /// An empty accordion.
    pub fn new() -> Self {
        Self::default()
    }
    /// Add an item.
    #[must_use]
    pub fn item(mut self, title: impl Into<String>, body: impl Into<String>) -> Self {
        self.items.push((title.into(), body.into()));
        self
    }
    /// The item shown open at the start.
    #[must_use]
    pub fn expanded(mut self, open: usize) -> Self {
        self.expanded = Some(open);
        self
    }
    /// The bundle to spawn.
    pub fn bundle(self) -> impl Bundle {
        accordion(self.items, self.expanded)
    }
}

/// A message in a tone.
pub struct Alert {
    tone: Tone,
    message: String,
}

impl Alert {
    /// An alert of `tone`.
    pub fn new(tone: Tone) -> Self {
        Self {
            tone,
            message: String::new(),
        }
    }
    /// Message text.
    #[must_use]
    pub fn message(mut self, m: impl Into<String>) -> Self {
        self.message = m.into();
        self
    }
    /// The bundle to spawn.
    pub fn bundle(self) -> impl Bundle {
        alert(self.tone, self.message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builders_make_bundles() {
        fn is_bundle(_: impl Bundle) {}
        is_bundle(Badge::new("3", Tone::Info).bundle());
        is_bundle(Avatar::new("AB").size(40.0).bundle());
        is_bundle(Breadcrumb::new(["Home", "Docs"]).bundle());
        is_bundle(Accordion::new().item("a", "b").expanded(0).bundle());
        is_bundle(Alert::new(Tone::Warning).message("Low").bundle());
    }
}
