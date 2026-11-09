//! Menus built from plain nodes, laid out like the gpui `Menu`: an icon slot, the label, a
//! shortcut and a sub-menu mark per row, separators between groups, danger and disabled rows.

use dodai_core::Tone;
use dodai_icons::IconId;

use crate::b::*;
use crate::catalog::Ctx;
use crate::node::{Axis, Font, Gap, Ink, Node, Stack, TextRole, TextStyle};

/// One row of a menu.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Item {
    /// The label.
    pub label: String,
    /// Leading icon.
    pub icon: Option<IconId>,
    /// Shortcut text at the end.
    pub shortcut: Option<String>,
    /// Opens a sub-menu (a mark at the end).
    pub sub: bool,
    /// A destructive action (danger colour).
    pub danger: bool,
    /// Greyed out.
    pub disabled: bool,
}

/// A menu row or a separator.
#[derive(Debug, Clone, PartialEq)]
pub enum Entry {
    /// A row.
    Item(Item),
    /// A line between groups.
    Separator,
}

/// A row labelled `label`.
pub fn item(label: &str) -> Item {
    Item {
        label: label.into(),
        ..Item::default()
    }
}

impl Item {
    /// Leading icon.
    pub fn icon(mut self, i: IconId) -> Self {
        self.icon = Some(i);
        self
    }
    /// Shortcut text.
    pub fn shortcut(mut self, s: &str) -> Self {
        self.shortcut = Some(s.into());
        self
    }
    /// Opens a sub-menu.
    pub fn sub(mut self) -> Self {
        self.sub = true;
        self
    }
    /// Destructive.
    pub fn danger(mut self) -> Self {
        self.danger = true;
        self
    }
    /// Greyed out.
    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }
    /// As a menu entry.
    pub fn entry(self) -> Entry {
        Entry::Item(self)
    }
}

/// A floating menu `width` px wide holding `entries`.
pub fn menu(c: &Ctx, width: f32, entries: Vec<Entry>) -> Node {
    Node::Popup(Box::new(Node::Stack(Stack {
        axis: Axis::Column,
        wrap: false,
        gap: Gap::Xs,
        centre: false,
        width: Some(width),
        items: entries.into_iter().map(|e| row(c, e)).collect(),
    })))
}

fn row(c: &Ctx, e: Entry) -> Node {
    let it = match e {
        Entry::Separator => return Node::Divider,
        Entry::Item(it) => it,
    };
    let ty = &c.theme.typography;
    let ink = if it.disabled {
        Ink::Disabled
    } else if it.danger {
        Ink::Tone(Tone::Danger)
    } else {
        Ink::Text
    };
    let slot = c.theme.sizes().icon_sm + c.theme.spacing.half();
    let mut items = vec![match it.icon {
        Some(i) => Node::Icon(i, c.theme.sizes().icon_sm),
        None => Node::Space(slot * 0.7),
    }];
    items.push(Node::Text(
        it.label,
        TextRole::Styled(TextStyle {
            size: ty.size_md,
            font: Font::Body,
            ink,
            caps: ty.uppercase_labels,
        }),
    ));
    items.push(Node::Grow);
    if let Some(s) = it.shortcut {
        items.push(muted(s));
    }
    if it.sub {
        items.push(Node::Icon(IconId::ChevronRight, c.theme.sizes().icon_sm));
    }
    line(items)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_and_separators_keep_their_order() {
        let theme = dodai_core::themes::all()[0].clone();
        let values = crate::Values::default();
        let c = Ctx {
            theme: &theme,
            values: &values,
        };
        let m = menu(
            &c,
            240.0,
            vec![
                item("Edit").icon(IconId::Edit).shortcut("⌘E").entry(),
                Entry::Separator,
                item("Delete").danger().entry(),
            ],
        );
        let Node::Popup(body) = m else {
            panic!("a popup")
        };
        let Node::Stack(s) = *body else {
            panic!("a column")
        };
        assert_eq!(s.width, Some(240.0));
        assert_eq!(s.items.len(), 3);
        assert_eq!(s.items[1], Node::Divider);
    }
}
