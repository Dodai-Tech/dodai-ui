//! A popup action list as data, and the keyboard rule for it, shared by every toolkit's `Menu`:
//! Up and Down move the highlight past separators and disabled rows, Home and End jump to the
//! ends, Enter and Space pick the highlighted row. The toolkit draws the rows and maps its key
//! events to [`step`].

use crate::roving::{self, Axis};

/// One row of a menu.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MenuRow {
    /// Label; empty for a separator.
    pub label: String,
    /// Right-aligned key hint, e.g. `⌘C`.
    pub shortcut: Option<String>,
    /// Not pickable.
    pub disabled: bool,
    /// A destructive action.
    pub danger: bool,
    /// Shows a check before the label.
    pub checked: bool,
    /// A rule between groups (the other fields are ignored).
    pub separator: bool,
}

impl MenuRow {
    /// A plain row.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            ..Self::default()
        }
    }

    /// A rule between groups.
    pub fn separator() -> Self {
        Self {
            separator: true,
            ..Self::default()
        }
    }

    /// The key hint.
    pub fn shortcut(mut self, s: impl Into<String>) -> Self {
        self.shortcut = Some(s.into());
        self
    }

    /// Not pickable.
    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }

    /// A destructive action.
    pub fn danger(mut self) -> Self {
        self.danger = true;
        self
    }

    /// Shows a check.
    pub fn checked(mut self) -> Self {
        self.checked = true;
        self
    }

    /// Whether the row can be highlighted and picked.
    pub fn pickable(&self) -> bool {
        !self.separator && !self.disabled
    }
}

/// What a key does to a menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// Move the highlight to this row.
    Move(usize),
    /// Pick this row.
    Pick(usize),
}

/// The first pickable row, where the highlight starts.
pub fn first(rows: &[MenuRow]) -> Option<usize> {
    rows.iter().position(MenuRow::pickable)
}

/// What `key` (a web or native key name) does with `active` highlighted. `None`: nothing.
pub fn step(key: &str, rows: &[MenuRow], active: usize) -> Option<Step> {
    if matches!(key, "Enter" | " " | "Space" | "enter" | "space") {
        return rows
            .get(active)
            .is_some_and(MenuRow::pickable)
            .then_some(Step::Pick(active));
    }
    let name = roving::key_name(key)?;
    roving::next(name, Axis::Vertical, active, rows.len(), |i| {
        rows[i].pickable()
    })
    .map(Step::Move)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rows() -> Vec<MenuRow> {
        vec![
            MenuRow::new("Copy"),
            MenuRow::separator(),
            MenuRow::new("Paste").disabled(),
            MenuRow::new("Delete").danger(),
        ]
    }

    #[test]
    fn highlight_starts_on_the_first_pickable_row() {
        assert_eq!(first(&rows()), Some(0));
        assert_eq!(first(&[MenuRow::separator()]), None);
    }

    #[test]
    fn arrows_skip_separators_and_disabled_rows() {
        let r = rows();
        assert_eq!(step("ArrowDown", &r, 0), Some(Step::Move(3)));
        assert_eq!(step("ArrowUp", &r, 3), Some(Step::Move(0)));
        assert_eq!(step("ArrowUp", &r, 0), None);
        assert_eq!(step("End", &r, 0), Some(Step::Move(3)));
        assert_eq!(step("Home", &r, 3), Some(Step::Move(0)));
    }

    #[test]
    fn enter_and_space_pick_only_a_pickable_row() {
        let r = rows();
        assert_eq!(step("Enter", &r, 3), Some(Step::Pick(3)));
        assert_eq!(step(" ", &r, 0), Some(Step::Pick(0)));
        assert_eq!(step("Enter", &r, 2), None);
        assert_eq!(step("Enter", &r, 9), None);
        assert_eq!(step("x", &r, 0), None);
    }
}
