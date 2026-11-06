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
    /// Glow tubes: stacked-digit tubes behind a wire mesh, bargraph tubes, neon bulbs and a
    /// ring of glowing cathodes on a dark chassis.
    Nixie,
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
        Self::Nixie,
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
            Self::Nixie => "Nixie",
        }
    }
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
