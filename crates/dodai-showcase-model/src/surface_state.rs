//! The app-wide surface mix (texture and effects) of the Surface effects page, kept in the
//! showcase's values under `sfx:` keys so the controls go through the usual `Action::Set` /
//! `Action::Press`. It starts empty, like the gpui showcase: nothing is drawn until a control
//! or the design's preset turns something on.

use dodai_core::{PressStyle, SurfaceClass, SurfaceFx, SurfaceTargets, Texture, Theme};

use crate::state::{Value, Values};

/// Key prefix of every surface-mix value.
pub const PREFIX: &str = "sfx:";
/// Mode picker: Off / Default (the design's preset) / Custom.
pub const MODE: &str = "sfx:mode";
/// Component texture picker (index into [`component_textures`]).
pub const TEXTURE: &str = "sfx:texture";
/// Container texture picker (index into [`container_textures`]).
pub const CONTAINER_TEXTURE: &str = "sfx:container_texture";
/// Ripple style picker (index into `PressStyle::ALL`).
pub const PRESS: &str = "sfx:press";
/// Backdrop of the frost demo: 0 the design's canvas, 1 [`dodai_core::glass_picture`].
pub const WALLPAPER: &str = "sfx-wallpaper";

/// Whether the frost demo put the glass picture behind the app.
pub fn wallpaper(values: &Values) -> bool {
    matches!(values.0.get(WALLPAPER), Some(Value::Index(1)))
}

/// Test aid (`DODAI_SFX=frost,picture,preset` in the showcases): `preset` loads the design's
/// mix, `picture` puts the glass picture behind the app, any other word sets that slider of the
/// mix (by field name, e.g. `frost`) to its upper bound.
pub fn apply_spec(values: &mut Values, t: &Theme, spec: &str) {
    for word in spec.split(',').map(str::trim) {
        match word {
            "preset" => store(values, t, t.surface_preset()),
            "picture" => values.set(WALLPAPER, Value::Index(1)),
            w => {
                if let Some(a) = AMOUNTS.iter().find(|a| a.key[PREFIX.len()..] == *w) {
                    values.set(a.key, Value::Num(a.max));
                }
            }
        }
    }
}

/// A slider of the mix: value key, label, upper bound and the field it sets.
pub struct Amount {
    /// Value key.
    pub key: &'static str,
    /// Label.
    pub name: &'static str,
    /// Upper bound (the lower is 0).
    pub max: f32,
    /// The field.
    pub field: fn(&mut SurfaceFx) -> &mut f32,
}

macro_rules! amount {
    ($field:ident, $name:expr, $max:expr) => {
        Amount {
            key: concat!("sfx:", stringify!($field)),
            name: $name,
            max: $max,
            field: |f| &mut f.$field,
        }
    };
}

/// Every slider of the page, by field.
pub const AMOUNTS: [Amount; 24] = [
    amount!(strength, "Strength", 1.0),
    amount!(scale, "Scale", 4.0),
    amount!(neon, "Lit lines", 1.0),
    amount!(clear, "Clear centre", 1.0),
    amount!(container_strength, "Strength", 1.0),
    amount!(container_scale, "Scale", 4.0),
    amount!(frost, "Frost", 1.0),
    amount!(glitch, "Glitch", 1.0),
    amount!(sheen, "Sheen", 1.0),
    amount!(flicker, "Flicker", 1.0),
    amount!(bloom, "Bloom", 1.0),
    amount!(pulse, "Pulse", 1.0),
    amount!(chase, "Chase", 1.0),
    amount!(edge, "Edge light", 1.0),
    amount!(brackets, "Brackets", 1.0),
    amount!(depth, "Depth", 1.0),
    amount!(ripple, "Strength", 1.0),
    amount!(tint, "Tonal wash", 1.0),
    amount!(pop, "Pop", 1.0),
    amount!(tape, "Tape", 1.0),
    amount!(gloss, "Gloss", 1.0),
    amount!(strip, "Light strip", 1.0),
    amount!(wash, "Gradient wash", 1.0),
    amount!(rim, "Gradient rim", 1.0),
];

/// The slider of `key`.
pub fn amount(key: &str) -> Option<&'static Amount> {
    AMOUNTS.iter().find(|a| a.key == key)
}

/// Textures a design offers for components (buttons, inputs, tags).
pub fn component_textures(t: &Theme) -> Vec<Texture> {
    let owned = t.surface_textures();
    Texture::ALL
        .iter()
        .copied()
        .filter(|x| x.for_components() && owned.contains(x))
        .collect()
}

/// Textures a design offers for containers (cards, panels, bars).
pub fn container_textures(t: &Theme) -> Vec<Texture> {
    let owned = t.surface_textures();
    Texture::ALL
        .iter()
        .copied()
        .filter(|x| owned.contains(x))
        .collect()
}

/// Value key of the "apply to" switch of a surface class.
pub fn target_key(c: SurfaceClass) -> String {
    format!("sfx:target:{}", c.label())
}

/// The mix the values describe under theme `t` (unset values are [`SurfaceFx::NONE`]'s).
pub fn mix(values: &Values, t: &Theme) -> SurfaceFx {
    let mut f = SurfaceFx::NONE;
    for a in &AMOUNTS {
        if let Some(Value::Num(v)) = values.0.get(a.key) {
            *(a.field)(&mut f) = *v;
        }
    }
    let pick = |key: &str| match values.0.get(key) {
        Some(Value::Index(i)) => Some(*i),
        _ => None,
    };
    if let Some(i) = pick(TEXTURE) {
        f.texture = component_textures(t)
            .get(i)
            .copied()
            .unwrap_or(Texture::None);
    }
    if let Some(i) = pick(CONTAINER_TEXTURE) {
        f.container_texture = container_textures(t)
            .get(i)
            .copied()
            .unwrap_or(Texture::None);
    }
    if let Some(i) = pick(PRESS) {
        f.press = PressStyle::ALL.get(i).copied().unwrap_or_default();
    }
    f.targets = SurfaceClass::ALL
        .iter()
        .fold(SurfaceTargets::DEFAULT, |s, &c| {
            match values.0.get(&target_key(c)) {
                Some(Value::Bool(on)) => s.with(c, *on),
                _ => s,
            }
        });
    f
}

/// Writes `fx` into the values (every field, so the sliders show it).
pub fn store(values: &mut Values, t: &Theme, fx: SurfaceFx) {
    let mut f = fx;
    for a in &AMOUNTS {
        values.set(a.key, Value::Num(*(a.field)(&mut f)));
    }
    let index = |list: Vec<Texture>, x: Texture| list.iter().position(|y| *y == x);
    match index(component_textures(t), fx.texture) {
        Some(i) => values.set(TEXTURE, Value::Index(i)),
        None => values.0.remove(TEXTURE).map_or((), |_| ()),
    }
    match index(container_textures(t), fx.container_texture) {
        Some(i) => values.set(CONTAINER_TEXTURE, Value::Index(i)),
        None => values.0.remove(CONTAINER_TEXTURE).map_or((), |_| ()),
    }
    let press = PressStyle::ALL
        .iter()
        .position(|p| *p == fx.press)
        .unwrap_or(0);
    values.set(PRESS, Value::Index(press));
    for c in SurfaceClass::ALL {
        values.set(target_key(c), Value::Bool(fx.targets.contains(c)));
    }
}

/// The mode the mix is in: 0 Off (nothing drawn), 1 the design's preset, 2 custom.
pub fn mode(fx: &SurfaceFx, t: &Theme) -> usize {
    if fx.is_none() {
        0
    } else if *fx == t.surface_preset() {
        1
    } else {
        2
    }
}

/// Applies the mode picker: Off clears the mix (keeping where it applies), Default loads the
/// design's preset, Custom changes nothing.
pub fn set_mode(values: &mut Values, t: &Theme, i: usize) {
    let targets = mix(values, t).targets;
    match i {
        0 => store(
            values,
            t,
            SurfaceFx {
                targets,
                ..SurfaceFx::NONE
            },
        ),
        1 => store(values, t, t.surface_preset()),
        _ => {}
    }
}

/// The "apply to" shortcuts: `default`, `all` or `none`.
pub fn set_targets(values: &mut Values, to: &str) {
    let set = match to {
        "all" => SurfaceTargets::ALL,
        "none" => SurfaceTargets::NONE,
        _ => SurfaceTargets::DEFAULT,
    };
    for c in SurfaceClass::ALL {
        values.set(target_key(c), Value::Bool(set.contains(c)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cyberpunk() -> Theme {
        dodai_core::themes::all()
            .into_iter()
            .find(|t| t.meta.family == "cyberpunk")
            .unwrap()
    }

    #[test]
    fn the_test_spec_sets_sliders_the_preset_and_the_picture() {
        let t = cyberpunk();
        let mut v = Values::default();
        assert!(!wallpaper(&v));
        apply_spec(&mut v, &t, "frost, picture, nonsense");
        assert_eq!(mix(&v, &t).frost, amount("sfx:frost").unwrap().max);
        assert!(wallpaper(&v));
        apply_spec(&mut v, &t, "preset");
        assert_eq!(mix(&v, &t), t.surface_preset());
    }

    #[test]
    fn nothing_set_is_no_mix_and_off() {
        let t = cyberpunk();
        let v = Values::default();
        let f = mix(&v, &t);
        assert!(f.is_none());
        assert_eq!(mode(&f, &t), 0);
    }

    #[test]
    fn the_preset_round_trips_and_reads_as_default() {
        let t = cyberpunk();
        let mut v = Values::default();
        set_mode(&mut v, &t, 1);
        let f = mix(&v, &t);
        assert_eq!(f, t.surface_preset());
        assert_eq!(mode(&f, &t), 1);
        // A hand edit makes it custom; Off clears it but keeps the targets.
        v.set("sfx:glitch", Value::Num(0.33));
        assert_eq!(mode(&mix(&v, &t), &t), 2);
        set_targets(&mut v, "all");
        set_mode(&mut v, &t, 0);
        let off = mix(&v, &t);
        assert!(off.is_none());
        assert_eq!(off.targets, SurfaceTargets::ALL);
    }

    #[test]
    fn every_amount_key_is_unique_and_found() {
        for a in &AMOUNTS {
            assert_eq!(amount(a.key).map(|b| b.key), Some(a.key));
            assert_eq!(AMOUNTS.iter().filter(|b| b.key == a.key).count(), 1);
        }
        assert!(amount("sfx:nope").is_none());
    }
}
