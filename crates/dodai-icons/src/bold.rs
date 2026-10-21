//! Neo-Brutalism's bold drawings (private data module).

use dodai_icon_id::IconId;
use dodai_material_icons::Icon;

#[path = "bold_data.rs"]
mod data;
use data::ALL;

/// The bold drawing of `id`; `filled` picks the `-fill` weight, falling back to the outline.
/// Dodai names are stored under their Dodai spelling (`arrow_drop_down`), the rest under the
/// source's (`address-book`); both are tried.
pub(crate) fn get(id: IconId, filled: bool) -> Option<&'static Icon> {
    let key = id.name();
    let kebab = key.replace('_', "-");
    let pick = |n: &str| {
        filled
            .then(|| dodai_material_icons::find(ALL, &format!("{n}-fill")))
            .flatten()
            .or_else(|| dodai_material_icons::find(ALL, n))
    };
    pick(key).or_else(|| pick(&kebab))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sorted_and_unique() {
        assert!(ALL.windows(2).all(|w| w[0].name < w[1].name));
    }

    #[test]
    fn every_icon_has_geometry() {
        for i in ALL {
            assert!(!i.polygons().is_empty(), "{}", i.name);
            for p in i.polygons() {
                for (x, y) in p {
                    assert!(
                        (-1.0..=25.0).contains(&x) && (-1.0..=25.0).contains(&y),
                        "{}",
                        i.name
                    );
                }
            }
        }
    }

    #[test]
    fn outline_and_fill_exist() {
        for id in [
            IconId::Home,
            IconId::Search,
            IconId::Close,
            IconId::ChevronDown,
        ] {
            assert!(get(id, false).is_some(), "{id:?}");
        }
        assert!(get(IconId::Home, true).unwrap().is_filled());
    }
}
