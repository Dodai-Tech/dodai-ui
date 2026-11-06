//! The look every vehicle instrument shares, so a whole dash can change style at once.

/// How a vehicle instrument is drawn. Gauges, pods, the menu and the dash clusters all take
/// one; each look keeps the same data and only changes the drawing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Look {
    /// The sports dash: ribbed ring pods, a ring dial with the road speed in the middle.
    #[default]
    Sport,
    /// A watch case: knurled bezel, fine scales, tapered hands and a glass.
    Chronograph,
    /// An instrument panel of old: a screwed plate, a round bezel, a warm face, spade hands and a
    /// name plate.
    Vintage,
    /// A heads-up display: rings of dashes, brackets, chamfered frames and segment bars.
    HeadsUp,
    /// A segment display: an LCD module with seven-segment figures, unlit ghost segments and
    /// segment bars.
    Segment,
    /// A modern light display: soft halos, gradient arcs of light, glowing tips and thin type.
    Halo,
    /// A glass cockpit: engine-indication arcs with a filled sweep and a boxed readout, soft-key
    /// pages, cyan selections and accent bugs on a dark display.
    GlassCockpit,
    /// A motorsport data display: a carbon plate with a cut corner and speed stripes, slanted
    /// segment bars that rise like a rev bar, slanted tags and big leaning figures.
    Race,
    /// A minimal line display: one hairline ring or line per gauge, a dot riding on it, a thin
    /// arc up to the value, scale end figures only and a large light figure in open space.
    Line,
}

impl Look {
    /// Every look, in the order a gallery shows them.
    pub const ALL: [Self; 9] = [
        Self::Sport,
        Self::Chronograph,
        Self::Vintage,
        Self::HeadsUp,
        Self::Segment,
        Self::Halo,
        Self::GlassCockpit,
        Self::Race,
        Self::Line,
    ];

    /// A short name for a caption or a picker.
    pub fn name(self) -> &'static str {
        match self {
            Self::Sport => "Sport",
            Self::Chronograph => "Chronograph",
            Self::Vintage => "Vintage",
            Self::HeadsUp => "Heads-up",
            Self::Segment => "Segment LCD",
            Self::Halo => "Halo",
            Self::GlassCockpit => "Glass cockpit",
            Self::Race => "Race",
            Self::Line => "Line",
        }
    }
}

impl Look {
    /// The layouts this look draws a gauge in; the first, [`Layout::Dial`], is the default.
    /// Pure.
    pub fn layouts(self) -> &'static [Layout] {
        match self {
            Self::Sport => &[Layout::Dial, Layout::Sweep, Layout::Needle, Layout::Ribbed],
            Self::Chronograph => &[Layout::Dial, Layout::Bezel, Layout::Register],
            Self::HeadsUp => &[Layout::Dial, Layout::Tape],
            Self::Vintage => &[Layout::Dial, Layout::Meter, Layout::Ribbon],
            Self::Segment => &[Layout::Dial, Layout::Bar, Layout::Column],
            Self::Halo => &[Layout::Dial, Layout::Arc, Layout::Bar],
            Self::GlassCockpit => &[Layout::Dial, Layout::Tape, Layout::Column],
            Self::Race => &[Layout::Dial],
            Self::Line => &[Layout::Dial, Layout::Bar],
        }
    }

    /// `layout` if this look has it, else [`Layout::Dial`]. Pure.
    pub fn fit(self, layout: Layout) -> Layout {
        if self.layouts().contains(&layout) {
            layout
        } else {
            Layout::Dial
        }
    }

    /// The proportions of this look's face in `layout` (after [`Look::fit`]). Pure.
    pub(crate) fn shape(self, layout: Layout) -> Shape {
        match (self, self.fit(layout)) {
            (Self::HeadsUp, Layout::Tape)
            | (_, Layout::Dial | Layout::Bezel | Layout::Sweep | Layout::Needle | Layout::Ribbed) => {
                Shape::Square
            }
            (_, Layout::Tape | Layout::Column) => Shape::Tall,
            _ => Shape::Wide,
        }
    }
}

/// The proportions of a face: square as a dial, wider than tall, or taller than wide.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Shape {
    /// As wide as tall (heads-up tapes fill the square too).
    Square,
    /// As wide as the size, flatter than a dial.
    Wide,
    /// As tall as the size and under half as wide.
    Tall,
}

/// How a gauge is laid out within its [`Look`]; a look draws only the layouts it lists in
/// [`Look::layouts`] and falls back to the dial for the others.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Layout {
    /// A round dial: every look has one.
    #[default]
    Dial,
    /// A vertical tape scrolling under a pointer box ([`Look::HeadsUp`], and a narrow
    /// airspeed-style tape in [`Look::GlassCockpit`]).
    Tape,
    /// A panel meter: a wide window with a flat arc and a needle from a pivot below it
    /// ([`Look::Vintage`]).
    Meter,
    /// A ribbon speedometer: a long window where a coloured ribbon runs out along a straight
    /// scale ([`Look::Vintage`]).
    Ribbon,
    /// A half ring of light over a large figure ([`Look::Halo`]).
    Arc,
    /// A straight bar under a large figure: of light in [`Look::Halo`], a row of segments in
    /// [`Look::Segment`].
    Bar,
    /// A round dial with a thick band filled to the value, cut into segments at the ticks
    /// ([`Look::Sport`]).
    Sweep,
    /// A round dial with a long centre pointer and the figure in a tab below it
    /// ([`Look::Sport`]).
    Needle,
    /// A round dial with concentric grooves crossed by a bright cursor ([`Look::Sport`]).
    Ribbed,
    /// A watch dial with the scale on a dark bezel round it and a long centre hand
    /// ([`Look::Chronograph`]).
    Bezel,
    /// A wide watch case with two registers, the scale and its share, over a readout window
    /// ([`Look::Chronograph`]).
    Register,
    /// A narrow upright bar: a stack of segments in [`Look::Segment`], an engine-indication bar
    /// with a pointer in [`Look::GlassCockpit`].
    Column,
}

impl Layout {
    /// A short name for a caption or a picker.
    pub fn name(self) -> &'static str {
        match self {
            Self::Dial => "Dial",
            Self::Tape => "Tape",
            Self::Meter => "Meter",
            Self::Ribbon => "Ribbon",
            Self::Arc => "Arc",
            Self::Bar => "Bar",
            Self::Sweep => "Sweep",
            Self::Needle => "Needle",
            Self::Ribbed => "Ribbed",
            Self::Bezel => "Bezel",
            Self::Register => "Register",
            Self::Column => "Column",
        }
    }
}

/// What a reading past a limit colours in the looks drawn on a screen (heads-up, segment LCD,
/// halo, glass cockpit and line); the others keep their own colours.
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

    #[test]
    fn every_look_has_a_dial_first_and_falls_back_to_it() {
        for l in Look::ALL {
            assert_eq!(l.layouts()[0], Layout::Dial);
            assert_eq!(l.fit(Layout::Dial), Layout::Dial);
        }
        assert_eq!(Look::Halo.fit(Layout::Bar), Layout::Bar);
        assert_eq!(Look::Halo.fit(Layout::Tape), Layout::Dial);
        assert_eq!(Look::Sport.fit(Layout::Meter), Layout::Dial);
        assert_eq!(Look::Vintage.fit(Layout::Ribbon), Layout::Ribbon);
    }

    #[test]
    fn tapes_and_columns_stand_upright_and_the_rest_lie_flat() {
        assert_eq!(Look::HeadsUp.shape(Layout::Tape), Shape::Square);
        assert_eq!(Look::GlassCockpit.shape(Layout::Tape), Shape::Tall);
        assert_eq!(Look::Segment.shape(Layout::Column), Shape::Tall);
        assert_eq!(Look::Chronograph.shape(Layout::Bezel), Shape::Square);
        assert_eq!(Look::Chronograph.shape(Layout::Register), Shape::Wide);
        assert_eq!(Look::Sport.shape(Layout::Needle), Shape::Square);
        assert_eq!(Look::Halo.shape(Layout::Bar), Shape::Wide);
        assert_eq!(
            Look::Race.shape(Layout::Bar),
            Shape::Square,
            "falls back to the dial"
        );
    }
}
