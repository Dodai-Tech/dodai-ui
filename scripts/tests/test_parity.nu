#!/usr/bin/env nu
# Tests for scripts/lib/parity.nu.

use std/assert
use runner.nu *
use ../lib/parity.nu [node-variants matched-variants component-names snake exposes]

def "test parity: node variants are read from the enum" [] {
    let src = "pub enum Node {\n    /// doc\n    Stack(Stack),\n    Text { t: String },\n    Space,\n}\n"
    assert equal (node-variants $src) [Stack Text Space]
}

def "test parity: matched variants need the whole name" [] {
    let src = "Demo::Button { .. } => x, Demo::ButtonGroup => y"
    assert equal (matched-variants $src [Button Group ButtonGroup]) [Button ButtonGroup]
}

def "test parity: component names skip plumbing" [] {
    let src = "pub struct Button {\n}\npub struct ButtonState {\n}\npub struct Tag;\npub struct Tag2 {\n}\n"
    assert equal (component-names $src) [Button Tag Tag2]
}

def "test parity: snake case" [] {
    assert equal (snake "DataTable") "data_table"
    assert equal (snake "Fab") "fab"
}

def "test parity: exposes by kind" [] {
    assert equal (exposes "pub struct DataTable {" "DataTable" rust) true
    assert equal (exposes "pub fn data_table(" "DataTable" rust) true
    assert equal (exposes "pub struct Other {" "DataTable" rust) false
    assert equal (exposes "export component LButton inherits Rectangle" "Button" slint) true
    assert equal (exposes ".dodai-data-table {" "DataTable" css) true
}

def main [] { run-tests }
