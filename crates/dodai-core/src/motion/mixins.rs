//! Mixins: reusable behaviours any component can adopt, toolkit-independent.
//!
//! Two kinds:
//! * **Reactive** mixins ([`Reaction`]) follow a held state (hover, press, focus): a spring
//!   between rest and a target [`Pose`], so release and re-press keep their momentum.
//! * **One-shot** mixins (free functions returning a [`Clip`]) play once when something
//!   happens: an error, a changed value, a drop.
//!
//! Everything yields [`Pose`]s (or plain numbers) and reads time from the host, so an adapter
//! only applies the result. Strength and feel come from a [`Personality`].

use crate::motion::{
    nearest, rubber_band, Bounce, Clip, Decay, Easing, Lerp, Pose, Shake, Spring, SpringState,
};

/// How a design system feels: picks spring stiffness and how big gestures are.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Personality {
    /// Quick, critically damped, small gestures. Decisive.
    Snappy,
    #[default]
    /// Soft, barely overshooting. Calm.
    Smooth,
    /// Bouncy, bigger gestures. Friendly.
    Playful,
}

impl Personality {
    pub const ALL: [Personality; 3] = [
        Personality::Snappy,
        Personality::Smooth,
        Personality::Playful,
    ];
    pub fn spring(self) -> Spring {
        match self {
            Personality::Snappy => Spring::from_response(0.18, 0.0),
            Personality::Smooth => Spring::from_response(0.35, 0.1),
            Personality::Playful => Spring::from_response(0.45, 0.4),
        }
    }
    /// Gesture size multiplier.
    pub fn amount(self) -> f32 {
        match self {
            Personality::Snappy => 0.8,
            Personality::Smooth => 1.0,
            Personality::Playful => 1.3,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Personality::Snappy => "snappy",
            Personality::Smooth => "smooth",
            Personality::Playful => "playful",
        }
    }
}

/// The held state a reactive mixin follows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Trigger {
    Hover,
    Press,
    Focus,
    Selected,
}

/// A reactive mixin.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reactive {
    /// Shrinks slightly while pressed.
    PressSquash,
    /// Rises and glows a little while hovered.
    HoverLift,
    /// Glows while focused.
    FocusGlow,
    /// Pops up slightly while selected.
    SelectPop,
}

impl Reactive {
    pub fn trigger(self) -> Trigger {
        match self {
            Reactive::PressSquash => Trigger::Press,
            Reactive::HoverLift => Trigger::Hover,
            Reactive::FocusGlow => Trigger::Focus,
            Reactive::SelectPop => Trigger::Selected,
        }
    }
    /// The pose at full strength.
    pub fn target(self, p: Personality) -> Pose {
        let a = p.amount();
        match self {
            Reactive::PressSquash => Pose::scaled(1.0 - 0.04 * a),
            Reactive::HoverLift => Pose {
                dy: -2.0 * a,
                glow: 0.35,
                ..Pose::IDENTITY
            },
            Reactive::FocusGlow => Pose::glowing(1.0),
            Reactive::SelectPop => Pose {
                scale: 1.0 + 0.02 * a,
                glow: 0.5,
                ..Pose::IDENTITY
            },
        }
    }
}

/// One reactive mixin in motion.
#[derive(Clone, Copy, Debug)]
pub struct Reaction {
    mixin: Reactive,
    personality: Personality,
    amount: SpringState,
}

impl Reaction {
    pub fn new(mixin: Reactive, personality: Personality) -> Self {
        Self {
            mixin,
            personality,
            amount: SpringState::new(personality.spring(), 0.0),
        }
    }
    pub fn set(&mut self, on: bool, now: f32) {
        self.amount.retarget(if on { 1.0 } else { 0.0 }, now);
    }
    pub fn pose(&self, now: f32) -> Pose {
        Pose::IDENTITY.lerp(&self.mixin.target(self.personality), self.amount.value(now))
    }
    pub fn settled(&self, now: f32) -> bool {
        self.amount.settled(now, 0.002)
    }
}

/// Several reactive mixins stacked on one widget. Poses layer, so the adapter applies one.
///
/// ```
/// use dodai_core::motion::{Personality, Reactive, Reactions, Trigger};
/// let mut r = Reactions::new(Personality::Snappy)
///     .with(Reactive::HoverLift)
///     .with(Reactive::PressSquash);
/// r.set(Trigger::Hover, true, 0.0);
/// assert!(r.pose(0.3).dy < 0.0);
/// ```
#[derive(Clone, Debug, Default)]
pub struct Reactions {
    personality: Personality,
    items: Vec<Reaction>,
    reduced: bool,
}

impl Reactions {
    pub fn new(personality: Personality) -> Self {
        Self {
            personality,
            items: Vec::new(),
            reduced: false,
        }
    }
    pub fn with(mut self, m: Reactive) -> Self {
        self.items.push(Reaction::new(m, self.personality));
        self
    }
    /// Honour the user's reduced-motion preference.
    pub fn reduced(mut self, on: bool) -> Self {
        self.reduced = on;
        self
    }
    /// Switch the reduced-motion preference on a live set.
    pub fn set_reduced(&mut self, on: bool) {
        self.reduced = on;
    }
    pub fn set(&mut self, trigger: Trigger, on: bool, now: f32) {
        for r in self.items.iter_mut() {
            if r.mixin.trigger() == trigger {
                r.set(on, now);
            }
        }
    }
    pub fn pose(&self, now: f32) -> Pose {
        let mut p = Pose::IDENTITY;
        for r in &self.items {
            p = p.layer(r.pose(now));
        }
        if self.reduced {
            p.reduced()
        } else {
            p
        }
    }
    /// Nothing is moving: no need to request another frame.
    pub fn settled(&self, now: f32) -> bool {
        self.items.iter().all(|r| r.settled(now))
    }
}

// ───────────────────────────── one-shot mixins ─────────────────────────────

/// Invalid input: shakes sideways and settles.
pub fn shake(p: Personality) -> Clip<Pose> {
    Clip::shake(Shake::new(9.0, 7.0), 8.0 * p.amount()).map(|x| Pose::offset(x, 0.0))
}

/// Gentle attention tap: a small push that springs back.
pub fn nudge(p: Personality) -> Clip<Pose> {
    Clip::spring(p.spring(), 6.0 * p.amount(), 0.0, 0.0).map(|x| Pose::offset(x, 0.0))
}

/// "Look at me": drops from a small height and bounces.
pub fn attention_bounce(p: Personality) -> Clip<Pose> {
    let b = Bounce::new(14.0 * p.amount(), 2600.0, 0.55);
    Clip::bounce(b).map(|h| Pose::offset(0.0, -h))
}

/// A value changed: glows then fades back.
pub fn flash_on_change() -> Clip<Pose> {
    Clip::ease(1.0, 0.0, 0.6, Easing::EaseOut).map(Pose::glowing)
}

/// Doorbell wobble: a decaying rotation.
pub fn wobble(p: Personality) -> Clip<Pose> {
    Clip::shake(Shake::new(6.0, 5.0), 6.0 * p.amount()).map(Pose::rotated)
}

/// Appear with a springy pop: scales up from `0.8` while fading in.
pub fn pop_in(p: Personality) -> Clip<Pose> {
    let scale = Clip::spring(p.spring(), 0.8, 1.0, 0.0);
    let fade = Clip::ease(0.0, 1.0, 0.18, Easing::EaseOut);
    scale.zip_with(fade, |s, o| Pose {
        scale: s,
        opacity: o,
        ..Pose::IDENTITY
    })
}

/// Number tick-up: eases a displayed value to its new target.
pub fn count_up(from: f32, to: f32, secs: f32) -> Clip<f32> {
    Clip::ease(from, to, secs, Easing::EaseOut)
}

/// Success tick: stroke progress `0..=1` for a path-trim animation.
pub fn check_draw(secs: f32) -> Clip<f32> {
    Clip::ease(0.0, 1.0, secs, Easing::EaseOut)
}

// ───────────────────────────── loading ─────────────────────────────

/// Skeleton shimmer position sweeping `0..=1` across the placeholder, forever.
pub fn skeleton_shimmer(period: f32) -> Clip<f32> {
    Clip::ease(0.0, 1.0, period, Easing::Linear).forever()
}

/// Indeterminate bar: `(start, end)` fractions of the track, forever. The head runs ahead,
/// then the tail catches up.
pub fn indeterminate(period: f32) -> Clip<(f32, f32)> {
    Clip::keyframes(
        period,
        vec![
            (0.0, (0.0, 0.0), Easing::Linear),
            (0.5, (0.1, 0.8), Easing::EaseInOut),
            (1.0, (1.0, 1.0), Easing::EaseInOut),
        ],
    )
    .forever()
}

/// Characters revealed so far, at `chars_per_sec`. Floor it for the visible count.
pub fn typewriter(chars: usize, chars_per_sec: f32) -> Clip<f32> {
    let n = chars as f32;
    Clip::ease(0.0, n, n / chars_per_sec.max(1e-3), Easing::Linear)
}

/// Marquee offset (`0` to `-(content + gap)` px) scrolling at `speed` px/s, forever.
pub fn marquee(content: f32, gap: f32, speed: f32) -> Clip<f32> {
    let run = content + gap;
    Clip::ease(0.0, -run, run / speed.max(1e-3), Easing::Linear).forever()
}

// ───────────────────────────── layout and gestures ─────────────────────────────

/// Height change (expand / collapse) as a spring.
pub fn collapse(from: f32, to: f32, p: Personality) -> Clip<f32> {
    Clip::spring(p.spring(), from, to, 0.0)
}

/// FLIP reorder: an item that moved by `delta` (old position minus new) starts there visually
/// and springs to its place. Returns the translation to apply.
pub fn flip(delta: (f32, f32), p: Personality) -> Clip<(f32, f32)> {
    Clip::spring(p.spring(), 1.0, 0.0, 0.0).map(move |k| (delta.0 * k, delta.1 * k))
}

/// What happens when a swiped item is let go.
pub enum Release {
    /// Fling off-screen: the offset to animate.
    Dismiss(Clip<f32>),
    /// Spring back to rest.
    Snap(Clip<f32>),
}

/// Let go of a swipe at offset `x` px moving `v` px/s. If its projected resting place is past
/// half the `width` it flies out the way it was going, otherwise it springs back.
pub fn swipe_release(x: f32, v: f32, width: f32, p: Personality) -> Release {
    let rest = Decay::new(x, v, 4.0).rest();
    if rest.abs() > 0.5 * width {
        Release::Dismiss(Clip::spring(p.spring(), x, width.copysign(rest) * 1.2, v))
    } else {
        Release::Snap(Clip::spring(p.spring(), x, 0.0, v))
    }
}

/// Land a drag released at `x` with velocity `v` on the nearest of `points` (pages, snap
/// stops), springing there with the throw's momentum.
pub fn page_snap(points: &[f32], x: f32, v: f32, p: Personality) -> Clip<f32> {
    Clip::spring(p.spring(), x, fling_to(points, x, v), v)
}

/// Which of `points` a throw released at `x` with velocity `v` coasts to (scroll-view
/// friction, then the nearest point).
pub fn fling_to(points: &[f32], x: f32, v: f32) -> f32 {
    nearest(points, Decay::new(x, v, 4.0).rest())
}

/// Overscroll resistance: how far content actually moves for `offset` px of drag past the
/// edge, never more than `limit`.
pub fn overscroll(offset: f32, limit: f32) -> f32 {
    rubber_band(offset, limit, 0.55)
}

/// Pull-to-refresh: `(content offset, armed)` for a pull of `pull` px; armed once `pull`
/// reaches `threshold`.
pub fn pull_to_refresh(pull: f32, threshold: f32) -> (f32, bool) {
    (
        rubber_band(pull.max(0.0), threshold * 1.5, 0.55),
        pull >= threshold,
    )
}

/// Magnetic pull: a pointer at `(dx, dy)` from the widget centre, within `radius`, drags it
/// towards the pointer by up to `strength` of the distance.
pub fn magnetic(dx: f32, dy: f32, radius: f32, strength: f32) -> Pose {
    let dist = (dx * dx + dy * dy).sqrt();
    if dist >= radius || radius <= 0.0 {
        return Pose::IDENTITY;
    }
    let pull = (1.0 - dist / radius) * strength;
    Pose::offset(dx * pull, dy * pull)
}

/// Tilt towards a pointer at normalised `(nx, ny)` (`-1..=1`), up to `max_deg`, with a little
/// parallax shift.
pub fn tilt(nx: f32, ny: f32, max_deg: f32) -> Pose {
    let (nx, ny) = (nx.clamp(-1.0, 1.0), ny.clamp(-1.0, 1.0));
    Pose {
        rotate: nx * max_deg,
        dx: nx * 2.0,
        dy: ny * 2.0,
        ..Pose::IDENTITY
    }
}
