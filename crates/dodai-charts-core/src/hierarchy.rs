//! Hierarchies for sunburst and icicle charts: a tree split into nested spans. Pure.

/// A node of a hierarchy. A leaf carries its value; a branch is the sum of its children.
#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    /// The label shown for this node.
    pub label: String,
    /// The leaf value (ignored on branches).
    pub value: f32,
    /// Children; empty for a leaf.
    pub children: Vec<Node>,
}

impl Node {
    /// A leaf with a value.
    pub fn leaf(label: impl Into<String>, value: f32) -> Self {
        Self {
            label: label.into(),
            value,
            children: Vec::new(),
        }
    }
    /// A branch whose value is the sum of its children.
    pub fn branch(label: impl Into<String>, children: impl IntoIterator<Item = Node>) -> Self {
        Self {
            label: label.into(),
            value: 0.0,
            children: children.into_iter().collect(),
        }
    }
    /// The node's size: its value for a leaf (non-finite or negative counts as 0), else the
    /// sum of its children.
    pub fn total(&self) -> f32 {
        if self.children.is_empty() {
            if self.value.is_finite() {
                self.value.max(0.0)
            } else {
                0.0
            }
        } else {
            self.children.iter().map(Node::total).sum()
        }
    }
    /// Levels below this node (0 for a leaf).
    pub fn depth(&self) -> usize {
        self.children
            .iter()
            .map(|c| c.depth() + 1)
            .max()
            .unwrap_or(0)
    }
}

/// One node's span in a partition.
#[derive(Clone, Debug, PartialEq)]
pub struct Span {
    /// 0 for the root's children, 1 for theirs, and so on.
    pub depth: usize,
    /// Start of the span, `0..=1` of the whole.
    pub start: f32,
    /// End of the span, `0..=1` of the whole.
    pub end: f32,
    /// The node's total.
    pub value: f32,
    /// The node's label.
    pub label: String,
    /// Which of the root's children this span descends from (for colour).
    pub branch: usize,
    /// Index of the parent span, `None` at depth 0.
    pub parent: Option<usize>,
}

/// Every node below `root` as a span of the unit interval, parents before children: each
/// child takes its share of its parent's span. Zero-sized nodes are left out. Pure.
pub fn partition(root: &Node) -> Vec<Span> {
    fn walk(
        n: &Node,
        start: f32,
        width: f32,
        depth: usize,
        branch: Option<usize>,
        parent: Option<usize>,
        out: &mut Vec<Span>,
    ) {
        let total = n.total();
        if total <= 0.0 {
            return;
        }
        let mut at = start;
        for (i, c) in n.children.iter().enumerate() {
            let v = c.total();
            if v <= 0.0 {
                continue;
            }
            let w = width * v / total;
            let me = out.len();
            out.push(Span {
                depth,
                start: at,
                end: at + w,
                value: v,
                label: c.label.clone(),
                branch: branch.unwrap_or(i),
                parent,
            });
            walk(
                c,
                at,
                w,
                depth + 1,
                Some(branch.unwrap_or(i)),
                Some(me),
                out,
            );
            at += w;
        }
    }
    let mut out = Vec::new();
    walk(root, 0.0, 1.0, 0, None, None, &mut out);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tree() -> Node {
        Node::branch(
            "all",
            [
                Node::branch("a", [Node::leaf("a1", 1.0), Node::leaf("a2", 3.0)]),
                Node::leaf("b", 4.0),
                Node::leaf("gone", f32::NAN),
            ],
        )
    }

    #[test]
    fn children_split_their_parent_by_value() {
        let p = partition(&tree());
        let names: Vec<_> = p.iter().map(|s| s.label.as_str()).collect();
        assert_eq!(names, ["a", "a1", "a2", "b"]);
        assert_eq!((p[0].start, p[0].end), (0.0, 0.5));
        assert_eq!((p[1].start, p[1].end), (0.0, 0.125));
        assert_eq!((p[2].start, p[2].end), (0.125, 0.5));
        assert_eq!((p[3].start, p[3].end, p[3].branch), (0.5, 1.0, 1));
        assert_eq!((p[2].depth, p[2].parent, p[2].branch), (1, Some(0), 0));
        assert_eq!(tree().depth(), 2);
    }

    #[test]
    fn an_empty_tree_has_no_spans() {
        assert!(partition(&Node::branch("x", [Node::leaf("z", 0.0)])).is_empty());
        assert!(partition(&Node::leaf("x", 5.0)).is_empty());
    }
}
