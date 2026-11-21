#!/usr/bin/env nu
# Tests for scripts/lib/components.nu.

use std/assert
use runner.nu *
use ../lib/components.nu [pascal render]

def "test components: pascal case" [] {
    assert equal (pascal "tag") "Tag"
    assert equal (pascal "data_table") "DataTable"
}

def "test components: optional colours fall back to transparent" [] {
    let spec = { component: [{ name: "tag", doc: "d", resolver: "tag", args: [{ name: "solid", ty: "bool" }], fields: [{ name: "border", kind: "opt_color" }, { name: "text", kind: "color" }] }] }
    let t = { color_ty: "C", convert: "conv", transparent: "C::NONE" }
    let out = (render $spec $t)
    assert ($out | str contains "pub struct TagStyle")
    assert ($out | str contains "border: tok.border.map_or(C::NONE, conv),")
    assert ($out | str contains "text: conv(tok.text),")
    assert ($out | str contains "pub fn of(theme: &Theme, solid: bool)")
    assert ($out | str contains "theme.tag(solid)")
}

def "test components: a target without transparent keeps options" [] {
    let spec = { component: [{ name: "tag", doc: "d", resolver: "tag", args: [], fields: [{ name: "border", kind: "opt_color" }] }] }
    let out = (render $spec { color_ty: "C", convert: "conv" })
    assert ($out | str contains "pub border: Option<C>,")
    assert ($out | str contains "border: tok.border.map(conv),")
}
