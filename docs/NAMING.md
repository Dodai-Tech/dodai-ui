# Naming

One spec (`dist/design/spec/*.md`), one vocabulary. Every toolkit exposes the same public names, spelled
the way its language spells them. Internals are free.

## Components
- PascalCase noun, the spec's name: `Badge`, `DataTable`, `SegmentedControl`. No toolkit prefix
  (`Ui`, `Dodai`, `L`) on the public name; the crate path is the namespace.
- Variants of one component are a property (`variant`), not a new component (`ButtonDisabled` is wrong).

## Properties
- snake_case in Rust, camelCase in JS/CSS-in-props, kebab-case for CSS classes and attributes
  (`cut_gap` / `cutGap` / `cut-gap`). Same words, same order, in every toolkit.
- Boolean flags are adjectives, not questions: `disabled`, `elevated`, `dismissible`
  (never `is_disabled`, `set_disabled`, `with_disabled`).
- Value properties are nouns: `size`, `tone`, `variant`, `width`, `height`.
- Collections add one entry per call with the singular: `item`, `tab`, `column`.
- A flag takes an optional bool where the language allows (`disabled(true)`); a bare call means true.

## Events
- Spec name is the past-tense-free noun or verb the user did: `click`, `change`, `select`, `dismiss`,
  `toggle`, `submit`.
- Rust builders prefix `on_` (`on_select`); other toolkits use their own handler convention
  (`onSelect`, `on-select`, a message variant `Select`). The stem is always the spec's.

## Constructors
- `new(content...)` takes exactly the spec's *Content* items, in order, and nothing else.
- Everything else is a property method. No positional "options" arguments.
- A component with a bundle/element conversion ends the chain with the toolkit's own verb
  (`bundle()`, `into_element()`, `view()`); that verb is not part of the spec.

## Enums and values
- Variant names are PascalCase and identical to the token names (`Tone::Warning`, `ButtonVariant::Outlined`).

## Checking
`nu scripts/design.nu conform <toolkit>` lists the spec names a toolkit lacks. Deviations are bugs, not
styles: rename the toolkit, or change the spec for everyone.
