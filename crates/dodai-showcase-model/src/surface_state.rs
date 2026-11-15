//! The app-wide surface (texture and effects) of the Surface effects page, kept in the
//! showcase's values under `sfx:` keys so the controls go through the usual `Action::Set` /
//! `Action::Press`. It starts empty, which means the design's own surface ([`Surface::INHERIT`]):
//! a control or a mode only stores what it changes.

use dodai_core::{
    PressStyle, ResolvedSurface, Surface, SurfaceClass, SurfaceMode, SurfaceTargets, Texture, Theme,
};

use crate::state::{Value, Values};

/// Key prefix of every surface value.
pub const PREFIX: &str = "sfx:";
/// Mode picker: the [`SurfaceMode`]s.
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
            "preset" => store(values, t, t.signature_surface()),
            "frosted" => store(values, t, t.resolve_surface(&Surface::frosted())),
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
    pub field: fn(&mut ResolvedSurface) -> &mut f32,
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

impl Amount {
    /// What the slider does, in one line.
    pub fn hint(&self) -> &'static str {
        let key = self.key.trim_start_matches("sfx:");
        match key.trim_start_matches("container_") {
            "strength" => "How visible the pattern is. 0 hides it.",
            "scale" => "Size of the pattern. 1 is the design's own.",
            "neon" => "Brightness of the lit pattern lines.",
            "clear" => "Keeps the middle clean so labels stay readable.",
            "glitch" => "Short digital tears, now and then.",
            "sheen" => "A slow light sweep across the surface.",
            "flicker" => "Unsteady light, like a failing tube.",
            "bloom" => "A soft halo around bright parts.",
            "pulse" => "Slow breathing of the glow.",
            "chase" => "Light that travels along the border.",
            "edge" => "Light falling in from the borders.",
            "brackets" => "Marks in the four corners.",
            "depth" => "An inner shadow that sinks the surface.",
            "ripple" => "How strong the click ripple plays.",
            "tint" => "A tonal wash over the whole surface.",
            "pop" => "A hard offset block behind the surface.",
            "tape" => "A strip of tape across a corner.",
            "gloss" => "A clear-coat reflection across the top.",
            "strip" => "A thin strip of light.",
            "wash" => "The design's gradient tinting the surface.",
            "rim" => "The outline lit by the gradient, brightest top left.",
            "frost" => "See-through fill over a blurred backdrop. 0 is an opaque fill.",
            _ => "",
        }
    }
}

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

/// Whether nothing of the page was touched: the surface is the design's own.
pub fn untouched(values: &Values) -> bool {
    !values.0.keys().any(|k| k.starts_with(PREFIX))
}

/// The app-wide surface the values describe: the design's own until a control sets something.
pub fn surface(values: &Values, t: &Theme) -> Surface {
    if untouched(values) {
        Surface::INHERIT
    } else {
        Surface::from(mix(values, t))
    }
}

/// The mix the values describe under theme `t` (unset values are the design's own).
pub fn mix(values: &Values, t: &Theme) -> ResolvedSurface {
    let mut f = t.default_surface();
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
pub fn store(values: &mut Values, t: &Theme, fx: ResolvedSurface) {
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

/// The picker index of the mode the values are in (a [`SurfaceMode`] in [`SurfaceMode::ALL`]).
pub fn mode(values: &Values, t: &Theme) -> usize {
    let m = SurfaceMode::of(t, &surface(values, t));
    SurfaceMode::ALL.iter().position(|x| *x == m).unwrap_or(0)
}

/// Applies the mode picker: Design forgets everything set, the others load their surface (keeping
/// where it applies), Custom changes nothing.
pub fn set_mode(values: &mut Values, t: &Theme, i: usize) {
    let Some(mode) = SurfaceMode::ALL.get(i).copied() else {
        return;
    };
    let next = mode.apply(t, &surface(values, t));
    if next == Surface::INHERIT {
        values.0.retain(|k, _| !k.starts_with(PREFIX));
    } else if mode != SurfaceMode::Custom {
        store(values, t, t.resolve_surface(&next));
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
        assert_eq!(mix(&v, &t), t.signature_surface());
    }

    fn dodai() -> Theme {
        dodai_core::themes::by_id("dodai-dark").unwrap()
    }

    #[test]
    fn nothing_set_is_the_designs_own_surface() {
        for t in [cyberpunk(), dodai()] {
            let v = Values::default();
            assert!(untouched(&v));
            assert_eq!(surface(&v, &t), Surface::INHERIT);
            assert_eq!(mix(&v, &t), t.default_surface());
            assert_eq!(SurfaceMode::ALL[mode(&v, &t)], SurfaceMode::Design);
        }
        // Frost is opt-in on every design.
        assert!(mix(&Values::default(), &dodai()).is_none());
        assert!(mix(&Values::default(), &cyberpunk()).is_none());
    }

    #[test]
    fn every_mode_round_trips_and_a_hand_edit_is_custom() {
        for t in [cyberpunk(), dodai()] {
            for (i, want) in SurfaceMode::ALL.iter().enumerate() {
                if *want == SurfaceMode::Custom {
                    continue;
                }
                let mut v = Values::default();
                set_mode(&mut v, &t, i);
                assert_eq!(SurfaceMode::ALL[mode(&v, &t)], *want, "{want:?}");
            }
            let mut v = Values::default();
            set_mode(&mut v, &t, 3);
            assert_eq!(mix(&v, &t), t.signature_surface());
            v.set("sfx:glitch", Value::Num(0.33));
            assert_eq!(SurfaceMode::ALL[mode(&v, &t)], SurfaceMode::Custom);
            // Off keeps where it applies; Design forgets everything.
            set_targets(&mut v, "all");
            set_mode(&mut v, &t, 0);
            let off = mix(&v, &t);
            assert!(off.is_none());
            assert_eq!(off.targets, SurfaceTargets::ALL);
            set_mode(&mut v, &t, 1);
            assert!(untouched(&v));
        }
    }

    #[test]
    fn a_slider_edit_keeps_the_designs_glass() {
        let t = dodai();
        let mut v = Values::default();
        v.set("sfx:glitch", Value::Num(0.4));
        let fx = mix(&v, &t);
        assert_eq!(fx.glitch, 0.4);
        assert_eq!(fx.frost, t.default_surface().frost);
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
