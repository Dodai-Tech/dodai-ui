//! [`Drivetrain`], what drives the car, and [`PowerFlow`]: where a hybrid's power goes between
//! the engine, the battery and the wheels.

use super::*;

/// What drives the car. A [`RingTach`](crate::dash::RingTach) reads it to show the revs and the
/// fuel, the motor's power and the charge, or both.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Drivetrain {
    /// A petrol or diesel engine alone.
    #[default]
    Combustion,
    /// An engine and an electric motor with its own battery.
    Hybrid,
    /// An electric motor alone.
    Electric,
}

impl Drivetrain {
    /// Every drivetrain, in showcase order.
    pub const ALL: [Self; 3] = [Self::Combustion, Self::Hybrid, Self::Electric];

    /// A short name for a caption or a picker.
    pub fn name(self) -> &'static str {
        match self {
            Self::Combustion => "Combustion",
            Self::Hybrid => "Hybrid",
            Self::Electric => "Electric",
        }
    }

    /// Whether it burns fuel (and so has a tank). Pure.
    pub fn burns_fuel(self) -> bool {
        self != Self::Electric
    }

    /// Whether it has a traction battery. Pure.
    pub fn has_battery(self) -> bool {
        self != Self::Combustion
    }
}

/// The paths power takes in a hybrid at one moment.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Flows {
    /// The engine drives the wheels.
    pub engine_to_wheels: bool,
    /// The battery drives the wheels through the motor.
    pub battery_to_wheels: bool,
    /// The wheels charge the battery while slowing down.
    pub wheels_to_battery: bool,
    /// The engine charges the battery through the motor.
    pub engine_to_battery: bool,
}

/// Where the power goes with the engine at `engine_kw` and the motor at `motor_kw` (negative
/// while it charges the battery): with the engine running a charging motor is fed by the
/// engine, otherwise by the wheels. Pure.
pub fn flows(engine_kw: f32, motor_kw: f32) -> Flows {
    let (e, m) = (pm::finite(engine_kw, 0.0), pm::finite(motor_kw, 0.0));
    let (engine, drive, charge) = (e > 0.5, m > 0.5, m < -0.5);
    Flows {
        engine_to_wheels: engine,
        battery_to_wheels: drive,
        wheels_to_battery: charge && !engine,
        engine_to_battery: charge && engine,
    }
}

/// What a hybrid is doing, as its flows show it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HybridMode {
    /// Nothing flows: stopped, or coasting.
    Ready,
    /// The motor alone drives.
    Electric,
    /// The engine alone drives.
    Engine,
    /// Engine and motor drive together.
    Boost,
    /// The wheels charge the battery.
    Recovering,
    /// The engine drives and charges the battery.
    Charging,
}

impl HybridMode {
    /// The mode `f` shows. Pure.
    pub fn of(f: Flows) -> Self {
        match (f.engine_to_wheels, f.battery_to_wheels) {
            _ if f.wheels_to_battery => Self::Recovering,
            _ if f.engine_to_battery => Self::Charging,
            (true, true) => Self::Boost,
            (true, false) => Self::Engine,
            (false, true) => Self::Electric,
            (false, false) => Self::Ready,
        }
    }

    /// The heading for this mode.
    pub fn name(self) -> &'static str {
        match self {
            Self::Ready => "Ready",
            Self::Electric => "Electric",
            Self::Engine => "Engine",
            Self::Boost => "Boost",
            Self::Recovering => "Recovering",
            Self::Charging => "Charging",
        }
    }
}

/// A hybrid's power flow: the engine, the battery and the wheels, with an arrow lit along each
/// path the power takes, the mode as the heading and the battery's charge.
///
/// Sport draws the three as a diagram in the pod. Every other look shows the engine, the
/// motor and the battery as rows of a [`RowsPod`] in that look, headed with the mode.
#[derive(IntoElement)]
pub struct PowerFlow {
    follow: Follow,
    engine: f32,
    motor: f32,
    charge: f32,
    max_engine: f32,
    max_motor: f32,
    /// The look drawn: the given one, else the design's, settled when rendered.
    look: Look,
    /// The look given with `.look(..)`; without one, the design's own.
    given_look: Option<Look>,
    style: PodStyle,
    layout: crate::look::PodLayout,
    size: f32,
}

impl PowerFlow {
    follow_api!();

    /// The engine at `engine_kw`, the motor at `motor_kw` (negative while it charges) and the
    /// battery at `charge` percent.
    #[track_caller]
    pub fn new(engine_kw: f32, motor_kw: f32, charge: f32) -> Self {
        Self {
            follow: Follow::here(),
            engine: pm::finite(engine_kw, 0.0),
            motor: pm::finite(motor_kw, 0.0),
            charge: pm::percent(charge),
            max_engine: 150.0,
            max_motor: 80.0,
            look: Look::Sport,
            given_look: None,
            style: PodStyle::Ring,
            layout: crate::look::PodLayout::Round,
            size: 300.0,
        }
    }
    /// The layout: the round pod (default), a card, a strip or a column, each in the look's
    /// style (see [`PodLayout`](crate::look::PodLayout)).
    pub fn layout(mut self, layout: crate::look::PodLayout) -> Self {
        self.layout = layout;
        self
    }
    /// The most the engine and the motor give in kW, for the bars (default 150 and 80).
    pub fn limits(mut self, engine_kw: f32, motor_kw: f32) -> Self {
        self.max_engine = pm::finite(engine_kw, 150.0).abs().max(1.0);
        self.max_motor = pm::finite(motor_kw, 80.0).abs().max(1.0);
        self
    }
    /// The look (see [`Look`]); every look but [`Look::Sport`] draws rows of a [`RowsPod`].
    pub fn look(mut self, look: Look) -> Self {
        self.given_look = Some(look);
        self
    }
    /// How the frame is drawn (open ring, closed disc, glow or bezel).
    pub fn style(mut self, style: PodStyle) -> Self {
        self.style = style;
        self
    }
    /// Diameter in px.
    pub fn size(mut self, s: f32) -> Self {
        self.size = pod_size(s);
        self
    }
}

impl RenderOnce for PowerFlow {
    fn render(mut self, w: &mut Window, cx: &mut App) -> impl IntoElement {
        self.look = crate::look::resolve(self.given_look, cx);
        self.engine = pm::finite(follow(w, cx, &self.follow, 0, self.engine), 0.0);
        self.motor = pm::finite(follow(w, cx, &self.follow, 1, self.motor), 0.0);
        self.charge = pm::percent(follow(w, cx, &self.follow, 2, self.charge));
        let f = flows(self.engine, self.motor);
        let mode = HybridMode::of(f);
        let tone = charge_tone(self.charge);
        if self.look != Look::Sport || self.layout != crate::look::PodLayout::Round {
            let motor_tone = if self.motor < 0.0 {
                Tone::Success
            } else {
                Tone::Info
            };
            let rows = vec![
                PodRow::new(Symbol::Engine, format!("{:.0} kW", self.engine.max(0.0)))
                    .bar(self.engine.max(0.0) / self.max_engine, Tone::Off),
                PodRow::new(
                    if self.motor < 0.0 {
                        Symbol::Recuperation
                    } else {
                        Symbol::Hybrid
                    },
                    power_text(self.motor),
                )
                .bar((self.motor.abs() / self.max_motor).min(1.0), motor_tone),
                PodRow::new(Symbol::TractionBattery, format!("{:.0} %", self.charge))
                    .bar(self.charge / 100.0, tone),
            ];
            return RowsPod::new(rows)
                .heading(mode.name())
                .layout(self.layout)
                .look(self.look)
                .style(self.style)
                .size(self.size)
                .into_any_element();
        }
        let t = theme_of(cx);
        let size = self.size;
        let c = size / 2.0;
        let (ink, muted) = (hsla(t.palette.text), hsla(t.palette.text_muted));
        let (drive, regen) = (hsla(t.palette.info), hsla(t.palette.success));
        let idle = scale_alpha(ink, 0.14);
        // The three nodes: engine top left, battery top right, wheels below.
        let (eng, bat, whl) = (
            (c - size * 0.2, c - size * 0.06),
            (c + size * 0.2, c - size * 0.06),
            (c, c + size * 0.2),
        );
        let node = size * 0.075;
        let lw = (size * 0.01).max(1.5);
        // Each path: from, to, lit, colour. A lit path ends in an arrowhead.
        let paths = [
            (eng, whl, f.engine_to_wheels, ink),
            (bat, whl, f.battery_to_wheels, drive),
            (whl, bat, f.wheels_to_battery, regen),
            (eng, bat, f.engine_to_battery, regen),
        ];
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            for (a, b, lit, col) in paths {
                let (dx, dy) = (b.0 - a.0, b.1 - a.1);
                let len = (dx * dx + dy * dy).sqrt().max(1e-3);
                let (ux, uy) = (dx / len, dy / len);
                let p0 = (a.0 + ux * node * 1.3, a.1 + uy * node * 1.3);
                let p1 = (b.0 - ux * node * 1.3, b.1 - uy * node * 1.3);
                if !lit {
                    stroke(w, o, &[p0, p1], lw * 0.6, idle, false);
                    continue;
                }
                stroke(w, o, &[p0, p1], lw, col, false);
                let h = node * 0.5;
                let head = [
                    (p1.0 - ux * h - uy * h * 0.6, p1.1 - uy * h + ux * h * 0.6),
                    p1,
                    (p1.0 - ux * h + uy * h * 0.6, p1.1 - uy * h - ux * h * 0.6),
                ];
                fill(w, o, &head, col);
            }
            for (x, y) in [eng, bat, whl] {
                stroke(
                    w,
                    o,
                    &disc(x, y, node, 48),
                    lw * 0.8,
                    scale_alpha(ink, 0.5),
                    true,
                );
            }
        };
        let icon = |s: Symbol, (x, y): (f32, f32), col: Hsla| {
            div()
                .absolute()
                .left(px(x - node * 0.6))
                .top(px(y - node * 0.6))
                .child(Icon::symbol(s).size(node * 1.2).color(col))
        };
        let label = |(x, y): (f32, f32), text: String, col: Hsla| {
            text_box(x, y + node * 1.25, size * 0.24)
                .text_size(px(size * 0.042))
                .text_color(col)
                .child(text)
        };
        let content = div()
            .absolute()
            .left_0()
            .top_0()
            .size_full()
            .child(
                text_box(c, size * 0.17, size * 0.5)
                    .text_size(px(size * 0.05))
                    .text_color(muted)
                    .child(mode.name().to_uppercase()),
            )
            .child(icon(Symbol::Engine, eng, ink))
            .child(icon(Symbol::TractionBattery, bat, tone.color(&t)))
            .child(icon(Symbol::Tyre, whl, ink))
            .child(label(eng, format!("{:.0} kW", self.engine.max(0.0)), ink))
            .child(label(bat, format!("{:.0} %", self.charge), ink))
            .child(label(
                whl,
                power_text(self.motor),
                if self.motor < 0.0 { regen } else { muted },
            ));
        ring(size, [None, None], self.style, &t, paint, content).into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn power_flows_where_engine_and_motor_send_it() {
        let f = flows(40.0, 20.0);
        assert!(f.engine_to_wheels && f.battery_to_wheels);
        assert_eq!(HybridMode::of(f), HybridMode::Boost);
        assert_eq!(HybridMode::of(flows(0.0, 30.0)), HybridMode::Electric);
        assert_eq!(HybridMode::of(flows(60.0, 0.0)), HybridMode::Engine);
        assert_eq!(HybridMode::of(flows(0.0, 0.0)), HybridMode::Ready);
    }

    #[test]
    fn a_charging_motor_is_fed_by_the_engine_if_it_runs_else_by_the_wheels() {
        let regen = flows(0.0, -25.0);
        assert!(regen.wheels_to_battery && !regen.engine_to_battery);
        assert_eq!(HybridMode::of(regen), HybridMode::Recovering);
        let charge = flows(50.0, -10.0);
        assert!(charge.engine_to_battery && !charge.wheels_to_battery);
        assert_eq!(HybridMode::of(charge), HybridMode::Charging);
        assert_eq!(HybridMode::of(flows(f32::NAN, 0.0)), HybridMode::Ready);
    }

    #[test]
    fn drivetrains_know_their_tank_and_battery() {
        assert!(Drivetrain::Combustion.burns_fuel() && !Drivetrain::Combustion.has_battery());
        assert!(Drivetrain::Hybrid.burns_fuel() && Drivetrain::Hybrid.has_battery());
        assert!(!Drivetrain::Electric.burns_fuel() && Drivetrain::Electric.has_battery());
        assert_eq!(
            Drivetrain::ALL.map(|d| d.name()),
            ["Combustion", "Hybrid", "Electric"]
        );
    }
}
