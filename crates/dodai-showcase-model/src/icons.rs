//! The icon catalogue of the Foundations page: the named icons every design draws in its own
//! style (by category) and the original dashboard symbols (by group), as data every toolkit
//! draws from [`IconRef::glyph`].

use dodai_core::Theme;
use dodai_icons::symbol::{Group, Symbol};
use dodai_icons::{Glyph, IconId, Pack};

/// One icon of the catalogue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IconRef {
    /// A catalogue icon (`Icon::Heart`), drawn in each design's own style.
    Named(IconId),
    /// An original symbol (`Icon::symbol(Symbol::OilCan)`), one drawing stroked in each
    /// design's weight and corners.
    Symbol(Symbol),
}

impl IconRef {
    /// Its stable kebab-case name.
    pub fn name(self) -> &'static str {
        match self {
            Self::Named(i) => i.name(),
            Self::Symbol(s) => s.name(),
        }
    }

    /// Its drawing on `theme` (`filled` where the design has a filled form; symbols have one
    /// form). `None` when the design has no such icon.
    pub fn glyph(self, theme: &Theme, filled: bool) -> Option<Glyph> {
        match self {
            Self::Named(i) => dodai_icons::glyph_for_theme(i, theme, filled),
            Self::Symbol(s) => Some(s.glyph_styled(Pack::from(theme.look().icons).symbol_style())),
        }
    }
}

/// One block of the catalogue: a category of named icons or a group of symbols.
#[derive(Debug, Clone, PartialEq)]
pub struct IconSource {
    /// Shown title ("Arrows").
    pub label: String,
    /// Its icons, in catalogue order.
    pub entries: Vec<IconRef>,
}

/// Every icon the catalogue lists.
pub fn total() -> usize {
    IconId::ALL.len() + Symbol::ALL.len()
}

fn capitalised(s: &str) -> String {
    let mut c = s.chars();
    c.next()
        .map(|f| f.to_uppercase().chain(c).collect())
        .unwrap_or_default()
}

/// Every block, sorted by label: the Phosphor categories, the shared names they leave out
/// ("Other") and the symbol groups (as the gpui showcase lists them).
pub fn sources() -> Vec<IconSource> {
    let mut named: Vec<(&'static str, Vec<IconId>)> = dodai_icons::CATEGORIES
        .iter()
        .map(|&(c, ids)| (c, ids.to_vec()))
        .collect();
    let listed: std::collections::HashSet<IconId> =
        named.iter().flat_map(|(_, v)| v.iter().copied()).collect();
    let rest = IconId::ALL
        .iter()
        .copied()
        .filter(|i| !listed.contains(i))
        .collect();
    named.push(("other", rest));
    let mut out: Vec<IconSource> = named
        .into_iter()
        .map(|(c, ids)| IconSource {
            label: capitalised(c),
            entries: ids.into_iter().map(IconRef::Named).collect(),
        })
        .collect();
    for g in Group::ALL {
        out.push(IconSource {
            label: capitalised(g.name()),
            entries: Symbol::ALL
                .iter()
                .filter(|s| s.group() == *g)
                .map(|&s| IconRef::Symbol(s))
                .collect(),
        });
    }
    out.retain(|s| !s.entries.is_empty());
    out.sort_by(|a, b| a.label.cmp(&b.label));
    out
}

/// How many characters of an icon name fit a tile.
pub const TILE_NAME_CHARS: usize = 13;

/// `name` cut to `max` characters with an ellipsis when longer (a tile's label).
pub fn shortened(name: &str, max: usize) -> String {
    if name.chars().count() <= max {
        return name.to_string();
    }
    let mut s: String = name.chars().take(max.saturating_sub(1)).collect();
    s.push('…');
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_blocks_hold_every_icon_once() {
        let n: usize = sources().iter().map(|s| s.entries.len()).sum();
        assert_eq!(n, total());
    }

    #[test]
    fn every_design_draws_the_symbols() {
        for t in dodai_core::themes::all() {
            assert!(IconRef::Symbol(Symbol::ALL[0]).glyph(&t, false).is_some());
        }
    }

    #[test]
    fn long_names_are_cut_with_an_ellipsis() {
        assert_eq!(shortened("arrow-circle-up-left", 8), "arrow-c…");
        assert_eq!(shortened("heart", 8), "heart");
    }
}
