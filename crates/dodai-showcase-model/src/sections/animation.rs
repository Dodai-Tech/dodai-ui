//! Animation: each easing as a plot with its description, and the physics models as lanes.
//!
//! The gpui showcase also plays a live entry card and a moving dot per section; those need a
//! clock the model does not have, so here the curves are drawn whole.

use dodai_core::motion::{rubber_band, Bounce, Decay, Easing, Personality, Shake};

use crate::b::*;
use crate::catalog::{Ctx, Section};
use crate::node::Node;
use crate::plot;

const CAT: &str = "Animation";

/// One easing section: its title, curve and description.
struct Ease {
    title: &'static str,
    easing: Easing,
    text: &'static str,
}

const EASINGS: [Ease; 12] = [
    Ease {
        title: "Fade",
        easing: Easing::EaseOut,
        text: "Opacity 0 → 1. The default entry style: quiet, never shifts layout. Ease-out starts fast and settles.",
    },
    Ease {
        title: "Rise",
        easing: Easing::CubicBezier(0.2, 0.8, 0.2, 1.0),
        text: "Fades in while moving up 14 px. A snappy cubic-bezier(.2,.8,.2,1) gives a quick start and a soft landing.",
    },
    Ease {
        title: "Drop",
        easing: Easing::EaseInOut,
        text: "Fades in while dropping down 14 px. Ease-in-out accelerates, then decelerates; good for menus and toasts.",
    },
    Ease {
        title: "Slide left",
        easing: Easing::CubicBezier(0.7, 0.0, 0.3, 1.0),
        text: "Fades in sliding from the left. A strong ease-in-out (.7,0,.3,1) feels deliberate; fits sidebars and drawers.",
    },
    Ease {
        title: "Slide right",
        easing: Easing::EaseIn,
        text: "Fades in sliding from the right. Ease-in starts slowly and finishes fast; use for trailing panels.",
    },
    Ease {
        title: "Spring",
        easing: Easing::SPRING,
        text: "Physics-based easing: a damped spring (stiffness 170, damping 12). Overshoots its target and settles; entry effects clamp at 1.0, value animations (charts, sliders) show the overshoot.",
    },
    Ease {
        title: "Spring · stiff",
        easing: Easing::SPRING_STIFF,
        text: "Stiffness 400, damping 26: snaps into place with a tiny overshoot. Used by the Toggle thumb.",
    },
    Ease {
        title: "Spring · wobbly",
        easing: Easing::SPRING_WOBBLY,
        text: "Stiffness 120, damping 6: loose, lingering wobble. Playful; use sparingly.",
    },
    Ease {
        title: "Spring · gentle",
        easing: Easing::SPRING_GENTLE,
        text: "Stiffness 100, damping 20: heavily damped, no overshoot, soft landing.",
    },
    Ease {
        title: "Bounce",
        easing: Easing::Bounce,
        text: "Ball drop: falls to the target and bounces to rest. Fits dropped-in toasts and badges.",
    },
    Ease {
        title: "Elastic",
        easing: Easing::Elastic,
        text: "Rubber band: shoots past the target and wobbles with a decaying sine.",
    },
    Ease {
        title: "Back · anticipate",
        easing: Easing::Back(1.7),
        text: "Back overshoots the target about 10% and eases back; Anticipate (the mirror) winds up first by pulling away before moving.",
    },
];

macro_rules! ease_section {
    ($i:expr) => {
        Section {
            category: CAT,
            title: EASINGS[$i].title,
            build: |c| ease(c, $i),
        }
    };
}

pub(crate) const SECTIONS: [Section; 13] = [
    ease_section!(0),
    ease_section!(1),
    ease_section!(2),
    ease_section!(3),
    ease_section!(4),
    ease_section!(5),
    ease_section!(6),
    ease_section!(7),
    ease_section!(8),
    ease_section!(9),
    ease_section!(10),
    ease_section!(11),
    Section {
        category: CAT,
        title: "Dynamics · physics models",
        build: physics,
    },
];

fn ease(c: &Ctx, i: usize) -> Node {
    let e = &EASINGS[i];
    let curve = plot::control_points(e.easing)
        .map(|(a, b, x, y)| format!("  cubic-bezier({a}, {b}, {x}, {y})"))
        .unwrap_or_default();
    row(vec![
        Node::Plot(plot::easing(c.theme, e.easing, 200.0)),
        col(vec![muted(format!("{}{curve}", e.text))]),
    ])
}

fn physics(c: &Ctx) -> Node {
    let t = c.theme;
    let m = &t.motion;
    let summary = format!(
        "This design: {} springs, values travel {}, {} ms base duration.",
        m.personality.label(),
        format!("{:?}", m.travel).to_lowercase(),
        m.duration.as_millis()
    );
    let spring = |p: Personality| {
        let s = p.spring();
        move |x: f32| s.at(x * 1.6, 0.0, 0.0, 1.0).0
    };
    let feel = |p: Personality, what: &str| {
        let tag = if p == m.personality {
            " This design's feel."
        } else {
            ""
        };
        format!("{what}{tag}")
    };
    let decay = Decay::new(0.0, 4.0, 3.5);
    let ball = Bounce::new(100.0, 900.0, 0.6);
    let dur = ball.duration();
    let shake = Shake::new(5.0, 3.5);
    let lane = |name: &str, note: String, f: &dyn Fn(f32) -> f32, hi: f32| {
        line(vec![
            Node::Plot(plot::lane(t, 150.0, 70.0, 0.0, hi, f)),
            col(vec![label(name), muted(note)]),
        ])
    };
    col(vec![
        muted(summary),
        lane(
            "Spring · snappy",
            feel(Personality::Snappy, "No overshoot: the fastest arrival."),
            &spring(Personality::Snappy),
            1.4,
        ),
        lane(
            "Spring · smooth",
            feel(Personality::Smooth, "A hair of overshoot, calm."),
            &spring(Personality::Smooth),
            1.4,
        ),
        lane(
            "Spring · playful",
            feel(Personality::Playful, "Clear overshoot and ring-down."),
            &spring(Personality::Playful),
            1.4,
        ),
        lane(
            "Fling (decay)",
            "Exponential friction: coasts to rest at v0 / friction, then snap to a page.".into(),
            &|x| decay.position(x * 1.6),
            1.4,
        ),
        lane(
            "Dropped ball",
            "Gravity with restitution: each bounce keeps 60 % of the speed.".into(),
            &|x| 1.0 - ball.at(x * dur) / 100.0,
            1.05,
        ),
        lane(
            "Shake",
            "A decaying sine: invalid input, doorbell wobble.".into(),
            &|x| 0.5 + 0.5 * shake.at(x * 1.5),
            1.0,
        ),
        lane(
            "Rubber band",
            "Drag past an edge with resistance: output never reaches the limit.".into(),
            &|x| rubber_band(x * 6.0, 1.0, 0.55),
            1.0,
        ),
    ])
}
