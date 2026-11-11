//! The look every vehicle instrument shares, so a whole dash can change style at once. A design
//! names its own in [`ChartStyle::vehicle`](crate::ChartStyle::vehicle); an instrument given no
//! look of its own takes that one.

/// How a vehicle instrument is drawn. Gauges, pods, the menu, lamps and gear displays all take
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
    /// A dark clinical dial: big white numerals, fine track rings, a red blade on a hub and a
    /// pill naming the mode. Its track layout floods the scale with a red band.
    Apex,
    /// A bright clinical cluster: a white-faced dial with black numerals and a red blade, beside
    /// dark ring dials with a hairline track, a red marker and a bold figure.
    Daylight,
}

impl Look {
    /// Every look, in the order a gallery shows them.
    pub const ALL: [Self; 11] = [
        Self::Sport,
        Self::Chronograph,
        Self::Vintage,
        Self::HeadsUp,
        Self::Segment,
        Self::Halo,
        Self::GlassCockpit,
        Self::Race,
        Self::Line,
        Self::Apex,
        Self::Daylight,
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
            Self::Apex => "Apex",
            Self::Daylight => "Daylight",
        }
    }
}

impl Look {
    /// The layouts this look draws a gauge in; the first, [`Layout::Dial`], is the default.
    /// Pure.
    pub fn layouts(self) -> &'static [Layout] {
        match self {
            Self::Sport => &[Layout::Dial, Layout::Sweep, Layout::Track, Layout::Ribbed],
            Self::Chronograph => &[
                Layout::Dial,
                Layout::Bezel,
                Layout::Sector,
                Layout::Register,
            ],
            Self::HeadsUp => &[Layout::Dial, Layout::Cut, Layout::Arc, Layout::Tape],
            Self::Vintage => &[Layout::Dial, Layout::Bezel, Layout::Meter, Layout::Ribbon],
            Self::Segment => &[Layout::Dial, Layout::Arc, Layout::Bar, Layout::Column],
            Self::Halo => &[Layout::Dial, Layout::Capsule, Layout::Arc, Layout::Bar],
            Self::GlassCockpit => &[Layout::Dial, Layout::Bar, Layout::Tape, Layout::Column],
            Self::Race => &[Layout::Dial, Layout::Arc, Layout::Bar, Layout::Column],
            Self::Line => &[Layout::Dial, Layout::Arc, Layout::Bar, Layout::Column],
            Self::Apex => &[Layout::Dial, Layout::Band, Layout::Lit, Layout::Fill],
            Self::Daylight => &[Layout::Dial, Layout::Lit, Layout::Ring, Layout::Pointer],
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
    /// A half ring over a large figure: of light in [`Look::Halo`], of hairlines in
    /// [`Look::HeadsUp`] and [`Look::Line`], of segments in [`Look::Segment`] and of blocks in
    /// [`Look::Race`].
    Arc,
    /// A straight bar under a large figure: of light in [`Look::Halo`], a row of segments in
    /// [`Look::Segment`], a hairline in [`Look::Line`], blocks in [`Look::Race`] and an
    /// engine-indication bar in [`Look::GlassCockpit`].
    Bar,
    /// A round dial with a thick band filled to the value, cut into segments at the ticks
    /// ([`Look::Sport`]).
    Sweep,
    /// A round dial with a track of dots lit up to the value, the last one larger, the figure
    /// in the middle and no hand ([`Look::Sport`]).
    Track,
    /// A round dial with concentric grooves crossed by a bright cursor ([`Look::Sport`]).
    Ribbed,
    /// A round dial in a bright bezel: the scale on a dark bezel round a watch dial with a long
    /// centre hand in [`Look::Chronograph`], a chrome-ringed cream gauge without the panel in
    /// [`Look::Vintage`].
    Bezel,
    /// A sector dial: a watch face printed in concentric bands, the scale in the outer one, a
    /// baton hand and the figure in a small window ([`Look::Chronograph`]).
    Sector,
    /// A wide watch case with two registers, the scale and its share, over a readout window
    /// ([`Look::Chronograph`]).
    Register,
    /// A narrow upright bar: a stack of segments in [`Look::Segment`], an engine-indication bar
    /// with a pointer in [`Look::GlassCockpit`], a hairline in [`Look::Line`] and a stack of
    /// blocks in [`Look::Race`].
    Column,
    /// A cut-corner frame with a fan of hairlines lit up to the value and a glowing needle
    /// ([`Look::HeadsUp`]).
    Cut,
    /// A dark dial whose scale is flooded with a thick band in the primary colour, numerals on
    /// it, the figure big at the side ([`Look::Apex`]).
    Band,
    /// A dial with no hand: a lit red arc fills the track ring up to the value, a bead on its
    /// end, the numerals past it dim and the figure in the middle ([`Look::Apex`] and
    /// [`Look::Daylight`]).
    Lit,
    /// The red band without a hand: lit up to the value, the figure big in the middle
    /// ([`Look::Apex`]).
    Fill,
    /// The ring dial with a short hand from a small cap inside the ring, the figure under it
    /// ([`Look::Daylight`]).
    Pointer,
    /// A dark dial with one hairline ring, a marker riding on it, a thin arc up to the value and
    /// the figure bold in the middle ([`Look::Daylight`]).
    Ring,
    /// A stadium frame with a band of light from the primary to the accent colour round the
    /// figure ([`Look::Halo`]).
    Capsule,
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
            Self::Track => "Track",
            Self::Sector => "Sector",
            Self::Ribbed => "Ribbed",
            Self::Bezel => "Bezel",
            Self::Register => "Register",
            Self::Column => "Column",
            Self::Cut => "Cut",
            Self::Capsule => "Capsule",
            Self::Band => "Band",
            Self::Ring => "Ring",
            Self::Lit => "Lit",
            Self::Fill => "Fill",
            Self::Pointer => "Pointer",
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

/// How a pod of readings (energy, a trip, the tyres, the power flow) is laid out. Every
/// [`Look`] draws all four, each in its own style: the round pod, rings, a wide strip and a
/// tall column. The round looks draw the strip and the column as round registers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PodLayout {
    /// The round pod, the readings inside a ring or the look's own face.
    #[default]
    Round,
    /// Rings: every level an open ring, one inside the other, around the readings.
    Rings,
    /// A wide strip: the readings side by side, each over its level bar.
    Strip,
    /// A tall column: the readings stacked, each figure large over its level bar.
    Column,
}

impl PodLayout {
    /// Every pod layout, in the order a gallery shows them.
    pub const ALL: [Self; 4] = [Self::Round, Self::Rings, Self::Strip, Self::Column];

    /// A short name for a gallery caption.
    pub fn name(self) -> &'static str {
        match self {
            Self::Round => "Round",
            Self::Rings => "Rings",
            Self::Strip => "Strip",
            Self::Column => "Column",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_pod_layout_has_its_own_name() {
        let mut names: Vec<_> = PodLayout::ALL.iter().map(|l| l.name()).collect();
        names.dedup();
        assert_eq!(names.len(), 4);
        assert_eq!(PodLayout::default(), PodLayout::Round);
    }

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
    fn every_look_comes_in_four_layouts() {
        for l in Look::ALL {
            let mut ls = l.layouts().to_vec();
            assert_eq!(ls.len(), 4, "{}", l.name());
            ls.dedup();
            assert_eq!(ls.len(), 4, "{} repeats a layout", l.name());
        }
    }
}
