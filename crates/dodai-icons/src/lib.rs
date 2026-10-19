//! Framework-agnostic icon geometry: polylines on a 24×24 grid.
//!
//! The pack follows the Cyberpunk design language: **no curves**. Everything is
//! straight segments, 45° chamfers and octagons (circles become 8-gons), drawn
//! with a uniform stroke. Adapters stroke the polylines in the theme colour;
//! [`svg`] / [`sprite`] export the same geometry as SVG for non-Rust consumers.

/// One stroked polyline: `(points, closed)`.
pub type Shape = (Vec<(f32, f32)>, bool);

/// Points from integer-ish pairs: `pl![3,11; 12,3]`.
macro_rules! pl {
    ($($x:expr, $y:expr);+ $(;)?) => { vec![$(($x as f32, $y as f32)),+] };
}

fn o(v: Vec<(f32, f32)>) -> Shape {
    (v, false)
}
fn c(v: Vec<(f32, f32)>) -> Shape {
    (v, true)
}

pub(crate) fn rect(x0: f32, y0: f32, x1: f32, y1: f32) -> Vec<(f32, f32)> {
    vec![(x0, y0), (x1, y0), (x1, y1), (x0, y1)]
}

/// Rectangle with the top-right and bottom-left corners chamfered by `k`
/// (the signature Cyberpunk cut).
pub fn cb(x0: f32, y0: f32, x1: f32, y1: f32, k: f32) -> Vec<(f32, f32)> {
    vec![
        (x0, y0),
        (x1 - k, y0),
        (x1, y0 + k),
        (x1, y1),
        (x0 + k, y1),
        (x0, y1 - k),
    ]
}

/// Rectangle with all four corners chamfered by `k`.
pub fn oct(x0: f32, y0: f32, x1: f32, y1: f32, k: f32) -> Vec<(f32, f32)> {
    vec![
        (x0 + k, y0),
        (x1 - k, y0),
        (x1, y0 + k),
        (x1, y1 - k),
        (x1 - k, y1),
        (x0 + k, y1),
        (x0, y1 - k),
        (x0, y0 + k),
    ]
}

/// Regular octagon of radius `r` (the "circle" of this icon language).
pub fn ngon(cx: f32, cy: f32, r: f32) -> Vec<(f32, f32)> {
    (0..8)
        .map(|i| {
            let a = (22.5 + 45.0 * i as f32).to_radians();
            (cx + r * a.cos(), cy + r * a.sin())
        })
        .collect()
}

macro_rules! icons {
    ($($v:ident, $name:literal, $group:literal => $body:expr;)+) => {
        /// Built-in icon set (24×24 design grid).
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum IconName { $($v),+ }

        /// Every icon, for galleries, exports and tests.
        pub const ALL: &[IconName] = &[$(IconName::$v),+];

        impl IconName {
            /// Polylines `(points, closed)` on the 24×24 grid. Pure.
            pub fn shapes(self) -> Vec<Shape> {
                match self { $(IconName::$v => $body),+ }
            }
            /// Stable kebab-case identifier (used for SVG ids and file names).
            pub fn name(self) -> &'static str {
                match self { $(IconName::$v => $name),+ }
            }
            /// Gallery group.
            pub fn group(self) -> &'static str {
                match self { $(IconName::$v => $group),+ }
            }
            /// Look an icon up by [`name`](Self::name).
            pub fn from_name(n: &str) -> Option<Self> {
                match n { $($name => Some(IconName::$v),)+ _ => None }
            }
        }
    };
}

icons! {
    // ── interface ────────────────────────────────────────────────────────
    Grid, "grid", "interface" => vec![
        c(rect(3.0, 3.0, 10.0, 10.0)), c(rect(14.0, 3.0, 21.0, 10.0)),
        c(rect(3.0, 14.0, 10.0, 21.0)), c(rect(14.0, 14.0, 21.0, 21.0)),
    ];
    Layers, "layers", "interface" => vec![
        c(pl![12,3; 21,8; 12,13; 3,8]), o(pl![3,12; 12,17; 21,12]), o(pl![3,16; 12,21; 21,16]),
    ];
    Sliders, "sliders", "interface" => vec![
        o(pl![4,7; 20,7]), o(pl![4,12; 20,12]), o(pl![4,17; 20,17]),
        c(rect(7.0, 5.0, 11.0, 9.0)), c(rect(13.0, 10.0, 17.0, 14.0)), c(rect(6.0, 15.0, 10.0, 19.0)),
    ];
    Menu, "menu", "interface" => vec![o(pl![4,6; 20,6]), o(pl![4,12; 20,12]), o(pl![4,18; 20,18])];
    Dots, "dots", "interface" => vec![
        c(rect(4.0, 10.0, 8.0, 14.0)), c(rect(10.0, 10.0, 14.0, 14.0)), c(rect(16.0, 10.0, 20.0, 14.0)),
    ];
    DotsVertical, "dots-vertical", "interface" => vec![
        c(rect(10.0, 4.0, 14.0, 8.0)), c(rect(10.0, 10.0, 14.0, 14.0)), c(rect(10.0, 16.0, 14.0, 20.0)),
    ];
    Table, "table", "interface" => vec![
        c(rect(3.0, 5.0, 21.0, 19.0)), o(pl![3,10; 21,10]), o(pl![10,10; 10,19]),
    ];
    Sidebar, "sidebar", "interface" => vec![c(cb(3.0, 4.0, 21.0, 20.0, 3.0)), o(pl![9,4; 9,20])];
    Maximize, "maximize", "interface" => vec![
        o(pl![4,9; 4,4; 9,4]), o(pl![15,4; 20,4; 20,9]), o(pl![20,15; 20,20; 15,20]), o(pl![9,20; 4,20; 4,15]),
    ];
    Minimize, "minimize", "interface" => vec![
        o(pl![4,9; 9,9; 9,4]), o(pl![15,4; 15,9; 20,9]), o(pl![20,15; 15,15; 15,20]), o(pl![9,20; 9,15; 4,15]),
    ];
    Filter, "filter", "interface" => vec![c(pl![3,4; 21,4; 14,12; 14,20; 10,18; 10,12])];
    Sort, "sort", "interface" => vec![
        o(pl![7,4; 7,20]), o(pl![3,8; 7,4; 11,8]), o(pl![17,20; 17,4]), o(pl![13,16; 17,20; 21,16]),
    ];
    Settings, "settings", "interface" => vec![
        c(oct(7.0, 7.0, 17.0, 17.0, 3.0)),
        o(pl![12,2; 12,6]), o(pl![12,18; 12,22]), o(pl![2,12; 6,12]), o(pl![18,12; 22,12]),
        o(pl![5,5; 8,8]), o(pl![16,16; 19,19]), o(pl![19,5; 16,8]), o(pl![5,19; 8,16]),
    ];
    Search, "search", "interface" => vec![c(ngon(10.0, 10.0, 6.5)), o(pl![15,15; 21,21])];
    ZoomIn, "zoom-in", "interface" => vec![
        c(ngon(10.0, 10.0, 6.5)), o(pl![15,15; 21,21]), o(pl![7,10; 13,10]), o(pl![10,7; 10,13]),
    ];
    ZoomOut, "zoom-out", "interface" => vec![
        c(ngon(10.0, 10.0, 6.5)), o(pl![15,15; 21,21]), o(pl![7,10; 13,10]),
    ];
    Sparkle, "sparkle", "interface" => vec![c(pl![12,2; 14,10; 22,12; 14,14; 12,22; 10,14; 2,12; 10,10])];
    Hash, "hash", "interface" => vec![
        o(pl![9,4; 7,20]), o(pl![17,4; 15,20]), o(pl![4,9; 20,9]), o(pl![4,15; 20,15]),
    ];
    Sun, "sun", "interface" => vec![
        c(ngon(12.0, 12.0, 4.0)), o(pl![12,2; 12,5]), o(pl![12,19; 12,22]), o(pl![2,12; 5,12]), o(pl![19,12; 22,12]),
        o(pl![5,5; 7,7]), o(pl![17,17; 19,19]), o(pl![19,5; 17,7]), o(pl![5,19; 7,17]),
    ];
    Wave, "wave", "interface" => vec![o(pl![2,12; 6,12; 8,6; 12,18; 16,6; 18,12; 22,12])];
    Pixels, "pixels", "interface" => vec![
        c(rect(3.0, 3.0, 8.0, 8.0)), c(rect(10.0, 3.0, 15.0, 8.0)),
        c(rect(3.0, 10.0, 8.0, 15.0)), c(rect(10.0, 10.0, 15.0, 15.0)), c(rect(17.0, 10.0, 22.0, 15.0)),
        c(rect(10.0, 17.0, 15.0, 22.0)), c(rect(17.0, 17.0, 22.0, 22.0)),
    ];
    Moon, "moon", "interface" => vec![c(pl![15,3; 10,5; 7,10; 8,16; 13,20; 20,17; 14,15; 12,9])];

    // ── navigation & arrows ──────────────────────────────────────────────
    ArrowRight, "arrow-right", "arrows" => vec![o(pl![4,12; 20,12]), o(pl![13,5; 20,12; 13,19])];
    ArrowLeft, "arrow-left", "arrows" => vec![o(pl![20,12; 4,12]), o(pl![11,5; 4,12; 11,19])];
    ArrowUp, "arrow-up", "arrows" => vec![o(pl![12,20; 12,4]), o(pl![5,11; 12,4; 19,11])];
    ArrowDown, "arrow-down", "arrows" => vec![o(pl![12,4; 12,20]), o(pl![5,13; 12,20; 19,13])];
    ChevronRight, "chevron-right", "arrows" => vec![o(pl![9,5; 16,12; 9,19])];
    ChevronLeft, "chevron-left", "arrows" => vec![o(pl![15,5; 8,12; 15,19])];
    ChevronDown, "chevron-down", "arrows" => vec![o(pl![5,9; 12,16; 19,9])];
    ChevronUp, "chevron-up", "arrows" => vec![o(pl![5,15; 12,8; 19,15])];
    ChevronsRight, "chevrons-right", "arrows" => vec![o(pl![5,6; 11,12; 5,18]), o(pl![13,6; 19,12; 13,18])];
    ChevronsLeft, "chevrons-left", "arrows" => vec![o(pl![11,6; 5,12; 11,18]), o(pl![19,6; 13,12; 19,18])];
    ExternalLink, "external-link", "arrows" => vec![
        o(pl![10,6; 5,6; 5,19; 18,19; 18,14]), o(pl![13,4; 20,4; 20,11]), o(pl![20,4; 11,13]),
    ];
    Home, "home", "arrows" => vec![
        o(pl![3,11; 12,3; 21,11]), o(pl![6,9; 6,21; 18,21; 18,9]), o(pl![10,21; 10,15; 14,15; 14,21]),
    ];
    Undo, "undo", "arrows" => vec![o(pl![9,5; 4,10; 9,15]), o(pl![4,10; 15,10; 20,15; 20,19])];
    Redo, "redo", "arrows" => vec![o(pl![15,5; 20,10; 15,15]), o(pl![20,10; 9,10; 4,15; 4,19])];
    Refresh, "refresh", "arrows" => vec![
        o(pl![4,12; 4,8; 8,4; 16,4; 20,8]), o(pl![20,3; 20,8; 15,8]),
        o(pl![20,12; 20,16; 16,20; 8,20; 4,16]), o(pl![4,21; 4,16; 9,16]),
    ];
    Repeat, "repeat", "arrows" => vec![
        o(pl![17,3; 20,6; 17,9]), o(pl![4,12; 4,9; 7,6; 20,6]),
        o(pl![7,21; 4,18; 7,15]), o(pl![20,12; 20,15; 17,18; 4,18]),
    ];
    Check, "check", "arrows" => vec![o(pl![4,12.5; 9.5,18; 20,6])];
    Close, "close", "arrows" => vec![o(pl![5,5; 19,19]), o(pl![19,5; 5,19])];
    Plus, "plus", "arrows" => vec![o(pl![12,4; 12,20]), o(pl![4,12; 20,12])];
    Minus, "minus", "arrows" => vec![o(pl![4,12; 20,12])];

    // ── actions ──────────────────────────────────────────────────────────
    Edit, "edit", "actions" => vec![c(pl![4,20; 5,15; 16,4; 20,8; 9,19]), o(pl![14,6; 18,10])];
    Trash, "trash", "actions" => vec![
        o(pl![4,6; 20,6]), c(pl![6,6; 7,21; 17,21; 18,6]), o(pl![9,6; 9,3; 15,3; 15,6]),
        o(pl![10,10; 10,17]), o(pl![14,10; 14,17]),
    ];
    Copy, "copy", "actions" => vec![c(cb(8.0, 8.0, 20.0, 20.0, 3.0)), o(pl![8,16; 4,16; 4,4; 16,4; 16,8])];
    Paste, "paste", "actions" => vec![c(rect(5.0, 5.0, 19.0, 21.0)), c(rect(9.0, 3.0, 15.0, 7.0))];
    Save, "save", "actions" => vec![
        c(cb(4.0, 4.0, 20.0, 20.0, 3.0)), c(rect(8.0, 4.0, 16.0, 9.0)), c(rect(8.0, 14.0, 16.0, 20.0)),
    ];
    Download, "download", "actions" => vec![o(pl![12,4; 12,16]), o(pl![6,11; 12,17; 18,11]), o(pl![4,20; 20,20])];
    Upload, "upload", "actions" => vec![o(pl![12,17; 12,5]), o(pl![6,10; 12,4; 18,10]), o(pl![4,20; 20,20])];
    Share, "share", "actions" => vec![
        c(rect(16.0, 3.0, 20.0, 7.0)), c(rect(4.0, 10.0, 8.0, 14.0)), c(rect(16.0, 17.0, 20.0, 21.0)),
        o(pl![16,5; 8,11]), o(pl![8,13; 16,19]),
    ];
    Send, "send", "actions" => vec![c(pl![3,3; 21,12; 3,21; 6,12]), o(pl![6,12; 21,12])];
    Link, "link", "actions" => vec![
        o(pl![10,7; 6,7; 3,10; 3,14; 6,17; 10,17]), o(pl![14,7; 18,7; 21,10; 21,14; 18,17; 14,17]), o(pl![8,12; 16,12]),
    ];
    Unlink, "unlink", "actions" => vec![
        o(pl![10,7; 6,7; 3,10; 3,14; 6,17; 8,17]), o(pl![14,7; 18,7; 21,10; 21,14; 18,17; 16,17]),
        o(pl![9,4; 8,7]), o(pl![15,20; 16,17]),
    ];
    Print, "print", "actions" => vec![
        c(rect(3.0, 9.0, 21.0, 18.0)), o(pl![7,9; 7,3; 17,3; 17,9]), c(rect(7.0, 14.0, 17.0, 21.0)),
    ];
    Power, "power", "actions" => vec![
        o(pl![12,3; 12,11]), o(pl![7,6; 4,10; 4,15; 8,20; 16,20; 20,15; 20,10; 17,6]),
    ];
    Lock, "lock", "actions" => vec![
        c(rect(5.0, 11.0, 19.0, 21.0)), o(pl![8,11; 8,7; 10,4; 14,4; 16,7; 16,11]), o(pl![12,15; 12,17]),
    ];
    Unlock, "unlock", "actions" => vec![
        c(rect(5.0, 11.0, 19.0, 21.0)), o(pl![8,11; 8,7; 10,4; 14,4; 16,7]), o(pl![12,15; 12,17]),
    ];
    Key, "key", "actions" => vec![
        c(oct(3.0, 6.0, 11.0, 14.0, 2.0)), o(pl![11,10; 21,10]), o(pl![18,10; 18,14]), o(pl![15,10; 15,13]),
    ];
    Eye, "eye", "actions" => vec![c(pl![2,12; 7,6; 17,6; 22,12; 17,18; 7,18]), c(oct(9.0, 9.0, 15.0, 15.0, 2.0))];
    EyeOff, "eye-off", "actions" => vec![
        c(pl![2,12; 7,6; 17,6; 22,12; 17,18; 7,18]), c(oct(9.0, 9.0, 15.0, 15.0, 2.0)), o(pl![3,3; 21,21]),
    ];
    Pin, "pin", "actions" => vec![c(pl![9,3; 15,3; 16,10; 19,13; 5,13; 8,10]), o(pl![12,13; 12,21])];
    Bookmark, "bookmark", "actions" => vec![c(pl![6,3; 18,3; 18,21; 12,16; 6,21])];
    Star, "star", "actions" => vec![c(pl![12,2; 15,9; 22,9; 16,14; 18,21; 12,17; 6,21; 8,14; 2,9; 9,9])];
    Heart, "heart", "actions" => vec![c(pl![12,20; 3,11; 3,7; 6,4; 10,4; 12,7; 14,4; 18,4; 21,7; 21,11])];
    Flag, "flag", "actions" => vec![o(pl![5,21; 5,3]), c(pl![5,4; 19,4; 16,8; 19,12; 5,12])];
    Tag, "tag", "actions" => vec![c(pl![3,3; 12,3; 21,12; 12,21; 3,12]), c(rect(6.0, 6.0, 9.0, 9.0))];
    Pause, "pause", "actions" => vec![c(rect(6.0, 5.0, 10.0, 19.0)), c(rect(14.0, 5.0, 18.0, 19.0))];
    Play, "play", "actions" => vec![c(pl![7,4; 20,12; 7,20])];
    Stop, "stop", "actions" => vec![c(rect(6.0, 6.0, 18.0, 18.0))];
    Next, "next", "actions" => vec![c(pl![5,5; 15,12; 5,19]), o(pl![19,5; 19,19])];
    Prev, "prev", "actions" => vec![c(pl![19,5; 9,12; 19,19]), o(pl![5,5; 5,19])];
    Record, "record", "actions" => vec![c(ngon(12.0, 12.0, 8.5)), c(rect(10.0, 10.0, 14.0, 14.0))];
    Volume, "volume", "actions" => vec![
        c(pl![4,9; 8,9; 13,5; 13,19; 8,15; 4,15]), o(pl![16,9; 18,12; 16,15]), o(pl![19,6; 22,12; 19,18]),
    ];
    Mute, "mute", "actions" => vec![
        c(pl![4,9; 8,9; 13,5; 13,19; 8,15; 4,15]), o(pl![16,9; 21,15]), o(pl![21,9; 16,15]),
    ];
    Mic, "mic", "actions" => vec![
        c(cb(9.0, 3.0, 15.0, 14.0, 2.0)), o(pl![6,11; 6,15; 9,18; 15,18; 18,15; 18,11]),
        o(pl![12,18; 12,21]), o(pl![8,21; 16,21]),
    ];
    Dot, "dot", "actions" => vec![c(rect(10.0, 10.0, 14.0, 14.0))];

    // ── status ───────────────────────────────────────────────────────────
    Info, "info", "status" => vec![c(oct(3.0, 3.0, 21.0, 21.0, 6.0)), o(pl![12,11; 12,17]), o(pl![12,7; 12,8.5])];
    Warning, "warning", "status" => vec![c(pl![12,3; 22,20; 2,20]), o(pl![12,9; 12,14]), o(pl![12,17; 12,18])];
    Error, "error", "status" => vec![c(oct(3.0, 3.0, 21.0, 21.0, 6.0)), o(pl![8,8; 16,16]), o(pl![16,8; 8,16])];
    Help, "help", "status" => vec![
        c(oct(3.0, 3.0, 21.0, 21.0, 6.0)),
        o(pl![9,9; 9,8; 11,6; 13,6; 15,8; 15,10; 12,13; 12,14]), o(pl![12,17; 12,18]),
    ];
    CheckCircle, "check-circle", "status" => vec![c(oct(3.0, 3.0, 21.0, 21.0, 6.0)), o(pl![8,12; 11,15; 16,9])];
    Bell, "bell", "status" => vec![
        c(pl![6,17; 6,10; 9,5; 15,5; 18,10; 18,17]), o(pl![4,17; 20,17]), o(pl![10,20; 14,20]),
    ];
    BellOff, "bell-off", "status" => vec![
        c(pl![6,17; 6,10; 9,5; 15,5; 18,10; 18,17]), o(pl![4,17; 20,17]), o(pl![10,20; 14,20]), o(pl![3,3; 21,21]),
    ];
    Shield, "shield", "status" => vec![c(pl![12,2; 20,5; 20,13; 12,22; 4,13; 4,5])];
    ShieldCheck, "shield-check", "status" => vec![
        c(pl![12,2; 20,5; 20,13; 12,22; 4,13; 4,5]), o(pl![8,12; 11,15; 16,9]),
    ];
    Bolt, "bolt", "status" => vec![c(pl![13,2; 5,13; 11,13; 9,22; 19,10; 13,10])];
    Flame, "flame", "status" => vec![c(pl![12,2; 16,8; 18,13; 16,19; 12,22; 8,19; 6,13; 9,9; 10,12; 12,8])];
    Clock, "clock", "status" => vec![c(oct(3.0, 3.0, 21.0, 21.0, 6.0)), o(pl![12,7; 12,12; 16,14])];
    Timer, "timer", "status" => vec![
        c(ngon(12.0, 13.5, 8.0)), o(pl![12,13; 12,9]), o(pl![10,2; 14,2]), o(pl![12,2; 12,5]), o(pl![18,6; 20,4]),
    ];
    Calendar, "calendar", "status" => vec![
        c(rect(3.0, 5.0, 21.0, 21.0)), o(pl![3,10; 21,10]), o(pl![8,3; 8,7]), o(pl![16,3; 16,7]),
    ];
    Hourglass, "hourglass", "status" => vec![
        o(pl![6,3; 18,3]), o(pl![6,21; 18,21]),
        o(pl![7,3; 7,7; 12,12; 7,17; 7,21]), o(pl![17,3; 17,7; 12,12; 17,17; 17,21]),
    ];
    Battery, "battery", "status" => vec![
        c(rect(3.0, 7.0, 19.0, 17.0)), c(rect(19.0, 10.0, 21.0, 14.0)), c(rect(5.0, 9.0, 9.0, 15.0)),
    ];
    Signal, "signal", "status" => vec![
        c(rect(3.0, 16.0, 6.0, 20.0)), c(rect(8.0, 12.0, 11.0, 20.0)),
        c(rect(13.0, 8.0, 16.0, 20.0)), c(rect(18.0, 4.0, 21.0, 20.0)),
    ];
    Wifi, "wifi", "status" => vec![
        o(pl![2,9; 7,5; 17,5; 22,9]), o(pl![5,13; 9,10; 15,10; 19,13]), o(pl![8,17; 10,15; 14,15; 16,17]), o(pl![12,19; 12,20]),
    ];
    Bluetooth, "bluetooth", "status" => vec![o(pl![6,7; 18,17; 12,22; 12,2; 18,7; 6,17])];

    // ── data & dev ───────────────────────────────────────────────────────
    BarChart, "bar-chart", "data" => vec![
        c(rect(4.0, 13.0, 8.0, 20.0)), c(rect(10.0, 8.0, 14.0, 20.0)), c(rect(16.0, 4.0, 20.0, 20.0)),
    ];
    LineChart, "line-chart", "data" => vec![o(pl![3,3; 3,21; 21,21]), o(pl![6,16; 10,11; 14,14; 20,6])];
    AreaChart, "area-chart", "data" => vec![c(pl![3,20; 3,14; 8,9; 13,13; 21,5; 21,20])];
    PieChart, "pie-chart", "data" => vec![c(ngon(12.0, 12.0, 9.0)), o(pl![12,3; 12,12]), o(pl![12,12; 18.4,18.4])];
    Database, "database", "data" => vec![
        c(cb(4.0, 3.0, 20.0, 8.0, 2.0)), c(cb(4.0, 9.0, 20.0, 14.0, 2.0)), c(cb(4.0, 15.0, 20.0, 20.0, 2.0)),
    ];
    Server, "server", "data" => vec![
        c(rect(3.0, 4.0, 21.0, 10.0)), c(rect(3.0, 14.0, 21.0, 20.0)),
        o(pl![6,7; 8,7]), o(pl![6,17; 8,17]), o(pl![16,7; 18,7]), o(pl![16,17; 18,17]),
    ];
    Cpu, "cpu", "data" => vec![
        c(cb(7.0, 7.0, 17.0, 17.0, 2.0)), c(rect(10.0, 10.0, 14.0, 14.0)),
        o(pl![10,3; 10,7]), o(pl![14,3; 14,7]), o(pl![10,17; 10,21]), o(pl![14,17; 14,21]),
        o(pl![3,10; 7,10]), o(pl![3,14; 7,14]), o(pl![17,10; 21,10]), o(pl![17,14; 21,14]),
    ];
    Chip, "chip", "data" => vec![c(cb(4.0, 4.0, 20.0, 20.0, 4.0)), c(rect(8.0, 8.0, 16.0, 16.0))];
    Terminal, "terminal", "data" => vec![c(cb(3.0, 4.0, 21.0, 20.0, 3.0)), o(pl![7,9; 11,12; 7,15]), o(pl![13,15; 17,15])];
    Code, "code", "data" => vec![o(pl![8,6; 3,12; 8,18]), o(pl![16,6; 21,12; 16,18]), o(pl![14,4; 10,20])];
    Bug, "bug", "data" => vec![
        c(cb(8.0, 7.0, 16.0, 19.0, 2.0)), o(pl![8,12; 4,12]), o(pl![16,12; 20,12]), o(pl![8,16; 5,19]),
        o(pl![16,16; 19,19]), o(pl![8,9; 5,6]), o(pl![16,9; 19,6]), o(pl![10,7; 10,4]), o(pl![14,7; 14,4]),
    ];
    GitBranch, "git-branch", "data" => vec![
        c(rect(4.0, 3.0, 8.0, 7.0)), c(rect(16.0, 3.0, 20.0, 7.0)), c(rect(4.0, 17.0, 8.0, 21.0)),
        o(pl![6,7; 6,17]), o(pl![18,7; 18,10; 6,14]),
    ];
    Folder, "folder", "data" => vec![c(pl![3,5; 10,5; 12,8; 21,8; 21,20; 3,20])];
    File, "file", "data" => vec![c(pl![5,3; 14,3; 19,8; 19,21; 5,21]), o(pl![14,3; 14,8; 19,8])];
    FileText, "file-text", "data" => vec![
        c(pl![5,3; 14,3; 19,8; 19,21; 5,21]), o(pl![14,3; 14,8; 19,8]), o(pl![8,12; 16,12]), o(pl![8,16; 16,16]),
    ];
    Image, "image", "data" => vec![
        c(rect(3.0, 4.0, 21.0, 20.0)), o(pl![3,16; 9,10; 14,15; 17,12; 21,16]), c(rect(15.0, 7.0, 18.0, 10.0)),
    ];
    Archive, "archive", "data" => vec![c(rect(3.0, 4.0, 21.0, 9.0)), c(rect(5.0, 9.0, 19.0, 21.0)), o(pl![10,13; 14,13])];
    Cloud, "cloud", "data" => vec![c(pl![6,18; 3,15; 3,12; 6,10; 8,6; 14,6; 17,9; 20,10; 21,14; 21,18])];
    Network, "network", "data" => vec![
        c(rect(9.0, 3.0, 15.0, 8.0)), c(rect(3.0, 16.0, 9.0, 21.0)), c(rect(15.0, 16.0, 21.0, 21.0)),
        o(pl![12,8; 12,12]), o(pl![6,16; 6,12; 18,12; 18,16]),
    ];

    // ── objects & people ─────────────────────────────────────────────────
    User, "user", "objects" => vec![c(oct(8.0, 3.0, 16.0, 11.0, 2.0)), o(pl![4,21; 4,17; 8,14; 16,14; 20,17; 20,21])];
    Users, "users", "objects" => vec![
        c(oct(5.0, 4.0, 11.0, 10.0, 2.0)), o(pl![2,20; 2,16; 5,13; 11,13; 14,16; 14,20]),
        o(pl![15,5; 18,4; 20,6; 20,9; 18,11; 15,10]), o(pl![17,14; 21,16; 22,20]),
    ];
    Mail, "mail", "objects" => vec![c(rect(3.0, 5.0, 21.0, 19.0)), o(pl![3,5; 12,13; 21,5])];
    Chat, "chat", "objects" => vec![
        c(pl![4,4; 20,4; 20,16; 12,16; 7,20; 7,16; 4,16]), o(pl![8,8; 16,8]), o(pl![8,12; 13,12]),
    ];
    Phone, "phone", "objects" => vec![c(cb(7.0, 2.0, 17.0, 22.0, 2.0)), o(pl![10,19; 14,19])];
    Globe, "globe", "objects" => vec![
        c(ngon(12.0, 12.0, 9.0)), o(pl![3,12; 21,12]), o(pl![12,3; 9,12; 12,21]), o(pl![12,3; 15,12; 12,21]),
    ];
    MapPin, "map-pin", "objects" => vec![c(pl![12,22; 5,12; 5,7; 8,3; 16,3; 19,7; 19,12]), c(rect(10.0, 7.0, 14.0, 11.0))];
    Camera, "camera", "objects" => vec![
        c(rect(3.0, 7.0, 21.0, 20.0)), o(pl![8,7; 9,4; 15,4; 16,7]), c(ngon(12.0, 13.5, 3.5)),
    ];
    Cart, "cart", "objects" => vec![
        o(pl![2,3; 5,3; 7,15; 19,15; 21,7; 6,7]), c(rect(8.0, 18.0, 10.0, 20.0)), c(rect(16.0, 18.0, 18.0, 20.0)),
    ];
    Wallet, "wallet", "objects" => vec![c(cb(3.0, 6.0, 21.0, 20.0, 3.0)), c(rect(15.0, 12.0, 21.0, 16.0))];
    Gift, "gift", "objects" => vec![
        c(rect(4.0, 10.0, 20.0, 21.0)), c(rect(3.0, 6.0, 21.0, 10.0)), o(pl![12,6; 12,21]),
        o(pl![12,6; 8,3; 6,5; 10,6]), o(pl![12,6; 16,3; 18,5; 14,6]),
    ];
    Cube, "cube", "objects" => vec![
        c(pl![12,2; 21,7; 21,17; 12,22; 3,17; 3,7]), o(pl![3,7; 12,12; 21,7]), o(pl![12,12; 12,22]),
    ];
    LayersPlus, "layers-plus", "objects" => vec![
        c(pl![12,3; 21,8; 12,13; 3,8]), o(pl![3,12; 12,17; 21,12]), o(pl![12,17; 12,22]), o(pl![9,20; 15,20]),
    ];
    Music, "music", "objects" => vec![
        o(pl![9,18; 9,5; 19,3; 19,16]), c(rect(5.0, 16.0, 9.0, 20.0)), c(rect(15.0, 14.0, 19.0, 18.0)),
    ];
    Gamepad, "gamepad", "objects" => vec![
        c(cb(2.0, 8.0, 22.0, 17.0, 3.0)), o(pl![7,10; 7,15]), o(pl![5,12.5; 9,12.5]),
        c(rect(15.0, 11.0, 17.0, 13.0)), c(rect(18.0, 12.0, 20.0, 14.0)),
    ];

    // ── cyber ────────────────────────────────────────────────────────────
    Target, "target", "cyber" => vec![c(ngon(12.0, 12.0, 9.0)), c(ngon(12.0, 12.0, 5.0)), c(rect(11.0, 11.0, 13.0, 13.0))];
    Crosshair, "crosshair", "cyber" => vec![
        c(ngon(12.0, 12.0, 6.0)), o(pl![12,2; 12,8]), o(pl![12,16; 12,22]), o(pl![2,12; 8,12]), o(pl![16,12; 22,12]),
    ];
    Radar, "radar", "cyber" => vec![c(ngon(12.0, 12.0, 9.0)), c(ngon(12.0, 12.0, 4.0)), o(pl![12,12; 19,5])];
    Skull, "skull", "cyber" => vec![
        c(pl![5,10; 5,6; 8,3; 16,3; 19,6; 19,10; 17,12; 17,17; 7,17; 7,12]),
        c(rect(8.0, 8.0, 11.0, 11.0)), c(rect(13.0, 8.0, 16.0, 11.0)), o(pl![10,17; 10,21]), o(pl![14,17; 14,21]),
    ];
    Visor, "visor", "cyber" => vec![c(pl![2,9; 5,7; 19,7; 22,9; 22,14; 19,16; 5,16; 2,14]), o(pl![7,11.5; 17,11.5])];
    Drone, "drone", "cyber" => vec![
        c(cb(9.0, 9.0, 15.0, 15.0, 2.0)), o(pl![3,5; 9,9]), o(pl![21,5; 15,9]), o(pl![3,19; 9,15]), o(pl![21,19; 15,15]),
        o(pl![2,4; 6,4]), o(pl![18,4; 22,4]), o(pl![2,20; 6,20]), o(pl![18,20; 22,20]),
    ];
    Robot, "robot", "cyber" => vec![
        c(cb(5.0, 7.0, 19.0, 19.0, 3.0)), o(pl![12,7; 12,3]), c(rect(8.0, 11.0, 11.0, 14.0)),
        c(rect(13.0, 11.0, 16.0, 14.0)), o(pl![9,16; 15,16]),
    ];
}

/// Plain `<path>` data for one shape (`M x y L … [Z]`), integers trimmed.
fn path_d(points: &[(f32, f32)], closed: bool) -> String {
    let mut d = String::new();
    for (i, (x, y)) in points.iter().enumerate() {
        d.push(if i == 0 { 'M' } else { 'L' });
        d.push_str(&format!("{} {}", trim(*x), trim(*y)));
        d.push(' ');
    }
    if closed {
        d.push('Z');
    } else {
        d.pop();
    }
    d
}

fn trim(v: f32) -> String {
    let s = format!("{v:.2}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

impl IconName {
    /// SVG `d` attribute for the whole icon (all shapes, one path).
    pub fn path_data(self) -> String {
        self.shapes()
            .iter()
            .map(|(p, closed)| path_d(p, *closed))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// A standalone SVG for `icon`, stroked with `currentColor` so CSS colours it.
pub fn svg(icon: IconName, size: u32, stroke_width: f32) -> String {
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{size}\" height=\"{size}\" viewBox=\"0 0 24 24\" \
fill=\"none\" stroke=\"currentColor\" stroke-width=\"{}\" stroke-linecap=\"square\" stroke-linejoin=\"miter\">\
<path d=\"{}\"/></svg>\n",
        trim(stroke_width),
        icon.path_data()
    )
}

/// One SVG sprite sheet with a `<symbol id="dodai-<name>">` per icon.
pub fn sprite(stroke_width: f32) -> String {
    let mut s = String::from("<svg xmlns=\"http://www.w3.org/2000/svg\" style=\"display:none\">\n");
    for &i in ALL {
        s.push_str(&format!(
            "<symbol id=\"dodai-{}\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" \
stroke-width=\"{}\" stroke-linecap=\"square\" stroke-linejoin=\"miter\"><path d=\"{}\"/></symbol>\n",
            i.name(),
            trim(stroke_width),
            i.path_data()
        ));
    }
    s.push_str("</svg>\n");
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn pack_is_extensive() {
        assert!(ALL.len() >= 120, "only {} icons", ALL.len());
    }

    #[test]
    fn shapes_inside_grid() {
        for &n in ALL {
            let s = n.shapes();
            assert!(!s.is_empty(), "{} empty", n.name());
            for (p, _) in s {
                assert!(p.len() >= 2, "{} has a degenerate shape", n.name());
                assert!(
                    p.iter()
                        .all(|&(x, y)| (0.0..=24.0).contains(&x) && (0.0..=24.0).contains(&y)),
                    "{} leaves the 24x24 grid",
                    n.name()
                );
            }
        }
    }

    #[test]
    fn names_are_unique_kebab_and_round_trip() {
        let mut seen = HashSet::new();
        for &n in ALL {
            let name = n.name();
            assert!(seen.insert(name), "duplicate name {name}");
            assert!(
                name.chars().all(|c| c.is_ascii_lowercase() || c == '-'),
                "{name} is not kebab-case"
            );
            assert_eq!(IconName::from_name(name), Some(n));
        }
        assert_eq!(IconName::from_name("nope"), None);
    }

    /// The pack is curve-free: every segment is horizontal, vertical or 45°,
    /// or part of a regular octagon / a deliberate diagonal.
    #[test]
    fn no_degenerate_segments() {
        for &n in ALL {
            for (p, closed) in n.shapes() {
                let mut pts = p.clone();
                if closed {
                    pts.push(p[0]);
                }
                for w in pts.windows(2) {
                    let (dx, dy) = (w[1].0 - w[0].0, w[1].1 - w[0].1);
                    assert!(
                        dx.abs() + dy.abs() > 0.4,
                        "{} has a zero-length segment",
                        n.name()
                    );
                }
            }
        }
    }

    #[test]
    fn groups_are_known_and_every_group_has_icons() {
        let groups: HashSet<_> = ALL.iter().map(|i| i.group()).collect();
        for g in [
            "interface",
            "arrows",
            "actions",
            "status",
            "data",
            "objects",
            "cyber",
        ] {
            assert!(groups.contains(g), "missing group {g}");
        }
        assert_eq!(groups.len(), 7);
    }

    #[test]
    fn svg_exports_are_well_formed() {
        let s = svg(IconName::Home, 24, 2.0);
        assert!(
            s.starts_with("<svg")
                && s.contains("viewBox=\"0 0 24 24\"")
                && s.contains("<path d=\"M3 11 L12 3")
        );
        let sp = sprite(2.0);
        assert_eq!(sp.matches("<symbol ").count(), ALL.len());
        assert!(sp.contains("id=\"dodai-skull\""));
        assert_eq!(trim(12.50), "12.5");
        assert_eq!(trim(3.0), "3");
        assert!(IconName::Check.path_data().contains("12.5"));
    }
}
