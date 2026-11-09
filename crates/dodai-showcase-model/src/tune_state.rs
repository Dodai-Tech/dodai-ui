//! The app-wide shape tuning of the Shape page (roundness, borders, density, corner cut,
//! button crop), kept in the showcase's values under `tune:` keys so the controls go through
//! the usual `Action::Set` / `Action::Press`. [`crate::Showcase::theme`] applies it, so every
//! component in every design follows, like the gpui showcase's global shape tune.

use dodai_core::{
    BorderTint, ControlColour, CornerStyle, Corners, Crop, GradientMode, ShadowDir, ShapeTune,
    Theme,
};

use crate::state::{Value, Values};

/// Key prefix of every tuning value; a change re-themes the app.
pub const PREFIX: &str = "tune:";
/// Press key of the Reset button: every tuning value goes back to the design's own.
pub const RESET: &str = "tune:reset";
/// Global roundness (`0..=2`, 1 is the design's own).
pub const ROUND: &str = "tune:round";
/// Corner style picker (index into [`STYLES`]).
pub const STYLE: &str = "tune:style";
/// Border width picker (index into [`WIDTHS`]).
pub const WIDTH: &str = "tune:width";
/// Border colour picker (index into `BorderTint::ALL`).
pub const TINT: &str = "tune:tint";
/// Outline all: 0 off, 1 on.
pub const OUTLINE_ALL: &str = "tune:all";
/// Density picker (index into [`DENSITIES`]).
pub const DENSITY: &str = "tune:density";
/// Gradient picker (index into `GradientMode::ALL`).
pub const GRADIENT: &str = "tune:gradient";
/// Hard shadow offset in px.
pub const SHADOW: &str = "tune:shadow";
/// Hard shadow direction (index into `ShadowDir::ALL`).
pub const SHADOW_DIR: &str = "tune:shadow-dir";
/// The control colour (index into `ControlColour::ALL`), also written by the top bar.
pub const COLOUR: &str = "tune:colour";
/// The control colour picker of the Shape page (index into the theme's offered colours).
pub const COLOUR_PICK: &str = "tune:colour-pick";
/// The corner cut (index into [`CORNERS`]; 0 is each design's own).
pub const CORNER: &str = "tune:corners";
/// The corner cut picker of the "sets" row (index into the first [`SETS`] of [`CORNERS`]).
pub const CORNER_SETS: &str = "tune:cut-sets";
/// The corner cut picker of the "single" row (index into the rest of [`CORNERS`]).
pub const CORNER_SINGLES: &str = "tune:cut-singles";
/// The button crop picker: 0 the design's own, 1 none, then the theme's crop ends.
pub const CROP: &str = "tune:crop";

/// The key of the roundness of a group (`RoundnessGroup::ALL[k]`).
pub fn group_key(k: usize) -> String {
    format!("{PREFIX}group{k}")
}

/// Corner styles: the design's own, then each style.
pub const STYLES: [(&str, Option<CornerStyle>); 4] = [
    ("Design", None),
    ("Round", Some(CornerStyle::Round)),
    ("Cut", Some(CornerStyle::Cut)),
    ("Scoop", Some(CornerStyle::Scoop)),
];

/// Border widths: the design's own, then fixed px.
pub const WIDTHS: [(&str, Option<f32>); 5] = [
    ("Default", None),
    ("None", Some(0.0)),
    ("Hairline", Some(1.0)),
    ("Medium", Some(2.0)),
    ("Thick", Some(3.0)),
];

/// Densities: control height and spacing factors.
pub const DENSITIES: [(&str, f32); 3] = [("Compact", 0.8), ("Default", 1.0), ("Roomy", 1.25)];

/// How many entries of [`CORNERS`] are in the "sets" row.
pub const SETS: usize = 5;

/// Corner cuts: the design's own, the sets, then single corners and sides.
pub const CORNERS: [(&str, Option<Corners>); 11] = [
    ("Default", None),
    ("None", Some(Corners::NONE)),
    ("All", Some(Corners::ALL)),
    ("Diagonal", Some(Corners::DIAGONAL)),
    ("Anti-diagonal", Some(Corners::ANTI_DIAGONAL)),
    ("Top left", Some(Corners::TOP_LEFT)),
    ("Top right", Some(Corners::TOP_RIGHT)),
    ("Bottom right", Some(Corners::BOTTOM_RIGHT)),
    ("Bottom left", Some(Corners::BOTTOM_LEFT)),
    ("Left", Some(Corners::LEFT)),
    ("Right", Some(Corners::RIGHT)),
];

fn num(values: &Values, key: &str, default: f32) -> f32 {
    match values.0.get(key) {
        Some(Value::Num(n)) => *n,
        _ => default,
    }
}

/// The shape tune the values describe.
pub fn tune(values: &Values) -> ShapeTune {
    let d = ShapeTune::DEFAULT;
    let idx = |k: &str, default: usize| values.index_of(k, default);
    ShapeTune {
        roundness: num(values, ROUND, d.roundness),
        groups: std::array::from_fn(|k| num(values, &group_key(k), 1.0)),
        border_width: WIDTHS.get(idx(WIDTH, 0)).and_then(|w| w.1),
        tint: BorderTint::ALL.get(idx(TINT, 0)).copied().unwrap_or(d.tint),
        outline_all: idx(OUTLINE_ALL, 0) == 1,
        hard_shadow: match values.0.get(SHADOW) {
            Some(Value::Num(n)) => Some(*n),
            _ => None,
        },
        shadow_dir: ShadowDir::ALL
            .get(idx(SHADOW_DIR, 0))
            .copied()
            .unwrap_or(d.shadow_dir),
        corner_style: STYLES.get(idx(STYLE, 0)).and_then(|s| s.1),
        density: DENSITIES.get(idx(DENSITY, 1)).map_or(1.0, |x| x.1),
        gradient: GradientMode::ALL
            .get(idx(GRADIENT, 1))
            .copied()
            .unwrap_or(d.gradient),
        control_colour: colour(values),
    }
}

/// The control colour in the values.
pub fn colour(values: &Values) -> ControlColour {
    ControlColour::ALL
        .get(values.index_of(COLOUR, 0))
        .copied()
        .unwrap_or(ControlColour::Design)
}

/// Store the control colour.
pub fn set_colour(values: &mut Values, c: ControlColour) {
    let i = ControlColour::ALL.iter().position(|x| *x == c).unwrap_or(0);
    values.set(COLOUR, Value::Index(i));
}

/// The corner cut every component takes (`None`: each design's own).
pub fn corners(values: &Values) -> Option<Corners> {
    CORNERS.get(values.index_of(CORNER, 0)).and_then(|c| c.1)
}

/// `theme` with the corner cut and the tune applied.
pub fn apply(values: &Values, theme: Theme) -> Theme {
    let base = match corners(values) {
        Some(c) => theme.with_corners(c),
        None => theme,
    };
    base.tuned(&tune(values))
}

/// The end every button without its own crop takes: the design's own (cut-corner designs crop
/// the left end), none, or the picked end.
pub fn default_crop(values: &Values, theme: &Theme) -> Option<Crop> {
    match values.index_of(CROP, 0) {
        0 => theme.look().cut_corners.then_some(Crop::Left),
        1 => None,
        i => theme.button_crops().get(i - 2).copied(),
    }
}

/// Run a tuning `Set` that is not stored as is (the pickers that write another key);
/// `false` when `key` is not one.
pub(crate) fn set(values: &mut Values, theme: &Theme, key: &str, i: usize) -> bool {
    match key {
        CORNER_SETS => values.set(CORNER, Value::Index(i)),
        CORNER_SINGLES => values.set(CORNER, Value::Index(SETS + i)),
        COLOUR_PICK => {
            if let Some(c) = ControlColour::offered(theme).get(i) {
                set_colour(values, *c);
            }
        }
        _ => return false,
    }
    true
}

/// Back to every design's own shape (the control colour stays: it lives in the top bar too).
pub(crate) fn reset(values: &mut Values) {
    values
        .0
        .retain(|k, _| !k.starts_with(PREFIX) || k == COLOUR);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_values_is_the_default_tune() {
        assert_eq!(tune(&Values::default()), ShapeTune::DEFAULT);
        let t = Theme::default();
        assert_eq!(apply(&Values::default(), t.clone()), t);
    }

    #[test]
    fn values_reach_the_tune_and_reset_clears_them() {
        let mut v = Values::default();
        v.set(ROUND, Value::Num(0.5));
        v.set(WIDTH, Value::Index(4));
        v.set(DENSITY, Value::Index(2));
        let tn = tune(&v);
        assert_eq!(
            (tn.roundness, tn.border_width, tn.density),
            (0.5, Some(3.0), 1.25)
        );
        reset(&mut v);
        assert_eq!(tune(&v), ShapeTune::DEFAULT);
    }

    #[test]
    fn the_two_corner_rows_write_one_cut() {
        let mut v = Values::default();
        let t = Theme::default();
        assert!(set(&mut v, &t, CORNER_SINGLES, 0));
        assert_eq!(corners(&v), Some(Corners::TOP_LEFT));
        assert!(set(&mut v, &t, CORNER_SETS, 2));
        assert_eq!(corners(&v), Some(Corners::ALL));
        assert!(!set(&mut v, &t, "other", 0));
    }
}
