//! Gantt layout: tasks to rows and horizontal spans.

#[derive(Debug, Clone, PartialEq)]
pub struct Task {
    pub name: String,
    pub start: f32,
    pub end: f32,
    /// `0..=1`
    pub progress: f32,
    /// Indices of tasks this one depends on.
    pub deps: Vec<usize>,
}

impl Task {
    pub fn new(name: impl Into<String>, start: f32, end: f32) -> Self {
        Self {
            name: name.into(),
            start,
            end,
            progress: 0.0,
            deps: Vec::new(),
        }
    }
    pub fn progress(mut self, p: f32) -> Self {
        self.progress = p.clamp(0.0, 1.0);
        self
    }
    pub fn after(mut self, i: usize) -> Self {
        self.deps.push(i);
        self
    }
}

/// Laid-out bar, in fractions `0..=1` of the full time span.
#[derive(Debug, Clone, PartialEq)]
pub struct Bar {
    pub row: usize,
    pub x: f32,
    pub w: f32,
    /// Width of the completed part, as a fraction of the whole span.
    pub done: f32,
}

/// Layout result: bars, plus dependency links `(from_bar, to_bar)`.
#[derive(Debug, Clone, PartialEq)]
pub struct Layout {
    pub bars: Vec<Bar>,
    pub links: Vec<(usize, usize)>,
    pub span: (f32, f32),
}

/// One row per task, x/width normalised to the overall span. Pure.
pub fn layout(tasks: &[Task]) -> Layout {
    let lo = tasks.iter().map(|t| t.start).fold(f32::INFINITY, f32::min);
    let hi = tasks
        .iter()
        .map(|t| t.end)
        .fold(f32::NEG_INFINITY, f32::max);
    if tasks.is_empty() || hi <= lo {
        return Layout {
            bars: Vec::new(),
            links: Vec::new(),
            span: (0.0, 0.0),
        };
    }
    let total = hi - lo;
    let bars = tasks
        .iter()
        .enumerate()
        .map(|(row, t)| {
            let w = (t.end - t.start).max(0.0) / total;
            Bar {
                row,
                x: (t.start - lo) / total,
                w,
                done: w * t.progress.clamp(0.0, 1.0),
            }
        })
        .collect();
    let links = tasks
        .iter()
        .enumerate()
        .flat_map(|(i, t)| {
            t.deps
                .iter()
                .filter(|d| **d < tasks.len())
                .map(move |d| (*d, i))
        })
        .collect();
    Layout {
        bars,
        links,
        span: (lo, hi),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalises_to_span() {
        let l = layout(&[
            Task::new("a", 10.0, 20.0).progress(0.5),
            Task::new("b", 20.0, 40.0).after(0),
        ]);
        assert_eq!(l.span, (10.0, 40.0));
        assert!((l.bars[0].w - 1.0 / 3.0).abs() < 1e-5);
        assert!((l.bars[0].done - 1.0 / 6.0).abs() < 1e-5);
        assert_eq!(l.bars[1].x, 1.0 / 3.0);
        assert_eq!(l.links, vec![(0, 1)]);
    }

    #[test]
    fn empty_and_bad_deps() {
        assert!(layout(&[]).bars.is_empty());
        let l = layout(&[Task::new("a", 0.0, 1.0).after(9)]);
        assert!(l.links.is_empty());
    }
}
