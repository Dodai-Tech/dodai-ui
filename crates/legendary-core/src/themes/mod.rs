//! Built-in themes. Add new families as sibling modules and list them in [`all`].

pub mod cyberpunk;

use crate::Theme;

/// Every built-in theme, in switcher order.
pub fn all() -> Vec<Theme> {
    let mut v = Vec::new();
    v.extend(cyberpunk::variants());
    v
}

/// Look up a built-in theme by id.
pub fn by_id(id: &str) -> Option<Theme> {
    all().into_iter().find(|t| t.id() == id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn ids_unique() {
        let ids: HashSet<_> = all().iter().map(|t| t.id()).collect();
        assert_eq!(ids.len(), all().len());
    }

    #[test]
    fn text_is_readable_everywhere() {
        for t in all() {
            let p = &t.palette;
            assert!(p.text.contrast(p.background) >= 7.0, "{}: text/bg", t.id());
            assert!(
                p.text.contrast(p.surface) >= 7.0,
                "{}: text/surface",
                t.id()
            );
            assert!(
                p.on_primary.contrast(p.primary) >= 4.5,
                "{}: on_primary",
                t.id()
            );
            assert!(
                p.on_secondary.contrast(p.secondary) >= 7.0,
                "{}: on_secondary",
                t.id()
            );
            assert!(
                p.text_muted.contrast(p.background) >= 4.5,
                "{}: muted",
                t.id()
            );
        }
    }

    #[test]
    fn lookup() {
        assert!(by_id("cyberpunk").is_some());
        assert!(by_id("missing").is_none());
    }
}
