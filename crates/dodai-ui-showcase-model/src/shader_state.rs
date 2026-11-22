//! The shader playground's state, kept in the showcase's value store: the chosen source
//! (`sh-gen`, an index into [`generators`]), the effect chain (`sh-fx`, ids joined by commas),
//! strength (`sh-int`), pixel size (`sh-px`) and whether the design's colours are used
//! (`sh-themed`). Buttons reach it as [`crate::Action::Press`] keys starting with `sh:`.

use dodai_ui_shaders::{catalog, Kind, ShaderInfo};

use crate::state::{Showcase, Value, Values};

/// Value key: chosen source (index into [`generators`]).
pub const GEN: &str = "sh-gen";
/// Value key: the effect chain, ids joined by `,`.
pub const FX: &str = "sh-fx";
/// Value key: effect strength.
pub const INTENSITY: &str = "sh-int";
/// Value key: device pixels per shader pixel.
pub const PIXEL: &str = "sh-px";
/// Value key: colours from the design system.
pub const THEMED: &str = "sh-themed";
/// Value key of the "add an effect" dropdown (its pick is consumed, never stored).
pub const FX_ADD: &str = "sh-fx-add";

/// Every source, in catalogue order.
pub fn generators() -> Vec<&'static ShaderInfo> {
    catalog()
        .iter()
        .filter(|s| matches!(s.kind, Kind::Generator))
        .collect()
}

/// Every effect, in catalogue order.
pub fn post_effects() -> Vec<&'static ShaderInfo> {
    catalog()
        .iter()
        .filter(|s| matches!(s.kind, Kind::PostFx))
        .collect()
}

impl Values {
    /// The chosen source's id.
    pub fn shader_source(&self) -> Option<&'static str> {
        generators().get(self.index_of(GEN, 0)).map(|s| s.id)
    }

    /// The effect chain's ids, in order.
    pub fn shader_effects(&self) -> Vec<&'static str> {
        match self.0.get(FX) {
            Some(Value::Text(t)) => t
                .split(',')
                .filter_map(|id| dodai_ui_shaders::by_id(id).map(|s| s.id))
                .collect(),
            _ => Vec::new(),
        }
    }

    fn set_effects(&mut self, ids: &[&str]) {
        self.set(FX, Value::Text(ids.join(",")));
    }

    fn number(&self, key: &str, default: f32) -> f32 {
        match self.0.get(key) {
            Some(Value::Num(n)) => *n,
            _ => default,
        }
    }

    /// Effect strength `0..=1`.
    pub fn shader_intensity(&self) -> f32 {
        self.number(INTENSITY, 1.0)
    }

    /// Device pixels per shader pixel.
    pub fn shader_pixel(&self) -> f32 {
        self.number(PIXEL, 2.0)
    }

    /// Whether the design's colours replace the shader's own.
    pub fn shader_themed(&self) -> bool {
        matches!(self.0.get(THEMED), Some(Value::Bool(true)))
    }

    /// A dropdown pick of the "add an effect" list appends that effect once.
    pub(crate) fn add_effect_at(&mut self, i: usize) {
        if let Some(fx) = post_effects().get(i) {
            let mut ids = self.shader_effects();
            if !ids.contains(&fx.id) {
                ids.push(fx.id);
                self.set_effects(&ids);
            }
        }
    }
}

impl Showcase {
    /// A `sh:` button: `use:<id>` picks a source or toggles an effect, `rm:<id>` drops an
    /// effect, `clear`, `int±`, `px±` and `themed`.
    pub(crate) fn shader_press(&mut self, what: &str) {
        let v = &mut self.values;
        if let Some(id) = what.strip_prefix("use:") {
            let Some(info) = dodai_ui_shaders::by_id(id) else {
                return;
            };
            if matches!(info.kind, Kind::Generator) {
                if let Some(at) = generators().iter().position(|s| s.id == id) {
                    v.set(GEN, Value::Index(at));
                }
            } else {
                let mut ids = v.shader_effects();
                if let Some(at) = ids.iter().position(|x| *x == id) {
                    ids.remove(at);
                } else {
                    ids.push(info.id);
                }
                v.set_effects(&ids);
            }
        } else if let Some(id) = what.strip_prefix("rm:") {
            let ids: Vec<_> = v
                .shader_effects()
                .into_iter()
                .filter(|x| *x != id)
                .collect();
            v.set_effects(&ids);
        } else {
            match what {
                "clear" => v.set_effects(&[]),
                "int-" => v.set(INTENSITY, Value::Num((v.shader_intensity() - 0.1).max(0.0))),
                "int+" => v.set(INTENSITY, Value::Num((v.shader_intensity() + 0.1).min(1.0))),
                "px-" => v.set(PIXEL, Value::Num((v.shader_pixel() - 1.0).max(1.0))),
                "px+" => v.set(PIXEL, Value::Num((v.shader_pixel() + 1.0).min(10.0))),
                "themed" => v.set(THEMED, Value::Bool(!v.shader_themed())),
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Action;

    fn press(m: &mut Showcase, k: &str) {
        m.update(Action::Press(format!("sh:{k}")));
    }

    #[test]
    fn using_an_effect_toggles_it_and_clear_empties_the_chain() {
        let mut m = Showcase::new();
        let fx = post_effects();
        let (a, b) = (fx[0].id, fx[1].id);
        press(&mut m, &format!("use:{a}"));
        press(&mut m, &format!("use:{b}"));
        assert_eq!(m.values.shader_effects(), vec![a, b]);
        press(&mut m, &format!("use:{a}"));
        assert_eq!(m.values.shader_effects(), vec![b]);
        press(&mut m, &format!("rm:{b}"));
        assert!(m.values.shader_effects().is_empty());
        press(&mut m, &format!("use:{a}"));
        press(&mut m, "clear");
        assert!(m.values.shader_effects().is_empty());
    }

    #[test]
    fn using_a_source_selects_it() {
        let mut m = Showcase::new();
        let g = generators();
        press(&mut m, &format!("use:{}", g[3].id));
        assert_eq!(m.values.shader_source(), Some(g[3].id));
    }

    #[test]
    fn the_dropdown_adds_each_effect_once() {
        let mut m = Showcase::new();
        m.update(Action::Set(FX_ADD.into(), Value::Index(2)));
        m.update(Action::Set(FX_ADD.into(), Value::Index(2)));
        assert_eq!(m.values.shader_effects().len(), 1);
        assert!(!m.values.0.contains_key(FX_ADD));
    }

    #[test]
    fn the_steppers_stay_in_range() {
        let mut m = Showcase::new();
        for _ in 0..20 {
            press(&mut m, "int+");
            press(&mut m, "px+");
        }
        assert_eq!(m.values.shader_intensity(), 1.0);
        assert_eq!(m.values.shader_pixel(), 10.0);
        for _ in 0..20 {
            press(&mut m, "int-");
            press(&mut m, "px-");
        }
        assert_eq!(m.values.shader_intensity(), 0.0);
        assert_eq!(m.values.shader_pixel(), 1.0);
        press(&mut m, "themed");
        assert!(m.values.shader_themed());
    }
}
