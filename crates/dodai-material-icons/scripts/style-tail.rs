
/// The outline form of a catalogue icon.
pub fn get(sym: crate::MaterialSymbol) -> Option<&'static Icon> {
    crate::find(ALL, sym.name())
}

/// The filled form of a catalogue icon.
pub fn get_fill(sym: crate::MaterialSymbol) -> Option<&'static Icon> {
    crate::find_fill(ALL, sym.name())
}

/// Material's drawing of a shared-vocabulary icon (`filled` falls back to the outline when the
/// icon has no filled form).
pub fn get_id(id: dodai_icon_id::IconId, filled: bool) -> Option<&'static Icon> {
    crate::lookup(ALL, id, filled)
}
