//! Machine-readable description of a component's tokens, produced by
//! `#[derive(TokenSpec)]` (see the `dodai-ui-spec` crate) on the token structs. The `dodai-ui-gen`
//! tool reads [`all`] to write per-toolkit glue, so the spec lives next to the code it
//! describes and the compiler checks it.

/// What a token field holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldKind {
    /// A colour that is always present.
    Color,
    /// A colour that may be absent.
    OptColor,
}

/// One field of a token struct.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Field {
    /// Field name.
    pub name: &'static str,
    /// What it holds.
    pub kind: FieldKind,
}

/// One argument of a resolver, with its type as written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Arg {
    /// Argument name.
    pub name: &'static str,
    /// Type as written, e.g. `Tone` or `bool`.
    pub ty: &'static str,
}

/// How a component's tokens are resolved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComponentSpec {
    /// Component name (the resolver's name).
    pub name: &'static str,
    /// One-line description.
    pub doc: &'static str,
    /// The `Theme` method that returns the tokens.
    pub resolver: &'static str,
    /// The token struct's name.
    pub tokens: &'static str,
    /// Arguments of the resolver after `&self`.
    pub args: &'static [Arg],
    /// Fields of the token struct.
    pub fields: &'static [Field],
}

/// Implemented by `#[derive(TokenSpec)]`.
pub trait TokenSpec {
    /// The spec of this token struct.
    const SPEC: ComponentSpec;
}

/// Every component that has a spec, in a stable order.
pub fn all() -> Vec<ComponentSpec> {
    vec![crate::TagTokens::SPEC, crate::PanelTokens::SPEC]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tag_spec_is_read_from_the_struct() {
        let s = crate::TagTokens::SPEC;
        assert_eq!((s.resolver, s.tokens), ("tag", "TagTokens"));
        assert_eq!(
            s.args.iter().map(|a| a.ty).collect::<Vec<_>>(),
            ["Tone", "bool"]
        );
        assert_eq!(s.fields[0].kind, FieldKind::OptColor);
        assert_eq!(s.fields[1].kind, FieldKind::Color);
    }

    #[test]
    fn every_spec_has_fields_and_a_unique_name() {
        let all = all();
        assert!(all.iter().all(|s| !s.fields.is_empty()));
        let mut names: Vec<_> = all.iter().map(|s| s.name).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), all.len());
    }
}
