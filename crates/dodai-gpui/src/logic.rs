//! Pure selection / paging math shared by components (no gpui context needed).

/// Move a selection index by `delta`. `wrap` wraps around, otherwise clamps.
/// `None` when `len == 0`; a missing `cur` starts before the first (delta>0) or after the last.
pub fn step_index(cur: Option<usize>, len: usize, delta: i32, wrap: bool) -> Option<usize> {
    if len == 0 {
        return None;
    }
    let n = len as i64;
    let start = match cur {
        Some(c) => c.min(len - 1) as i64,
        None if delta >= 0 => -1,
        None => n,
    };
    let next = start + delta as i64;
    Some(if wrap {
        next.rem_euclid(n) as usize
    } else {
        next.clamp(0, n - 1) as usize
    })
}

/// One slot in a pagination bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageItem {
    /// 1-based page number.
    Page(usize),
    Ellipsis,
}

/// Page buttons for `current` (1-based) of `total`, always showing first, last
/// and a window around `current`; at most `slots` (>= 5) entries.
pub fn page_window(current: usize, total: usize, slots: usize) -> Vec<PageItem> {
    let slots = slots.max(5);
    if total == 0 {
        return Vec::new();
    }
    if total <= slots {
        return (1..=total).map(PageItem::Page).collect();
    }
    let cur = current.clamp(1, total);
    let inner = slots - 2; // between first and last, includes ellipsis slots
    let mut out = vec![PageItem::Page(1)];
    let (lo, hi);
    if cur < inner {
        lo = 2;
        hi = inner;
        out.extend((lo..=hi).map(PageItem::Page));
        out.push(PageItem::Ellipsis);
    } else if cur >= total - (inner - 2) {
        out.push(PageItem::Ellipsis);
        out.extend((total - inner + 1..total).map(PageItem::Page));
    } else {
        let mid = inner - 2;
        let lo = cur - mid / 2;
        out.push(PageItem::Ellipsis);
        out.extend((lo..lo + mid).map(PageItem::Page));
        out.push(PageItem::Ellipsis);
    }
    out.push(PageItem::Page(total));
    out
}

/// Progress of one step in a stepper.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepStatus {
    Done,
    Current,
    Todo,
}

/// Status of step `i` when `current` is the active step.
pub fn step_status(i: usize, current: usize) -> StepStatus {
    match i.cmp(&current) {
        std::cmp::Ordering::Less => StepStatus::Done,
        std::cmp::Ordering::Equal => StepStatus::Current,
        std::cmp::Ordering::Greater => StepStatus::Todo,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn step_wraps_and_clamps() {
        assert_eq!(step_index(Some(2), 3, 1, true), Some(0));
        assert_eq!(step_index(Some(0), 3, -1, true), Some(2));
        assert_eq!(step_index(Some(2), 3, 1, false), Some(2));
        assert_eq!(step_index(Some(0), 3, -1, false), Some(0));
        assert_eq!(step_index(None, 3, 1, false), Some(0));
        assert_eq!(step_index(None, 3, -1, false), Some(2));
        assert_eq!(step_index(None, 0, 1, true), None);
        assert_eq!(step_index(Some(9), 3, 0, false), Some(2));
    }

    #[test]
    fn pages_small_and_large() {
        use PageItem::*;
        assert_eq!(page_window(2, 3, 7), vec![Page(1), Page(2), Page(3)]);
        let w = page_window(1, 20, 7);
        assert_eq!(w.len(), 7);
        assert_eq!(w[0], Page(1));
        assert_eq!(*w.last().unwrap(), Page(20));
        assert!(w.contains(&Ellipsis));
        for cur in 1..=20 {
            let w = page_window(cur, 20, 7);
            assert!(w.len() <= 7, "{cur}: {w:?}");
            assert!(w.contains(&Page(cur)), "{cur}: {w:?}");
            assert_eq!(w[0], Page(1));
            assert_eq!(*w.last().unwrap(), Page(20));
        }
        assert!(page_window(1, 0, 7).is_empty());
    }

    #[test]
    fn steps() {
        assert_eq!(step_status(0, 1), StepStatus::Done);
        assert_eq!(step_status(1, 1), StepStatus::Current);
        assert_eq!(step_status(2, 1), StepStatus::Todo);
    }
}
