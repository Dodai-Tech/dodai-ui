//! The look every vehicle instrument shares, so a whole dash can change style at once.

/// How a vehicle instrument is drawn. Gauges, pods, the menu and the dash clusters all take
/// one; each look keeps the same data and only changes the drawing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Look {
    /// The dash's own ring pods and ring dial.
    #[default]
    Classic,
    /// A watch case: knurled bezel, fine scales, tapered hands and a glass.
    Chrono,
    /// An instrument panel of old: a screwed plate, a round bezel, a warm face, spade hands and a
    /// name plate.
    Vintage,
    /// A heads-up display: rings of dashes, brackets, chamfered frames and segment bars.
    HeadsUp,
    /// A segment display: an LCD module with seven-segment figures, unlit ghost segments and
    /// segment bars.
    Digital,
    /// A modern light display: soft halos, gradient arcs of light, glowing tips and thin type.
    Futuristic,
    /// A glass cockpit: engine-indication arcs with a filled sweep and a boxed readout, soft-key
    /// pages, cyan selections and accent bugs on a dark display.
    Cockpit,
    /// A motorsport data display: a carbon plate with a cut corner and speed stripes, slanted
    /// segment bars that rise like a rev bar, slanted tags and big leaning figures.
    Race,
}

impl Look {
    /// Every look, in the order a gallery shows them.
    pub const ALL: [Self; 8] = [
        Self::Classic,
        Self::Chrono,
        Self::Vintage,
        Self::HeadsUp,
        Self::Digital,
        Self::Futuristic,
        Self::Cockpit,
        Self::Race,
    ];

    /// A short name for a caption or a picker.
    pub fn name(self) -> &'static str {
        match self {
            Self::Classic => "Classic",
            Self::Chrono => "Chrono",
            Self::Vintage => "Vintage",
            Self::HeadsUp => "Heads-up",
            Self::Digital => "Digital",
            Self::Futuristic => "Futuristic",
            Self::Cockpit => "Glass cockpit",
            Self::Race => "Race",
        }
    }
}

/// What a reading past a limit colours in the looks drawn on a screen (heads-up, digital,
/// futuristic and glass cockpit); the others keep their own colours.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AtLimit {
    /// Only the value: its pointer, sweep, bar or figure take the limit's colour.
    #[default]
    Value,
    /// The whole display: its glows, accents, rings and legends take the limit's colour too, so
    /// the exceedance shows at a glance.
    Display,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_look_has_its_own_name() {
        let mut names: Vec<_> = Look::ALL.iter().map(|l| l.name()).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), Look::ALL.len());
        assert_eq!(Look::default(), Look::ALL[0]);
    }
}
