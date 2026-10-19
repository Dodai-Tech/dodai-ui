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
        c(rect(3.0, 3.0, 9.0, 9.0)), c(rect(15.0, 3.0, 21.0, 9.0)), c(rect(9.0, 9.0, 15.0, 15.0)),
        c(rect(3.0, 15.0, 9.0, 21.0)), c(rect(15.0, 15.0, 21.0, 21.0)),
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
    Heart, "heart", "actions" => vec![c(pl![12,21; 3,12; 3,7; 6,4; 9,4; 12,7; 15,4; 18,4; 21,7; 21,12])];
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
        c(oct(4.0, 4.0, 10.0, 10.0, 2.0)), o(pl![1,20; 1,17; 4,13; 10,13; 13,17; 13,20]),
        c(oct(14.0, 4.0, 20.0, 10.0, 2.0)), o(pl![16,13; 19,13; 22,17; 22,20]),
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
        c(cb(2.0, 7.0, 22.0, 18.0, 3.0)), o(pl![7,9.5; 7,15.5]), o(pl![4,12.5; 10,12.5]),
        c(rect(14.5, 10.5, 16.5, 12.5)), c(rect(17.5, 12.5, 19.5, 14.5)),
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

    // ── batch 1: hand-drawn, named after their Material Symbols twin ─────
    // media
    FastForward, "fast_forward", "media" => vec![c(pl![3,5; 11,12; 3,19]), c(pl![12,5; 20,12; 12,19])];
    FastRewind, "fast_rewind", "media" => vec![c(pl![21,5; 13,12; 21,19]), c(pl![12,5; 4,12; 12,19])];
    Shuffle, "shuffle", "media" => vec![
        o(pl![3,7; 8,7; 16,17; 21,17]), o(pl![3,17; 8,17; 11,14]), o(pl![13,10; 16,7; 21,7]),
        o(pl![18,4; 21,7; 18,10]), o(pl![18,14; 21,17; 18,20]),
    ];
    PlayCircle, "play_circle", "media" => vec![c(oct(3.0, 3.0, 21.0, 21.0, 6.0)), c(pl![10,8; 16,12; 10,16])];
    PauseCircle, "pause_circle", "media" => vec![
        c(oct(3.0, 3.0, 21.0, 21.0, 6.0)), o(pl![10,9; 10,15]), o(pl![14,9; 14,15]),
    ];
    StopCircle, "stop_circle", "media" => vec![c(oct(3.0, 3.0, 21.0, 21.0, 6.0)), c(rect(9.0, 9.0, 15.0, 15.0))];
    VolumeDown, "volume_down", "media" => vec![c(pl![4,9; 8,9; 13,5; 13,19; 8,15; 4,15]), o(pl![16,9; 18,12; 16,15])];
    VolumeOff, "volume_off", "media" => vec![c(pl![4,9; 8,9; 13,5; 13,19; 8,15; 4,15]), o(pl![3,3; 21,21])];
    MicOff, "mic_off", "media" => vec![
        c(cb(9.0, 3.0, 15.0, 14.0, 2.0)), o(pl![6,11; 6,15; 9,18; 15,18; 18,15; 18,11]),
        o(pl![12,18; 12,21]), o(pl![8,21; 16,21]), o(pl![3,3; 21,21]),
    ];
    Videocam, "videocam", "media" => vec![c(cb(2.0, 6.0, 16.0, 18.0, 3.0)), c(pl![16,10; 22,6; 22,18; 16,14])];
    Movie, "movie", "media" => vec![
        c(cb(3.0, 4.0, 21.0, 20.0, 3.0)), o(pl![3,9; 21,9]), o(pl![3,15; 21,15]), o(pl![8,4; 8,20]), o(pl![16,4; 16,20]),
    ];
    Headphones, "headphones", "media" => vec![
        o(pl![4,16; 4,11; 7,5; 17,5; 20,11; 20,16]), c(rect(3.0, 14.0, 7.0, 20.0)), c(rect(17.0, 14.0, 21.0, 20.0)),
    ];
    Equalizer, "equalizer", "media" => vec![o(pl![5,20; 5,11]), o(pl![9,20; 9,4]), o(pl![13,20; 13,8]), o(pl![17,20; 17,14]), o(pl![21,20; 21,6])];
    Fullscreen, "fullscreen", "media" => vec![
        o(pl![4,9; 4,4; 9,4]), o(pl![15,4; 20,4; 20,9]), o(pl![20,15; 20,20; 15,20]), o(pl![9,20; 4,20; 4,15]),
    ];
    FullscreenExit, "fullscreen_exit", "media" => vec![
        o(pl![4,9; 9,9; 9,4]), o(pl![15,4; 15,9; 20,9]), o(pl![20,15; 15,15; 15,20]), o(pl![9,20; 9,15; 4,15]),
    ];
    // files
    FolderOpen, "folder_open", "files" => vec![
        o(pl![3,20; 3,5; 10,5; 12,8; 19,8; 19,11]), c(pl![3,20; 6,11; 22,11; 19,20]),
    ];
    CreateNewFolder, "create_new_folder", "files" => vec![
        c(pl![3,5; 10,5; 12,8; 21,8; 21,20; 3,20]), o(pl![12,11; 12,17]), o(pl![9,14; 15,14]),
    ];
    AttachFile, "attach_file", "files" => vec![
        o(pl![17,8; 17,17; 14,20; 10,20; 7,17; 7,6; 9,4; 13,4; 15,6; 15,15; 13,16; 11,16; 10,15; 10,8]),
    ];
    CloudUpload, "cloud_upload", "files" => vec![
        c(pl![6,18; 3,15; 3,12; 6,10; 8,6; 14,6; 17,9; 20,10; 21,14; 21,18]), o(pl![12,16; 12,10]), o(pl![9,13; 12,10; 15,13]),
    ];
    CloudDownload, "cloud_download", "files" => vec![
        c(pl![6,18; 3,15; 3,12; 6,10; 8,6; 14,6; 17,9; 20,10; 21,14; 21,18]), o(pl![12,10; 12,16]), o(pl![9,13; 12,16; 15,13]),
    ];
    CloudOff, "cloud_off", "files" => vec![
        c(pl![6,18; 3,15; 3,12; 6,10; 8,6; 14,6; 17,9; 20,10; 21,14; 21,18]), o(pl![3,3; 21,21]),
    ];
    Article, "article", "files" => vec![
        c(cb(4.0, 3.0, 20.0, 21.0, 3.0)), o(pl![8,8; 16,8]), o(pl![8,12; 16,12]), o(pl![8,16; 13,16]),
    ];
    ContentCut, "content_cut", "files" => vec![
        o(pl![7.5,16; 18,4]), o(pl![16.5,16; 6,4]), c(ngon(6.0, 18.5, 2.5)), c(ngon(18.0, 18.5, 2.5)),
    ];
    // text & editing
    FormatBold, "format_bold", "edit" => vec![
        c(pl![7,4; 13,4; 16,7; 16,10; 13,12; 7,12]), c(pl![7,12; 14,12; 17,14.5; 17,17.5; 14,20; 7,20]),
    ];
    FormatItalic, "format_italic", "edit" => vec![o(pl![10,4; 18,4]), o(pl![6,20; 14,20]), o(pl![15,4; 9,20])];
    FormatUnderlined, "format_underlined", "edit" => vec![o(pl![7,4; 7,11; 9,15; 15,15; 17,11; 17,4]), o(pl![5,20; 19,20])];
    FormatListBulleted, "format_list_bulleted", "edit" => vec![
        c(rect(3.0, 5.0, 6.0, 8.0)), c(rect(3.0, 10.5, 6.0, 13.5)), c(rect(3.0, 16.0, 6.0, 19.0)),
        o(pl![9,6.5; 21,6.5]), o(pl![9,12; 21,12]), o(pl![9,17.5; 21,17.5]),
    ];
    FormatListNumbered, "format_list_numbered", "edit" => vec![
        o(pl![3,7; 5,5; 5,9]), o(pl![3,12; 6,12; 6,14; 3,14; 3,16; 6,16]),
        o(pl![9,7; 21,7]), o(pl![9,13; 21,13]), o(pl![9,19; 21,19]),
    ];
    FormatAlignLeft, "format_align_left", "edit" => vec![o(pl![4,6; 20,6]), o(pl![4,10; 14,10]), o(pl![4,14; 20,14]), o(pl![4,18; 14,18])];
    FormatAlignCenter, "format_align_center", "edit" => vec![o(pl![4,6; 20,6]), o(pl![7,10; 17,10]), o(pl![4,14; 20,14]), o(pl![7,18; 17,18])];
    Title, "title", "edit" => vec![o(pl![4,7; 4,4; 20,4; 20,7]), o(pl![12,4; 12,20]), o(pl![9,20; 15,20])];
    Crop, "crop", "edit" => vec![o(pl![6,2; 6,18; 22,18]), o(pl![2,6; 18,6; 18,22])];
    Brush, "brush", "edit" => vec![c(pl![19,3; 21,5; 13,13; 11,11]), c(pl![10,12; 12,14; 11,17; 8,20; 3,21; 4,16; 7,13])];
    // devices
    Tablet, "tablet", "device" => vec![
        c(cb(2.0, 4.0, 22.0, 20.0, 3.0)), c(rect(5.0, 7.0, 16.0, 17.0)), o(pl![19,10; 19,14]),
    ];
    Monitor, "monitor", "device" => vec![c(cb(2.0, 4.0, 22.0, 16.0, 3.0)), o(pl![12,16; 12,20]), o(pl![8,20; 16,20])];
    Keyboard, "keyboard", "device" => vec![
        c(cb(2.0, 6.0, 22.0, 18.0, 3.0)), o(pl![6,10; 8,10]), o(pl![11,10; 13,10]), o(pl![16,10; 18,10]), o(pl![8,14.5; 16,14.5]),
    ];
    Mouse, "mouse", "device" => vec![c(oct(6.0, 2.0, 18.0, 22.0, 4.0)), o(pl![12,2; 12,9]), o(pl![6,9; 18,9])];
    Watch, "watch", "device" => vec![
        c(oct(7.0, 6.0, 17.0, 18.0, 2.0)), o(pl![9,6; 9,2; 15,2; 15,6]), o(pl![9,18; 9,22; 15,22; 15,18]), o(pl![12,10; 12,12; 14,13]),
    ];
    Tv, "tv", "device" => vec![c(cb(2.0, 6.0, 22.0, 19.0, 3.0)), o(pl![8,2; 12,6; 16,2])];
    Usb, "usb", "device" => vec![
        o(pl![12,6; 12,18]), c(pl![10,6; 12,3; 14,6]), o(pl![12,14; 7,10; 7,9]), c(rect(5.0, 6.0, 9.0, 9.0)),
        o(pl![12,11; 17,8]), c(ngon(17.5, 6.0, 1.8)), c(ngon(12.0, 20.5, 2.0)),
    ];
    Router, "router", "device" => vec![
        c(cb(3.0, 13.0, 21.0, 20.0, 2.0)), o(pl![7,16.5; 8,16.5]), o(pl![7,13; 7,7]), o(pl![17,13; 17,7]), o(pl![12,13; 12,10]),
    ];
    WifiOff, "wifi_off", "device" => vec![
        o(pl![2,9; 7,5; 17,5; 22,9]), o(pl![5,13; 9,10; 15,10; 19,13]), o(pl![8,17; 10,15; 14,15; 16,17]), o(pl![12,19; 12,20]), o(pl![3,3; 21,21]),
    ];
    // commerce
    ShoppingBag, "shopping_bag", "commerce" => vec![c(rect(4.0, 7.0, 20.0, 21.0)), o(pl![8,10; 8,6; 10,3; 14,3; 16,6; 16,10])];
    Store, "store", "commerce" => vec![
        c(pl![5,4; 19,4; 21,10; 3,10]), o(pl![5,10; 5,21; 19,21; 19,10]), c(rect(10.0, 15.0, 14.0, 21.0)),
    ];
    CreditCard, "credit_card", "commerce" => vec![c(cb(2.0, 5.0, 22.0, 19.0, 3.0)), o(pl![2,10; 22,10]), o(pl![5,15; 9,15])];
    Receipt, "receipt", "commerce" => vec![
        c(pl![5,3; 19,3; 19,21; 16,19; 12,21; 8,19; 5,21]), o(pl![8,8; 16,8]), o(pl![8,12; 16,12]),
    ];
    LocalShipping, "local_shipping", "commerce" => vec![
        c(rect(2.0, 6.0, 14.0, 17.0)), c(pl![14,9; 18,9; 21,12; 21,17; 14,17]), c(ngon(7.0, 18.5, 2.0)), c(ngon(17.0, 18.5, 2.0)),
    ];
    // people & things
    PersonAdd, "person_add", "people" => vec![
        c(oct(5.0, 4.0, 13.0, 12.0, 2.0)), o(pl![2,21; 2,18; 5,15; 13,15; 16,18; 16,21]), o(pl![19,8; 19,14]), o(pl![16,11; 22,11]),
    ];
    Group, "group", "people" => vec![
        c(oct(4.0, 4.0, 10.0, 10.0, 2.0)), o(pl![1,20; 1,17; 4,13; 10,13; 13,17; 13,20]),
        c(oct(14.0, 4.0, 20.0, 10.0, 2.0)), o(pl![16,13; 19,13; 22,17; 22,20]),
    ];
    AccountCircle, "account_circle", "people" => vec![
        c(oct(2.0, 2.0, 22.0, 22.0, 6.0)), c(oct(9.0, 6.0, 15.0, 12.0, 1.5)), o(pl![6,19; 6,17; 9,14; 15,14; 18,17; 18,19]),
    ];
    ThumbUp, "thumb_up", "people" => vec![c(rect(3.0, 10.0, 7.0, 20.0)), c(pl![7,10; 11,3; 14,3; 14,8; 20,8; 21,10; 19,19; 7,19])];
    ThumbDown, "thumb_down", "people" => vec![c(rect(3.0, 4.0, 7.0, 14.0)), c(pl![7,14; 11,21; 14,21; 14,16; 20,16; 21,14; 19,5; 7,5])];
    Lightbulb, "lightbulb", "objects" => vec![
        c(pl![9,17; 9,15; 6,11; 6,8; 9,4; 15,4; 18,8; 18,11; 15,15; 15,17]), o(pl![9,20; 15,20]), o(pl![10,22; 14,22]),
    ];
    RocketLaunch, "rocket_launch", "objects" => vec![
        c(pl![12,2; 16,6; 16,14; 12,17; 8,14; 8,6]), o(pl![8,11; 4,15; 4,18; 8,16]), o(pl![16,11; 20,15; 20,18; 16,16]),
        c(rect(11.0, 8.0, 13.0, 10.0)), o(pl![11,20; 12,22; 13,20]),
    ];
    Work, "work", "objects" => vec![c(rect(3.0, 7.0, 21.0, 20.0)), o(pl![9,7; 9,4; 15,4; 15,7]), o(pl![3,13; 21,13])];
    Explore, "explore", "objects" => vec![c(oct(2.0, 2.0, 22.0, 22.0, 6.0)), c(pl![15,9; 13,13; 9,15; 11,11])];
    Map, "map", "objects" => vec![c(pl![3,6; 9,4; 15,6; 21,4; 21,18; 15,20; 9,18; 3,20]), o(pl![9,4; 9,18]), o(pl![15,6; 15,20])];
    DirectionsCar, "directions_car", "objects" => vec![
        c(pl![3,13; 5,7; 19,7; 21,13; 21,18; 3,18]), o(pl![3,13; 21,13]), c(rect(5.0, 18.0, 8.0, 21.0)), c(rect(16.0, 18.0, 19.0, 21.0)),
    ];
    Flight, "flight", "objects" => vec![
        c(pl![12,2; 14,9; 22,14; 22,16; 14,14; 13,19; 16,21; 16,22; 12,21; 8,22; 8,21; 11,19; 10,14; 2,16; 2,14; 10,9]),
    ];
    Restaurant, "restaurant", "objects" => vec![
        o(pl![4,3; 4,9; 7,12; 10,9; 10,3]), o(pl![7,3; 7,21]), c(pl![16,3; 19,6; 19,12; 16,12]), o(pl![16,12; 16,21]),
    ];
    Coffee, "coffee", "objects" => vec![
        c(pl![4,8; 17,8; 17,16; 14,20; 7,20; 4,16]), o(pl![17,10; 20,10; 20,14; 17,14]), o(pl![8,3; 8,6]), o(pl![12,3; 12,6]),
    ];
    WaterDrop, "water_drop", "objects" => vec![c(pl![12,2; 18,10; 18,16; 15,20; 9,20; 6,16; 6,10])];
    AcUnit, "ac_unit", "objects" => vec![o(pl![12,2; 12,22]), o(pl![2,12; 22,12]), o(pl![5,5; 19,19]), o(pl![19,5; 5,19])];
    // interface
    DoneAll, "done_all", "interface" => vec![o(pl![2,13; 6,17; 15,6]), o(pl![10,16; 12,18; 22,6])];
    ArrowDropDown, "arrow_drop_down", "interface" => vec![c(pl![7,10; 17,10; 12,15])];
    ArrowDropUp, "arrow_drop_up", "interface" => vec![c(pl![7,14; 17,14; 12,9])];
    FirstPage, "first_page", "interface" => vec![o(pl![18,6; 12,12; 18,18]), o(pl![6,5; 6,19])];
    LastPage, "last_page", "interface" => vec![o(pl![6,6; 12,12; 6,18]), o(pl![18,5; 18,19])];
    OpenInNew, "open_in_new", "interface" => vec![
        o(pl![10,6; 5,6; 5,19; 18,19; 18,14]), o(pl![13,4; 20,4; 20,11]), o(pl![20,4; 11,13]),
    ];
    DragIndicator, "drag_indicator", "interface" => vec![
        c(rect(8.0, 5.0, 10.0, 7.0)), c(rect(14.0, 5.0, 16.0, 7.0)), c(rect(8.0, 11.0, 10.0, 13.0)),
        c(rect(14.0, 11.0, 16.0, 13.0)), c(rect(8.0, 17.0, 10.0, 19.0)), c(rect(14.0, 17.0, 16.0, 19.0)),
    ];
    Apps, "apps", "interface" => vec![
        c(rect(4.0, 4.0, 8.0, 8.0)), c(rect(10.0, 4.0, 14.0, 8.0)), c(rect(16.0, 4.0, 20.0, 8.0)),
        c(rect(4.0, 10.0, 8.0, 14.0)), c(rect(10.0, 10.0, 14.0, 14.0)), c(rect(16.0, 10.0, 20.0, 14.0)),
        c(rect(4.0, 16.0, 8.0, 20.0)), c(rect(10.0, 16.0, 14.0, 20.0)), c(rect(16.0, 16.0, 20.0, 20.0)),
    ];
    Dashboard, "dashboard", "interface" => vec![
        c(rect(3.0, 3.0, 11.0, 13.0)), c(rect(13.0, 3.0, 21.0, 9.0)), c(rect(3.0, 15.0, 11.0, 21.0)), c(rect(13.0, 11.0, 21.0, 21.0)),
    ];
    ViewList, "view_list", "interface" => vec![
        c(rect(3.0, 5.0, 6.0, 8.0)), c(rect(3.0, 10.5, 6.0, 13.5)), c(rect(3.0, 16.0, 6.0, 19.0)),
        c(rect(9.0, 5.0, 21.0, 8.0)), c(rect(9.0, 10.5, 21.0, 13.5)), c(rect(9.0, 16.0, 21.0, 19.0)),
    ];
    SwapHoriz, "swap_horiz", "interface" => vec![o(pl![4,8; 20,8]), o(pl![16,4; 20,8; 16,12]), o(pl![20,16; 4,16]), o(pl![8,12; 4,16; 8,20])];
    SwapVert, "swap_vert", "interface" => vec![o(pl![8,4; 8,20]), o(pl![4,8; 8,4; 12,8]), o(pl![16,20; 16,4]), o(pl![12,16; 16,20; 20,16])];
    History, "history", "interface" => vec![
        o(pl![4,13; 4,16; 8,20; 16,20; 20,16; 20,8; 16,4; 8,4; 4,8]), o(pl![1,10; 4,13; 7,10]), o(pl![12,8; 12,12; 15,14]),
    ];
    Logout, "logout", "interface" => vec![o(pl![10,4; 4,4; 4,20; 10,20]), o(pl![9,12; 20,12]), o(pl![16,8; 20,12; 16,16])];
    Login, "login", "interface" => vec![o(pl![14,4; 20,4; 20,20; 14,20]), o(pl![4,12; 15,12]), o(pl![11,8; 15,12; 11,16])];
    Report, "report", "status" => vec![
        c(pl![8,3; 16,3; 21,8; 21,16; 16,21; 8,21; 3,16; 3,8]), o(pl![12,7; 12,13]), o(pl![12,16; 12,17]),
    ];
    Block, "block", "status" => vec![c(oct(3.0, 3.0, 21.0, 21.0, 6.0)), o(pl![7,7; 17,17])];
    AddCircle, "add_circle", "status" => vec![c(oct(3.0, 3.0, 21.0, 21.0, 6.0)), o(pl![12,8; 12,16]), o(pl![8,12; 16,12])];
    NotificationsActive, "notifications_active", "status" => vec![
        c(pl![6,17; 6,10; 9,5; 15,5; 18,10; 18,17]), o(pl![4,17; 20,17]), o(pl![10,20; 14,20]), o(pl![1,9; 3,6]), o(pl![23,9; 21,6]),
    ];
    Label, "label", "interface" => vec![c(pl![3,6; 16,6; 21,12; 16,18; 3,18])];
    Inbox, "inbox", "interface" => vec![
        o(pl![3,13; 6,5; 18,5; 21,13]), c(pl![3,13; 8,13; 9,16; 15,16; 16,13; 21,13; 21,20; 3,20]),
    ];
    Reply, "reply", "interface" => vec![o(pl![10,5; 3,11; 10,17]), o(pl![3,11; 14,11; 20,14; 20,19])];
    Forward, "forward", "interface" => vec![o(pl![14,5; 21,11; 14,17]), o(pl![21,11; 10,11; 4,14; 4,19])];

    // ── batch 2 ──────────────────────────────────────────────────────────
    // travel
    LocationOn, "location_on", "travel" => vec![c(pl![12,22; 5,12; 5,7; 8,3; 16,3; 19,7; 19,12]), c(ngon(12.0, 9.0, 2.5))];
    NearMe, "near_me", "travel" => vec![c(pl![3,11; 21,3; 13,21; 11,13])];
    DirectionsWalk, "directions_walk", "travel" => vec![
        c(ngon(13.0, 4.0, 2.0)), o(pl![13,8; 11,13; 14,16; 14,21]), o(pl![11,13; 9,21]), o(pl![13,9; 17,12]), o(pl![13,9; 9,12]),
    ];
    DirectionsBike, "directions_bike", "travel" => vec![
        c(ngon(6.0, 16.0, 3.5)), c(ngon(18.0, 16.0, 3.5)), o(pl![6,16; 10,9; 16,9; 18,16]), o(pl![10,9; 12,16; 6,16]),
        o(pl![10,9; 9,6; 7,6]), o(pl![16,9; 15,6; 17,6]),
    ];
    DirectionsBus, "directions_bus", "travel" => vec![
        c(cb(4.0, 3.0, 20.0, 19.0, 3.0)), c(rect(6.0, 5.0, 18.0, 10.0)), o(pl![4,13; 20,13]),
        c(rect(6.0, 19.0, 9.0, 22.0)), c(rect(15.0, 19.0, 18.0, 22.0)),
    ];
    Train, "train", "travel" => vec![
        c(oct(6.0, 2.0, 18.0, 18.0, 3.0)), c(rect(8.0, 5.0, 16.0, 11.0)), o(pl![8,18; 5,22]), o(pl![16,18; 19,22]),
        o(pl![9,14.5; 9,15]), o(pl![15,14.5; 15,15]),
    ];
    LocalGasStation, "local_gas_station", "travel" => vec![
        c(rect(4.0, 4.0, 14.0, 21.0)), o(pl![4,10; 14,10]), o(pl![14,9; 17,9; 19,11; 19,18]), o(pl![3,21; 15,21]),
    ];
    Hotel, "hotel", "travel" => vec![
        o(pl![3,4; 3,20]), o(pl![3,17; 21,17; 21,20]), o(pl![21,17; 21,11; 11,11; 11,17]), c(ngon(7.0, 13.0, 2.0)),
    ];
    Luggage, "luggage", "travel" => vec![
        c(rect(6.0, 6.0, 18.0, 21.0)), o(pl![9,6; 9,3; 15,3; 15,6]), o(pl![10,10; 10,17]), o(pl![14,10; 14,17]),
    ];
    // weather
    Thunderstorm, "thunderstorm", "weather" => vec![
        c(pl![6,14; 3,11; 3,8; 6,6; 8,3; 14,3; 17,5; 20,6; 21,10; 21,14]), o(pl![13,12; 10,17; 14,17; 11,22]),
    ];
    Rainy, "rainy", "weather" => vec![
        c(pl![6,14; 3,11; 3,8; 6,6; 8,3; 14,3; 17,5; 20,6; 21,10; 21,14]), o(pl![8,17; 7,21]), o(pl![12,17; 11,21]), o(pl![16,17; 15,21]),
    ];
    Snowing, "snowing", "weather" => vec![
        c(pl![6,14; 3,11; 3,8; 6,6; 8,3; 14,3; 17,5; 20,6; 21,10; 21,14]),
        c(rect(7.0, 17.0, 9.0, 19.0)), c(rect(11.0, 19.0, 13.0, 21.0)), c(rect(15.0, 17.0, 17.0, 19.0)),
    ];
    Air, "air", "weather" => vec![
        o(pl![3,8; 14,8; 17,5; 20,5; 21,7]), o(pl![3,12; 19,12; 21,14; 20,17; 17,17]), o(pl![3,16; 9,16]),
    ];
    Thermostat, "thermostat", "weather" => vec![
        c(pl![10,3; 14,3; 14,13; 16,15; 16,19; 14,21; 10,21; 8,19; 8,15; 10,13]), c(ngon(12.0, 17.0, 1.5)), o(pl![12,6; 12,15]),
    ];
    // health
    MonitorHeart, "monitor_heart", "health" => vec![c(cb(2.0, 4.0, 22.0, 20.0, 3.0)), o(pl![5,12; 8,12; 10,8; 13,16; 15,12; 19,12])];
    Medication, "medication", "health" => vec![
        c(pl![9,3; 15,3; 18,6; 18,18; 15,21; 9,21; 6,18; 6,6]), o(pl![6,12; 18,12]),
    ];
    FitnessCenter, "fitness_center", "health" => vec![
        o(pl![7,12; 17,12]), c(rect(4.0, 8.0, 7.0, 16.0)), c(rect(17.0, 8.0, 20.0, 16.0)), o(pl![2,10; 2,14]), o(pl![22,10; 22,14]),
    ];
    // finance
    TrendingUp, "trending_up", "finance" => vec![o(pl![3,17; 9,11; 13,15; 21,7]), o(pl![15,7; 21,7; 21,13])];
    TrendingDown, "trending_down", "finance" => vec![o(pl![3,7; 9,13; 13,9; 21,17]), o(pl![15,17; 21,17; 21,11])];
    AccountBalance, "account_balance", "finance" => vec![
        c(pl![3,10; 12,3; 21,10]), o(pl![5,12; 5,18]), o(pl![10,12; 10,18]), o(pl![14,12; 14,18]), o(pl![19,12; 19,18]), o(pl![3,21; 21,21]),
    ];
    AttachMoney, "attach_money", "finance" => vec![
        o(pl![17,7; 15,5; 9,5; 7,7; 7,10; 9,12; 15,12; 17,14; 17,17; 15,19; 9,19; 7,17]), o(pl![12,2; 12,22]),
    ];
    Calculate, "calculate", "finance" => vec![
        c(cb(4.0, 3.0, 20.0, 21.0, 3.0)), o(pl![8,8; 16,8]), o(pl![8,13; 16,13]), o(pl![8,17; 16,17]),
    ];
    Percent, "percent", "finance" => vec![o(pl![19,5; 5,19]), c(ngon(7.0, 7.0, 2.5)), c(ngon(17.0, 17.0, 2.5))];
    // social
    Forum, "forum", "social" => vec![c(pl![3,3; 15,3; 15,11; 8,11; 5,14; 5,11; 3,11]), c(pl![10,14; 21,14; 21,22; 18,19; 10,19])];
    Comment, "comment", "social" => vec![c(pl![4,4; 20,4; 20,16; 11,16; 7,20; 7,16; 4,16])];
    Mood, "mood", "social" => vec![
        c(oct(3.0, 3.0, 21.0, 21.0, 6.0)), o(pl![9,9; 9,10]), o(pl![15,9; 15,10]), o(pl![8,14; 10,16; 14,16; 16,14]),
    ];
    SentimentDissatisfied, "sentiment_dissatisfied", "social" => vec![
        c(oct(3.0, 3.0, 21.0, 21.0, 6.0)), o(pl![9,9; 9,10]), o(pl![15,9; 15,10]), o(pl![8,17; 10,15; 14,15; 16,17]),
    ];
    Cake, "cake", "social" => vec![c(rect(4.0, 12.0, 20.0, 21.0)), o(pl![4,15; 7,17; 10,15; 14,17; 17,15; 20,17]), o(pl![12,12; 12,7]), o(pl![12,4; 12,5])];
    // security
    Fingerprint, "fingerprint", "security" => vec![
        o(pl![5,10; 7,6; 12,4; 17,6; 19,10]), o(pl![8,18; 8,10; 10,8; 14,8; 16,10; 16,14]), o(pl![12,12; 12,20]), o(pl![5,14; 5,17]),
    ];
    Password, "password", "security" => vec![
        c(cb(2.0, 6.0, 22.0, 18.0, 3.0)), o(pl![6,10; 6,14]), o(pl![4,12; 8,12]), o(pl![12,10; 12,14]), o(pl![10,12; 14,12]),
        o(pl![18,10; 18,14]), o(pl![16,12; 20,12]),
    ];
    // developer
    DataObject, "data_object", "dev" => vec![
        o(pl![9,4; 6,4; 5,5; 5,10; 3,12; 5,14; 5,19; 6,20; 9,20]), o(pl![15,4; 18,4; 19,5; 19,10; 21,12; 19,14; 19,19; 18,20; 15,20]),
    ];
    AccountTree, "account_tree", "dev" => vec![
        c(rect(3.0, 9.0, 8.0, 15.0)), c(rect(15.0, 3.0, 21.0, 8.0)), c(rect(15.0, 16.0, 21.0, 21.0)),
        o(pl![8,12; 11,12; 11,5.5; 15,5.5]), o(pl![11,12; 11,18.5; 15,18.5]),
    ];
    Hub, "hub", "dev" => vec![
        c(ngon(12.0, 12.0, 3.0)), c(rect(3.0, 3.0, 6.0, 6.0)), c(rect(18.0, 3.0, 21.0, 6.0)), c(rect(3.0, 18.0, 6.0, 21.0)),
        c(rect(18.0, 18.0, 21.0, 21.0)), o(pl![6,6; 9.5,9.5]), o(pl![18,6; 14.5,9.5]), o(pl![6,18; 9.5,14.5]), o(pl![18,18; 14.5,14.5]),
    ];
    Merge, "merge", "dev" => vec![
        c(ngon(6.0, 5.0, 2.0)), c(ngon(6.0, 19.0, 2.0)), c(ngon(18.0, 16.0, 2.0)), o(pl![6,7; 6,17]), o(pl![6,10; 12,16; 16,16]),
    ];
    CallSplit, "call_split", "dev" => vec![
        o(pl![3,3; 12,12]), o(pl![21,3; 12,12]), o(pl![12,12; 12,21]), o(pl![3,8; 3,3; 8,3]), o(pl![16,3; 21,3; 21,8]),
    ];
    // charts
    ShowChart, "show_chart", "charts" => vec![o(pl![3,17; 9,10; 13,14; 21,6])];
    StackedBarChart, "stacked_bar_chart", "charts" => vec![
        c(rect(5.0, 4.0, 10.0, 20.0)), o(pl![5,12; 10,12]), c(rect(14.0, 9.0, 19.0, 20.0)), o(pl![14,15; 19,15]),
    ];
    DonutLarge, "donut_large", "charts" => vec![c(ngon(12.0, 12.0, 9.0)), c(ngon(12.0, 12.0, 4.0)), o(pl![12,3; 12,8])];
    ScatterPlot, "scatter_plot", "charts" => vec![
        c(rect(4.0, 14.0, 7.0, 17.0)), c(rect(10.0, 8.0, 13.0, 11.0)), c(rect(16.0, 15.0, 19.0, 18.0)),
        c(rect(15.0, 4.0, 18.0, 7.0)), c(rect(5.0, 5.0, 8.0, 8.0)),
    ];
    BubbleChart, "bubble_chart", "charts" => vec![c(ngon(8.0, 16.0, 4.0)), c(ngon(16.0, 9.0, 3.0)), c(ngon(17.0, 18.0, 2.0))];
    CandlestickChart, "candlestick_chart", "charts" => vec![
        o(pl![7,3; 7,7]), o(pl![7,15; 7,21]), c(rect(5.0, 7.0, 9.0, 15.0)),
        o(pl![12,3; 12,10]), o(pl![12,17; 12,21]), c(rect(10.0, 10.0, 14.0, 17.0)),
        o(pl![17,3; 17,6]), o(pl![17,12; 17,19]), c(rect(15.0, 6.0, 19.0, 12.0)),
    ];
    Leaderboard, "leaderboard", "charts" => vec![
        c(rect(3.0, 12.0, 8.0, 21.0)), c(rect(9.5, 4.0, 14.5, 21.0)), c(rect(16.0, 9.0, 21.0, 21.0)),
    ];
    Analytics, "analytics", "charts" => vec![
        c(cb(3.0, 3.0, 21.0, 21.0, 3.0)), o(pl![8,17; 8,13]), o(pl![12,17; 12,8]), o(pl![16,17; 16,11]),
    ];
    TableChart, "table_chart", "charts" => vec![
        c(rect(3.0, 4.0, 21.0, 20.0)), o(pl![3,9; 21,9]), o(pl![3,14; 21,14]), o(pl![9,9; 9,20]), o(pl![15,9; 15,20]),
    ];
    // controls & misc
    EditNote, "edit_note", "edit" => vec![
        o(pl![3,6; 14,6]), o(pl![3,11; 10,11]), o(pl![3,16; 8,16]), c(pl![12,20; 13,16; 19,10; 22,13; 16,19]),
    ];
    Checklist, "checklist", "edit" => vec![
        o(pl![3,6; 5,8; 8,4]), o(pl![3,16; 5,18; 8,14]), o(pl![11,6; 21,6]), o(pl![11,16; 21,16]),
    ];
    TaskAlt, "task_alt", "status" => vec![
        o(pl![21,10; 21,16; 16,21; 8,21; 3,16; 3,8; 8,3; 16,3; 18,4]), o(pl![8,12; 11,15; 21,5]),
    ];
    ToggleOn, "toggle_on", "interface" => vec![c(oct(2.0, 6.0, 22.0, 18.0, 4.0)), c(oct(14.0, 8.0, 20.0, 16.0, 2.0))];
    ToggleOff, "toggle_off", "interface" => vec![c(oct(2.0, 6.0, 22.0, 18.0, 4.0)), c(oct(4.0, 8.0, 10.0, 16.0, 2.0))];
    RadioButtonChecked, "radio_button_checked", "interface" => vec![c(ngon(12.0, 12.0, 9.0)), c(ngon(12.0, 12.0, 4.0))];
    RadioButtonUnchecked, "radio_button_unchecked", "interface" => vec![c(ngon(12.0, 12.0, 9.0))];
    CheckBox, "check_box", "interface" => vec![c(cb(3.0, 3.0, 21.0, 21.0, 3.0)), o(pl![8,12; 11,15; 16,9])];
    CheckBoxOutlineBlank, "check_box_outline_blank", "interface" => vec![c(cb(3.0, 3.0, 21.0, 21.0, 3.0))];
    IndeterminateCheckBox, "indeterminate_check_box", "interface" => vec![c(cb(3.0, 3.0, 21.0, 21.0, 3.0)), o(pl![8,12; 16,12])];
    StarHalf, "star_half", "actions" => vec![c(pl![12,2; 15,9; 22,9; 16,14; 18,21; 12,17; 6,21; 8,14; 2,9; 9,9]), o(pl![12,2; 12,17])];
    LinkOff, "link_off", "actions" => vec![
        o(pl![10,7; 6,7; 3,10; 3,14; 6,17; 10,17]), o(pl![14,7; 18,7; 21,10; 21,14; 18,17; 14,17]), o(pl![3,3; 21,21]),
    ];
    QrCode, "qr_code", "device" => vec![
        c(rect(3.0, 3.0, 10.0, 10.0)), c(rect(14.0, 3.0, 21.0, 10.0)), c(rect(3.0, 14.0, 10.0, 21.0)),
        c(rect(5.5, 5.5, 7.5, 7.5)), c(rect(16.5, 5.5, 18.5, 7.5)), c(rect(5.5, 16.5, 7.5, 18.5)),
        c(rect(14.0, 14.0, 17.0, 17.0)), c(rect(18.0, 18.0, 21.0, 21.0)),
    ];
    Speed, "speed", "status" => vec![
        o(pl![3,18; 3,13; 6,7; 12,4; 18,7; 21,13; 21,18]), o(pl![12,15; 17,9]), c(rect(11.0, 14.0, 13.0, 16.0)),
    ];
    Alarm, "alarm", "status" => vec![c(ngon(12.0, 13.0, 8.0)), o(pl![12,9; 12,13; 15,15]), o(pl![3,5; 6,3]), o(pl![21,5; 18,3])];
    Event, "event", "status" => vec![
        c(rect(3.0, 5.0, 21.0, 21.0)), o(pl![3,10; 21,10]), o(pl![8,3; 8,7]), o(pl![16,3; 16,7]), c(rect(14.0, 13.0, 17.0, 16.0)),
    ];
    PhotoLibrary, "photo_library", "media" => vec![
        c(rect(7.0, 3.0, 21.0, 17.0)), o(pl![3,7; 3,21; 17,21]), o(pl![7,14; 11,10; 14,13; 17,10; 21,14]),
    ];
    Slideshow, "slideshow", "media" => vec![c(rect(3.0, 4.0, 21.0, 20.0)), c(pl![10,8; 16,12; 10,16])];
    Podcasts, "podcasts", "media" => vec![
        c(ngon(12.0, 11.0, 2.0)), o(pl![12,13; 12,21]), o(pl![7,6; 4,9; 4,13; 7,16]), o(pl![17,6; 20,9; 20,13; 17,16]),
    ];
    Speaker, "speaker", "device" => vec![c(cb(5.0, 2.0, 19.0, 22.0, 3.0)), c(ngon(12.0, 15.0, 3.5)), c(rect(11.0, 5.0, 13.0, 7.0))];
    Cast, "cast", "device" => vec![
        o(pl![3,10; 3,6; 21,6; 21,19; 14,19]), o(pl![3,14; 7,18]), o(pl![3,10; 11,18]), c(rect(3.0, 18.0, 5.0, 20.0)),
    ];
    LaptopMac, "laptop_mac", "device" => vec![c(rect(5.0, 5.0, 19.0, 15.0)), c(pl![2,15; 22,15; 20,19; 4,19])];

    // ── batch 3 ──────────────────────────────────────────────────────────
    // text & layout
    FormatSize, "format_size", "edit" => vec![o(pl![3,21; 8,6; 13,21]), o(pl![5,16; 11,16]), o(pl![15,21; 18,12; 21,21]), o(pl![16,18; 20,18])];
    FormatColorFill, "format_color_fill", "edit" => vec![c(pl![4,12; 11,5; 18,12; 11,19]), o(pl![11,5; 11,2]), c(pl![20,15; 18,18; 20,21; 22,18])];
    FormatQuote, "format_quote", "edit" => vec![c(pl![4,6; 10,6; 10,12; 7,18; 4,18]), c(pl![14,6; 20,6; 20,12; 17,18; 14,18])];
    FormatStrikethrough, "format_strikethrough", "edit" => vec![
        o(pl![3,12; 21,12]), o(pl![17,7; 15,5; 9,5; 7,7; 7,9; 9,11]), o(pl![15,13; 17,15; 17,17; 15,19; 9,19; 7,17]),
    ];
    FormatIndentIncrease, "format_indent_increase", "edit" => vec![
        o(pl![3,5; 21,5]), o(pl![11,10; 21,10]), o(pl![11,15; 21,15]), o(pl![3,20; 21,20]), c(pl![3,9; 7,12; 3,15]),
    ];
    FormatLineSpacing, "format_line_spacing", "edit" => vec![
        o(pl![11,6; 21,6]), o(pl![11,12; 21,12]), o(pl![11,18; 21,18]), o(pl![6,5; 6,19]), o(pl![3,8; 6,5; 9,8]), o(pl![3,16; 6,19; 9,16]),
    ];
    WrapText, "wrap_text", "edit" => vec![
        o(pl![3,6; 21,6]), o(pl![3,12; 17,12; 19,14; 19,16; 17,18; 12,18]), o(pl![14,15; 12,18; 14,21]), o(pl![3,18; 8,18]),
    ];
    TableRows, "table_rows", "layout" => vec![c(rect(3.0, 4.0, 21.0, 20.0)), o(pl![3,9; 21,9]), o(pl![3,14; 21,14])];
    ViewColumn, "view_column", "layout" => vec![c(rect(3.0, 4.0, 21.0, 20.0)), o(pl![9,4; 9,20]), o(pl![15,4; 15,20])];
    ViewAgenda, "view_agenda", "layout" => vec![c(rect(3.0, 4.0, 21.0, 10.0)), c(rect(3.0, 14.0, 21.0, 20.0))];
    ViewModule, "view_module", "layout" => vec![
        c(rect(3.0, 4.0, 8.0, 10.0)), c(rect(9.5, 4.0, 14.5, 10.0)), c(rect(16.0, 4.0, 21.0, 10.0)),
        c(rect(3.0, 14.0, 8.0, 20.0)), c(rect(9.5, 14.0, 14.5, 20.0)), c(rect(16.0, 14.0, 21.0, 20.0)),
    ];
    SpaceDashboard, "space_dashboard", "layout" => vec![
        c(rect(3.0, 3.0, 10.0, 14.0)), c(rect(14.0, 3.0, 21.0, 9.0)), c(rect(3.0, 17.0, 10.0, 21.0)), c(rect(14.0, 13.0, 21.0, 21.0)),
    ];
    SelectAll, "select_all", "layout" => vec![c(rect(3.0, 3.0, 21.0, 21.0)), c(rect(8.0, 8.0, 16.0, 16.0))];
    FilterNone, "filter_none", "layout" => vec![c(rect(3.0, 7.0, 17.0, 21.0)), o(pl![7,7; 7,3; 21,3; 21,17; 17,17])];
    AspectRatio, "aspect_ratio", "layout" => vec![c(rect(3.0, 5.0, 21.0, 19.0)), o(pl![6,12; 6,16; 10,16]), o(pl![18,12; 18,8; 14,8])];
    // accessibility
    Accessibility, "accessibility", "access" => vec![
        c(ngon(12.0, 4.0, 2.0)), o(pl![4,8; 12,10; 20,8]), o(pl![12,10; 12,15]), o(pl![12,15; 8,21]), o(pl![12,15; 16,21]),
    ];
    Accessible, "accessible", "access" => vec![
        c(ngon(10.0, 4.0, 2.0)), o(pl![10,8; 10,14; 16,14; 19,20]), o(pl![10,11; 15,11]), o(pl![7,12; 5,15; 6,19; 9,21; 13,21]),
    ];
    Hearing, "hearing", "access" => vec![o(pl![6,11; 7,6; 11,3; 16,4; 19,8; 18,12; 15,15; 14,19; 11,21; 8,20; 7,17]), o(pl![10,10; 11,8; 14,8; 15,10; 13,13])];
    Contrast, "contrast", "access" => vec![c(ngon(12.0, 12.0, 9.0)), c(pl![12,3; 18.4,5.6; 21,12; 18.4,18.4; 12,21])];
    // home
    Kitchen, "kitchen", "home" => vec![c(rect(6.0, 2.0, 18.0, 22.0)), o(pl![6,10; 18,10]), o(pl![9,5; 9,8]), o(pl![9,13; 9,17])];
    Chair, "chair", "home" => vec![o(pl![7,12; 7,4; 17,4; 17,12]), c(rect(5.0, 12.0, 19.0, 17.0)), o(pl![7,17; 7,21]), o(pl![17,17; 17,21])];
    Weekend, "weekend", "home" => vec![c(pl![5,9; 19,9; 19,13; 21,13; 21,19; 3,19; 3,13; 5,13]), o(pl![7,16; 17,16])];
    Bathtub, "bathtub", "home" => vec![o(pl![3,12; 21,12]), c(pl![4,12; 5,18; 7,20; 17,20; 19,18; 20,12]), o(pl![6,12; 6,6; 8,4; 10,4; 11,5])];
    DoorFront, "door_front", "home" => vec![
        c(rect(6.0, 3.0, 18.0, 21.0)), o(pl![3,21; 21,21]), o(pl![15,12; 15,13]), c(rect(9.0, 6.0, 15.0, 10.0)),
    ];
    Window, "window", "home" => vec![c(rect(4.0, 3.0, 20.0, 21.0)), o(pl![12,3; 12,21]), o(pl![4,12; 20,12])];
    Garage, "garage", "home" => vec![
        o(pl![3,21; 3,9; 12,3; 21,9; 21,21]), c(rect(7.0, 11.0, 17.0, 21.0)), o(pl![7,14; 17,14]), o(pl![7,17; 17,17]),
    ];
    Microwave, "microwave", "home" => vec![c(rect(3.0, 5.0, 21.0, 19.0)), c(rect(5.0, 7.0, 15.0, 17.0)), o(pl![18,8; 18,9]), o(pl![18,12; 18,16])];
    LocalLaundryService, "local_laundry_service", "home" => vec![c(cb(5.0, 2.0, 19.0, 22.0, 3.0)), c(ngon(12.0, 14.0, 5.0)), o(pl![9,5; 10,5])];
    // design
    Palette, "palette", "design" => vec![
        c(pl![12,3; 18,5; 21,10; 20,16; 16,18; 14,21; 10,21; 5,19; 3,14; 4,8; 8,4]),
        c(rect(8.0, 8.0, 10.0, 10.0)), c(rect(13.0, 7.0, 15.0, 9.0)), c(rect(15.0, 12.0, 17.0, 14.0)),
    ];
    Colorize, "colorize", "design" => vec![c(pl![15,3; 21,9; 18,12; 12,6]), o(pl![13,7; 4,16; 4,20; 8,20; 17,11])];
    Straighten, "straighten", "design" => vec![
        c(rect(2.0, 8.0, 22.0, 16.0)), o(pl![6,8; 6,12]), o(pl![10,8; 10,11]), o(pl![14,8; 14,12]), o(pl![18,8; 18,11]),
    ];
    Architecture, "architecture", "design" => vec![c(ngon(12.0, 4.0, 1.5)), o(pl![12,5; 5,21]), o(pl![12,5; 19,21]), o(pl![8,14; 16,14])];
    Draw, "draw", "design" => vec![c(pl![15,3; 21,9; 9,21; 3,21; 3,15]), o(pl![12,6; 18,12])];
    Polyline, "polyline", "design" => vec![
        c(rect(3.0, 16.0, 7.0, 20.0)), c(rect(10.0, 5.0, 14.0, 9.0)), c(rect(17.0, 13.0, 21.0, 17.0)),
        o(pl![6,16; 11,9]), o(pl![13,9; 18,13]),
    ];
    Flip, "flip", "design" => vec![
        o(pl![12,3; 12,5]), o(pl![12,8; 12,10]), o(pl![12,14; 12,16]), o(pl![12,19; 12,21]),
        c(pl![9,7; 3,17; 9,17]), c(pl![15,7; 21,17; 15,17]),
    ];
    RotateLeft, "rotate_left", "design" => vec![o(pl![14,6; 9,6; 5,10; 5,16; 9,20; 15,20; 19,16; 19,12]), c(pl![11,2; 15,6; 11,10])];
    RotateRight, "rotate_right", "design" => vec![o(pl![10,6; 15,6; 19,10; 19,16; 15,20; 9,20; 5,16; 5,12]), c(pl![13,2; 9,6; 13,10])];
    Opacity, "opacity", "design" => vec![c(pl![12,2; 19,10; 19,16; 16,20; 8,20; 5,16; 5,10]), c(pl![12,8; 15,12; 15,15; 12,17; 9,15; 9,12])];
    Gradient, "gradient", "design" => vec![c(rect(3.0, 3.0, 21.0, 21.0)), o(pl![3,12; 12,3]), o(pl![3,17; 17,3]), o(pl![3,21; 21,3])];
    Texture, "texture", "design" => vec![
        c(rect(3.0, 3.0, 21.0, 21.0)), o(pl![3,9; 9,3]), o(pl![3,15; 15,3]), o(pl![3,21; 21,3]), o(pl![9,21; 21,9]), o(pl![15,21; 21,15]),
    ];
    PanTool, "pan_tool", "design" => vec![
        o(pl![8,12; 8,5]), o(pl![11,12; 11,3]), o(pl![14,12; 14,4]), o(pl![17,12; 17,7]), c(pl![5,12; 19,12; 19,17; 15,21; 9,21; 5,17]),
    ];
    // sports
    SportsSoccer, "sports_soccer", "sports" => vec![
        c(ngon(12.0, 12.0, 9.0)), c(ngon(12.0, 12.0, 3.0)), o(pl![12,9; 12,3]), o(pl![14,14; 18,18]), o(pl![10,14; 6,18]),
    ];
    SportsBasketball, "sports_basketball", "sports" => vec![c(ngon(12.0, 12.0, 9.0)), o(pl![12,3; 12,21]), o(pl![3,12; 21,12])];
    SportsTennis, "sports_tennis", "sports" => vec![c(ngon(12.0, 12.0, 9.0)), o(pl![5,6; 8,9; 8,15; 5,18]), o(pl![19,6; 16,9; 16,15; 19,18])];
    Pool, "pool", "sports" => vec![
        o(pl![3,19; 6,17; 9,19; 12,17; 15,19; 18,17; 21,19]), o(pl![8,4; 8,14]), o(pl![15,4; 15,14]), o(pl![8,8; 15,8]), o(pl![8,11; 15,11]),
    ];
    GolfCourse, "golf_course", "sports" => vec![o(pl![8,21; 8,3]), c(pl![8,3; 16,6; 8,9]), o(pl![3,21; 21,21]), c(ngon(15.0, 18.0, 1.5))];
    // devices & network
    DesktopWindows, "desktop_windows", "device" => vec![c(rect(3.0, 4.0, 21.0, 16.0)), o(pl![8,20; 16,20]), o(pl![12,16; 12,20])];
    Devices, "devices", "device" => vec![c(rect(3.0, 5.0, 17.0, 15.0)), c(rect(15.0, 9.0, 21.0, 19.0)), o(pl![3,18; 12,18])];
    SdCard, "sd_card", "device" => vec![c(pl![6,3; 14,3; 18,7; 18,21; 6,21]), o(pl![9,6; 9,9]), o(pl![12,6; 12,9]), o(pl![15,6; 15,9])];
    SimCard, "sim_card", "device" => vec![c(pl![6,3; 14,3; 18,7; 18,21; 6,21]), c(rect(9.0, 11.0, 15.0, 17.0)), o(pl![9,14; 15,14])];
    Nfc, "nfc", "device" => vec![c(cb(3.0, 3.0, 21.0, 21.0, 3.0)), o(pl![8,16; 8,8; 16,16; 16,8])];
    WifiTethering, "wifi_tethering", "device" => vec![
        c(ngon(12.0, 12.0, 2.0)), o(pl![8,8; 6,12; 8,16]), o(pl![16,8; 18,12; 16,16]), o(pl![5,5; 3,12; 5,19]), o(pl![19,5; 21,12; 19,19]),
    ];
    CellTower, "cell_tower", "device" => vec![
        o(pl![12,10; 7,21]), o(pl![12,10; 17,21]), o(pl![9,17; 15,17]), c(ngon(12.0, 9.0, 1.5)), o(pl![7,5; 5,9; 7,13]), o(pl![17,5; 19,9; 17,13]),
    ];
    SatelliteAlt, "satellite_alt", "device" => vec![c(pl![4,8; 8,4; 11,7; 7,11]), c(pl![13,17; 17,13; 20,16; 16,20]), c(ngon(12.0, 12.0, 2.5)), o(pl![15,4; 20,9])];
    BatteryChargingFull, "battery_charging_full", "device" => vec![c(rect(3.0, 7.0, 19.0, 17.0)), o(pl![21,10; 21,14]), o(pl![11,9; 8,12; 12,12; 9,15])];
    BatteryAlert, "battery_alert", "device" => vec![c(rect(3.0, 7.0, 19.0, 17.0)), o(pl![21,10; 21,14]), o(pl![11,9; 11,13]), o(pl![11,14.5; 11,15])];
    Cable, "cable", "device" => vec![c(rect(7.0, 6.0, 17.0, 12.0)), o(pl![10,2; 10,6]), o(pl![14,2; 14,6]), o(pl![12,12; 12,16; 7,16; 7,22])];
    Lan, "lan", "device" => vec![
        c(rect(9.0, 3.0, 15.0, 8.0)), c(rect(3.0, 16.0, 9.0, 21.0)), c(rect(15.0, 16.0, 21.0, 21.0)), o(pl![12,8; 12,12]), o(pl![6,16; 6,12; 18,12; 18,16]),
    ];
    Scanner, "scanner", "device" => vec![c(rect(3.0, 10.0, 21.0, 18.0)), o(pl![3,14; 21,14]), o(pl![6,6; 18,6])];
    // nature, food & places
    School, "school", "places" => vec![c(pl![12,4; 2,9; 12,14; 22,9]), o(pl![6,11; 6,17; 12,20; 18,17; 18,11]), o(pl![22,9; 22,15])];
    Science, "science", "places" => vec![
        o(pl![9,3; 15,3]), c(pl![10,3; 10,9; 4,19; 5,21; 19,21; 20,19; 14,9; 14,3]), o(pl![7,15; 17,15]),
    ];
    Pets, "pets", "nature" => vec![
        c(ngon(12.0, 16.0, 4.0)), c(ngon(5.0, 11.0, 1.8)), c(ngon(9.0, 6.0, 1.8)), c(ngon(15.0, 6.0, 1.8)), c(ngon(19.0, 11.0, 1.8)),
    ];
    Park, "park", "nature" => vec![c(pl![12,3; 18,11; 15,11; 19,17; 5,17; 9,11; 6,11]), o(pl![12,17; 12,21])];
    Eco, "eco", "nature" => vec![c(pl![4,20; 5,11; 10,5; 20,4; 19,14; 13,19]), o(pl![4,20; 13,11])];
    EvStation, "ev_station", "places" => vec![
        c(rect(4.0, 4.0, 14.0, 21.0)), o(pl![3,21; 15,21]), o(pl![14,9; 17,9; 19,11; 19,17]), o(pl![10,8; 7,13; 11,13; 8,18]),
    ];
    LunchDining, "lunch_dining", "food" => vec![
        c(pl![4,11; 5,7; 9,4; 15,4; 19,7; 20,11]), o(pl![3,15; 21,15]), c(pl![4,18; 20,18; 19,21; 5,21]),
    ];
    LocalPizza, "local_pizza", "food" => vec![
        c(pl![12,21; 3,6; 21,6]), o(pl![3,6; 7,4; 17,4; 21,6]), c(rect(9.0, 9.0, 11.0, 11.0)), c(rect(13.0, 12.0, 15.0, 14.0)),
    ];
    LocalBar, "local_bar", "food" => vec![c(pl![3,4; 21,4; 12,13]), o(pl![12,13; 12,21]), o(pl![7,21; 17,21])];
    WineBar, "wine_bar", "food" => vec![c(pl![7,3; 17,3; 17,8; 14,12; 10,12; 7,8]), o(pl![12,12; 12,20]), o(pl![8,21; 16,21])];
    Icecream, "icecream", "food" => vec![c(pl![7,12; 12,22; 17,12]), c(pl![7,12; 7,8; 10,4; 14,4; 17,8; 17,12])];
    Inventory, "inventory", "commerce" => vec![c(rect(4.0, 9.0, 20.0, 21.0)), c(rect(3.0, 3.0, 21.0, 9.0)), o(pl![10,13; 14,13])];
    Warehouse, "warehouse", "commerce" => vec![
        o(pl![3,21; 3,8; 12,3; 21,8; 21,21]), c(rect(7.0, 12.0, 17.0, 21.0)), o(pl![7,16.5; 17,16.5]),
    ];
    Construction, "construction", "objects" => vec![
        c(pl![3,18; 21,18; 21,15; 19,10; 15,6; 9,6; 5,10; 3,15]), o(pl![12,6; 12,12]), o(pl![9,12; 15,12]),
    ];

    // ── gaps found by auditing common Material names ─────────────────────
    Build, "build", "dev" => vec![c(pl![5,21; 3,19; 12,10; 14,12]), o(pl![14,12; 17,13; 21,9; 19,7; 17,9; 15,7; 17,5; 15,3; 11,7; 12,10])];
    Extension, "extension", "objects" => vec![c(pl![4,8; 9,8; 9,6; 11,4; 13,6; 13,8; 18,8; 18,13; 20,13; 21,15; 20,17; 18,17; 18,21; 4,21])];
    Api, "api", "dev" => vec![c(ngon(6.0, 12.0, 2.0)), c(ngon(18.0, 12.0, 2.0)), c(ngon(12.0, 6.0, 2.0)), o(pl![8,12; 16,12]), o(pl![7,10; 11,7.5]), o(pl![17,10; 13,7.5])];
    Translate, "translate", "edit" => vec![o(pl![3,6; 13,6]), o(pl![8,4; 8,6]), o(pl![5,6; 6,10; 9,13]), o(pl![11,6; 9,10; 5,14]), o(pl![13,20; 17,10; 21,20]), o(pl![14.5,16; 19.5,16])];
    Spellcheck, "spellcheck", "edit" => vec![o(pl![3,16; 6,6; 9,16]), o(pl![4,13; 8,13]), o(pl![12,14; 16,18; 21,10])];
    TextFields, "text_fields", "edit" => vec![o(pl![2,18; 6,6; 10,18]), o(pl![3.5,14; 8.5,14]), o(pl![13,8; 21,8]), o(pl![17,8; 17,18])];
    FormatAlignRight, "format_align_right", "edit" => vec![o(pl![3,5; 21,5]), o(pl![9,9; 21,9]), o(pl![3,13; 21,13]), o(pl![9,17; 21,17]), o(pl![3,21; 21,21])];
    Umbrella, "umbrella", "weather" => vec![c(pl![3,12; 5,7; 9,4; 12,3; 15,4; 19,7; 21,12]), o(pl![12,12; 12,19; 10,21; 8,19])];
    Casino, "casino", "objects" => vec![c(oct(3.0, 3.0, 21.0, 21.0, 3.0)), o(pl![8,8; 8,9]), o(pl![16,8; 16,9]), o(pl![12,12; 12,13]), o(pl![8,16; 8,17]), o(pl![16,16; 16,17])];
    OpenWith, "open_with", "arrows" => vec![o(pl![12,3; 12,21]), o(pl![3,12; 21,12]), o(pl![9,6; 12,3; 15,6]), o(pl![9,18; 12,21; 15,18]), o(pl![6,9; 3,12; 6,15]), o(pl![18,9; 21,12; 18,15])];
    Campaign, "campaign", "social" => vec![c(pl![3,10; 8,10; 15,5; 15,19; 8,14; 3,14]), o(pl![8,14; 9,19; 11,19; 11,16]), o(pl![18,9; 21,8]), o(pl![18,12; 21,12]), o(pl![18,15; 21,16])];
    RssFeed, "rss_feed", "social" => vec![c(ngon(6.0, 18.0, 1.5)), o(pl![4,10; 8,10; 14,16; 14,20]), o(pl![4,4; 10,4; 20,14; 20,20])];
    Radio, "radio", "media" => vec![c(rect(3.0, 8.0, 21.0, 21.0)), o(pl![6,8; 17,3]), c(ngon(9.0, 14.5, 3.0)), o(pl![15,12; 18,12]), o(pl![15,16; 18,16])];
    Subtitles, "subtitles", "media" => vec![c(rect(3.0, 5.0, 21.0, 19.0)), o(pl![6,12; 10,12]), o(pl![12,12; 18,12]), o(pl![6,15; 14,15]), o(pl![16,15; 18,15])];
    Hd, "hd", "media" => vec![c(rect(3.0, 5.0, 21.0, 19.0)), o(pl![7,9; 7,15]), o(pl![7,12; 11,12]), o(pl![11,9; 11,15]), o(pl![14,9; 14,15; 17,15; 18,14; 18,10; 17,9; 14,9])];
    Square, "square", "interface" => vec![c(rect(4.0, 4.0, 20.0, 20.0))];
    Domain, "domain", "places" => vec![c(rect(4.0, 3.0, 14.0, 21.0)), c(rect(14.0, 10.0, 20.0, 21.0)), o(pl![7,7; 11,7]), o(pl![7,11; 11,11]), o(pl![7,15; 11,15]), o(pl![17,14; 17,17])];
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

/// A modifier that derives a variant from any icon: `wifi-off`, `heart-add`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Modifier {
    /// Diagonal strike-through.
    Off,
    /// Small `+` badge, bottom right.
    Add,
    /// Small `-` badge.
    Remove,
    /// Small check-mark badge.
    Check,
    /// Small padlock badge.
    Lock,
    /// The icon inside an octagonal ring.
    Circle,
}

impl Modifier {
    pub const ALL: [Modifier; 6] = [
        Modifier::Off,
        Modifier::Add,
        Modifier::Remove,
        Modifier::Check,
        Modifier::Lock,
        Modifier::Circle,
    ];

    /// The name suffix, without its separator.
    pub fn suffix(self) -> &'static str {
        match self {
            Modifier::Off => "off",
            Modifier::Add => "add",
            Modifier::Remove => "remove",
            Modifier::Check => "check",
            Modifier::Lock => "lock",
            Modifier::Circle => "circle",
        }
    }

    /// Apply the modifier to an icon's shapes (stays on the 24 grid).
    pub fn apply(self, base: Vec<Shape>) -> Vec<Shape> {
        let scale = |shapes: Vec<Shape>, k: f32, cx: f32, cy: f32| -> Vec<Shape> {
            shapes
                .into_iter()
                .map(|(p, closed)| {
                    (
                        p.into_iter()
                            .map(|(x, y)| (cx + (x - 12.0) * k, cy + (y - 12.0) * k))
                            .collect(),
                        closed,
                    )
                })
                .collect()
        };
        match self {
            Modifier::Off => {
                let mut v = base;
                v.push(o(pl![3, 3; 21, 21]));
                v
            }
            Modifier::Circle => {
                let mut v = scale(base, 0.55, 12.0, 12.0);
                v.push(c(ngon(12.0, 12.0, 10.0)));
                v
            }
            badge => {
                // Base shrinks to the top-left; the badge sits bottom right.
                let mut v = scale(base, 0.75, 9.0, 9.0);
                v.extend(match badge {
                    Modifier::Add => vec![o(pl![15, 18.5; 22, 18.5]), o(pl![18.5, 15; 18.5, 22])],
                    Modifier::Remove => vec![o(pl![15, 18.5; 22, 18.5])],
                    Modifier::Check => vec![o(pl![15, 18.5; 17.5, 21; 22, 15.5])],
                    _ => vec![
                        c(rect(16.0, 18.0, 22.0, 22.0)),
                        o(pl![17, 18; 17, 16; 21, 16; 21, 18]),
                    ],
                });
                v
            }
        }
    }
}

/// Split `wifi_off` / `heart-add` into `("wifi", Off)` / `("heart", Add)`.
pub fn split_modifier(name: &str) -> Option<(&str, Modifier)> {
    Modifier::ALL.iter().find_map(|&m| {
        let base = name.strip_suffix(m.suffix())?;
        let base = base.strip_suffix(['-', '_'])?;
        (!base.is_empty()).then_some((base, m))
    })
}

/// Common Material / Lucide names that mean an icon we already draw.
/// Resolved by [`lookup`] after exact names; targets may carry modifiers.
pub const SYNONYMS: &[(&str, &str)] = &[
    ("done", "check"),
    ("clear", "close"),
    ("people", "users"),
    ("email", "mail"),
    ("message", "chat"),
    ("chat_bubble", "chat"),
    ("photo", "image"),
    ("sync", "refresh"),
    ("filter_list", "filter"),
    ("expand_more", "chevron-down"),
    ("expand_less", "chevron-up"),
    ("launch", "external-link"),
    ("place", "map-pin"),
    ("navigation", "near_me"),
    ("public", "globe"),
    ("today", "calendar"),
    ("insert_chart", "bar-chart"),
    ("timeline", "line-chart"),
    ("list", "view_list"),
    ("brightness_high", "sun"),
    ("wb_sunny", "sun"),
    ("storage", "database"),
    ("security", "shield"),
    ("admin_panel_settings", "shield"),
    ("verified", "shield-check"),
    ("vpn_key", "key"),
    ("backup", "cloud_upload"),
    ("update", "history"),
    ("restore", "history"),
    ("cancel", "error"),
    ("error_outline", "error"),
    ("help_outline", "help"),
    ("contact_support", "help"),
    ("info_outline", "info"),
    ("report_problem", "warning"),
    ("laptop", "laptop_mac"),
    ("phone_iphone", "phone"),
    ("volume_mute", "mute"),
    ("grade", "star"),
    ("payment", "credit_card"),
    ("inventory_2", "inventory"),
    ("business", "domain"),
    ("apartment", "domain"),
    ("paid", "attach_money"),
    ("savings", "attach_money"),
    ("compare_arrows", "swap_horiz"),
    ("reorder", "menu"),
    ("drag_handle", "menu"),
    ("touch_app", "pan_tool"),
    ("rate_review", "edit_note"),
    ("question_answer", "forum"),
    ("feedback", "comment"),
    ("announcement", "report"),
    ("collections", "photo_library"),
    ("drafts", "mail"),
    ("markunread", "mail"),
    ("mark_email_read", "mail-check"),
    ("forward_to_inbox", "send"),
    ("move_to_inbox", "inbox"),
    ("reply_all", "reply"),
    ("unarchive", "archive"),
    ("folder_zip", "archive"),
    ("drive_file_move", "folder_open"),
    ("snippet_folder", "folder"),
    ("theaters", "movie"),
    ("audiotrack", "music"),
    ("library_music", "music"),
    ("queue_music", "music"),
    ("lyrics", "music"),
    ("album", "record"),
    ("manage_accounts", "account_circle"),
    ("supervisor_account", "users"),
    ("support_agent", "headphones"),
    ("face", "user"),
    ("badge", "user"),
    ("center_focus_strong", "target"),
    ("zoom_out_map", "maximize"),
    ("closed_caption", "subtitles"),
    ("emoji_emotions", "mood"),
    ("print_disabled", "print-off"),
    ("barcode_scanner", "qr_code"),
    ("qr_code_scanner", "qr_code"),
    ("pending", "timer"),
    ("sync_problem", "refresh"),
];

/// Shapes for `name`: a drawn icon, a [`SYNONYMS`] entry, or either plus [`Modifier`]s
/// (`bell-off`, `heart-add-circle`).
pub fn lookup(name: &str) -> Option<Vec<Shape>> {
    if let Some(i) = IconName::from_name(name) {
        return Some(i.shapes());
    }
    if let Some((_, to)) = SYNONYMS.iter().find(|(k, _)| *k == name) {
        return lookup(to);
    }
    let (base, m) = split_modifier(name)?;
    lookup(base).map(|s| m.apply(s))
}

/// Like [`lookup`], also resolving names that `alias` maps to a drawn name
/// (e.g. a Material spelling such as `arrow_back` -> `arrow-left`), with
/// modifiers on either (`notifications_off`).
pub fn lookup_with(name: &str, alias: &dyn Fn(&str) -> Option<&'static str>) -> Option<Vec<Shape>> {
    if let Some(s) = lookup(name) {
        return Some(s);
    }
    if let Some(s) = alias(name).and_then(lookup) {
        return Some(s);
    }
    let (base, m) = split_modifier(name)?;
    lookup_with(base, alias).map(|s| m.apply(s))
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
                name.chars()
                    .all(|c| c.is_ascii_lowercase() || c == '-' || c == '_'),
                "{name} is not kebab/snake-case"
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

    #[test]
    fn synonyms_resolve_and_do_not_shadow_drawn_icons() {
        for (k, to) in SYNONYMS {
            assert!(
                IconName::from_name(k).is_none(),
                "{k} is drawn; drop the synonym"
            );
            assert!(lookup(to).is_some(), "{k} -> {to} does not resolve");
            assert!(lookup(k).is_some());
        }
    }

    #[test]
    fn modifiers_derive_variants_on_the_grid() {
        assert_eq!(split_modifier("heart-add"), Some(("heart", Modifier::Add)));
        assert_eq!(split_modifier("wifi_off"), Some(("wifi", Modifier::Off)));
        assert_eq!(split_modifier("off"), None);
        assert!(lookup("nope-off").is_none());
        // A drawn name wins over a derived one.
        assert_eq!(lookup("wifi_off").unwrap(), IconName::WifiOff.shapes());
        for base in ["heart", "grid", "home", "cloud"] {
            let n = IconName::from_name(base).unwrap().shapes().len();
            for m in Modifier::ALL {
                let v = lookup(&format!("{base}-{}", m.suffix())).unwrap();
                assert!(v.len() > n.min(1), "{base} {m:?}");
                for (pts, _) in &v {
                    for &(x, y) in pts {
                        assert!(
                            (0.0..=24.0).contains(&x) && (0.0..=24.0).contains(&y),
                            "{base} {m:?}"
                        );
                    }
                }
            }
        }
        // Aliases resolve, with modifiers on either spelling.
        let alias = |n: &str| (n == "arrow_back").then_some("arrow-left");
        assert_eq!(
            lookup_with("arrow_back", &alias).unwrap(),
            IconName::ArrowLeft.shapes()
        );
        assert!(lookup_with("arrow_back_add", &alias).is_some());
        assert!(lookup_with("arrow_fwd", &alias).is_none());
        // Modifiers stack.
        assert!(lookup("heart-add-circle").is_some());
    }
}
