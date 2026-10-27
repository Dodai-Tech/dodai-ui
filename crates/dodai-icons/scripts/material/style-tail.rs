
/// The outline form of a catalogue icon.
pub fn get(sym: super::MaterialSymbol) -> Option<&'static Icon> {
    super::find(ALL, sym.name())
}

/// The filled form of a catalogue icon.
pub fn get_fill(sym: super::MaterialSymbol) -> Option<&'static Icon> {
    super::find_fill(ALL, sym.name())
}

/// Material's drawing of a shared-vocabulary icon (`filled` falls back to the outline when the
/// icon has no filled form).
pub fn get_id(id: crate::IconId, filled: bool) -> Option<&'static Icon> {
    super::lookup(ALL, id, filled)
}
