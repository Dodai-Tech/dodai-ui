//! Placing panels on the grid, and reading order for narrow screens.

use crate::spec::{Panel, Span, GRID_COLUMNS};

/// The span of every panel, in panel order. Panels with a span keep it. The others are placed
/// automatically, left to right and wrapping at [`GRID_COLUMNS`], below everything that has a
/// span, each at its kind's default size.
pub fn place(panels: &[Panel]) -> Vec<Span> {
    let floor = panels
        .iter()
        .filter_map(|p| p.span)
        .map(|s| s.bottom())
        .max()
        .unwrap_or(0);
    let (mut x, mut y, mut row_h) = (0u32, floor, 0u32);
    panels
        .iter()
        .map(|p| {
            if let Some(s) = p.span {
                return s;
            }
            let (w, h) = p.kind.default_size();
            let w = w.min(GRID_COLUMNS);
            if x + w > GRID_COLUMNS {
                x = 0;
                y += row_h;
                row_h = 0;
            }
            let s = Span { x, y, w, h };
            x += w;
            row_h = row_h.max(h);
            s
        })
        .collect()
}

/// Panel indices in reading order (top to bottom, then left to right). A narrow screen shows the
/// panels one under another in this order.
pub fn reading_order(spans: &[Span]) -> Vec<usize> {
    let mut order: Vec<usize> = (0..spans.len()).collect();
    order.sort_by_key(|&i| (spans[i].y, spans[i].x, i));
    order
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::PanelKind;

    fn panel(kind: PanelKind, span: Option<Span>) -> Panel {
        let mut p = Panel::new("t", kind, "m");
        p.span = span;
        p
    }

    #[test]
    fn explicit_spans_are_kept() {
        let s = Span {
            x: 2,
            y: 1,
            w: 4,
            h: 2,
        };
        assert_eq!(place(&[panel(PanelKind::Line, Some(s))]), vec![s]);
    }

    #[test]
    fn automatic_panels_flow_and_wrap() {
        let panels = [
            panel(PanelKind::Line, None),  // 6x3
            panel(PanelKind::Line, None),  // 6x3, fills the row
            panel(PanelKind::Stat, None),  // 3x2, wraps
            panel(PanelKind::Gauge, None), // 3x3, beside it
        ];
        let spans = place(&panels);
        assert_eq!(
            spans[0],
            Span {
                x: 0,
                y: 0,
                w: 6,
                h: 3
            }
        );
        assert_eq!(
            spans[1],
            Span {
                x: 6,
                y: 0,
                w: 6,
                h: 3
            }
        );
        assert_eq!(
            spans[2],
            Span {
                x: 0,
                y: 3,
                w: 3,
                h: 2
            }
        );
        assert_eq!(
            spans[3],
            Span {
                x: 3,
                y: 3,
                w: 3,
                h: 3
            }
        );
    }

    #[test]
    fn automatic_panels_go_below_explicit_ones() {
        let panels = [
            panel(
                PanelKind::Line,
                Some(Span {
                    x: 0,
                    y: 0,
                    w: 12,
                    h: 5,
                }),
            ),
            panel(PanelKind::Stat, None),
        ];
        assert_eq!(place(&panels)[1].y, 5);
    }

    #[test]
    fn reading_order_is_rows_then_columns() {
        let spans = [
            Span {
                x: 6,
                y: 0,
                w: 6,
                h: 3,
            },
            Span {
                x: 0,
                y: 3,
                w: 12,
                h: 2,
            },
            Span {
                x: 0,
                y: 0,
                w: 6,
                h: 3,
            },
        ];
        assert_eq!(reading_order(&spans), vec![2, 0, 1]);
    }
}
