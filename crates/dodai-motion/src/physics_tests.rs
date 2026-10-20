//! Tests for the physics, clip composition and mixins.

use crate::*;

fn near(a: f32, b: f32, e: f32) -> bool {
    (a - b).abs() <= e
}

// ── springs ──
#[test]
fn spring_starts_at_rest_position_and_ends_at_target_for_every_damping() {
    for bounce in [-1.0, 0.0, 0.3, 0.8] {
        let s = Spring::from_response(0.4, bounce);
        let (x, v) = s.at(0.0, 2.0, 0.0, 10.0);
        assert!(near(x, 2.0, 1e-4) && near(v, 0.0, 1e-3), "bounce {bounce}");
        let (x, v) = s.at(30.0, 2.0, 5.0, 10.0);
        assert!(near(x, 10.0, 1e-3) && near(v, 0.0, 1e-3), "bounce {bounce}");
    }
}

#[test]
fn spring_velocity_is_the_derivative_of_position() {
    for bounce in [-0.5, 0.0, 0.5] {
        let s = Spring::from_response(0.5, bounce);
        for t in [0.05f32, 0.2, 0.6] {
            let h = 1e-3;
            let num = (s.at(t + h, 0.0, 3.0, 1.0).0 - s.at(t - h, 0.0, 3.0, 1.0).0) / (2.0 * h);
            let v = s.at(t, 0.0, 3.0, 1.0).1;
            assert!(
                near(num, v, 0.02 * v.abs().max(1.0)),
                "b{bounce} t{t}: {num} vs {v}"
            );
        }
    }
}

#[test]
fn bouncy_springs_overshoot_and_critical_ones_do_not() {
    let peak = |s: Spring| {
        (0..400)
            .map(|i| s.at(i as f32 / 200.0, 0.0, 0.0, 1.0).0)
            .fold(0.0f32, f32::max)
    };
    assert!(peak(Spring::from_response(0.4, 0.6)) > 1.05);
    assert!(peak(Spring::from_response(0.4, 0.0)) <= 1.001);
    assert!(peak(Spring::from_response(0.4, -1.0)) <= 1.001);
}

#[test]
fn retargeting_a_spring_keeps_its_velocity() {
    let mut s = SpringState::new(Spring::from_response(0.4, 0.2), 0.0);
    s.retarget(1.0, 0.0);
    let (x, v) = (s.value(0.1), s.velocity(0.1));
    assert!(v > 0.0);
    s.retarget(0.0, 0.1);
    assert!(
        near(s.value(0.1), x, 1e-4) && near(s.velocity(0.1), v, 1e-3),
        "continuous"
    );
    assert!(!s.settled(0.1, 0.002) && s.settled(20.0, 0.002));
}

#[test]
fn impulse_kicks_a_resting_spring() {
    let mut s = SpringState::new(Spring::from_response(0.3, 0.3), 0.0);
    s.impulse(5.0, 1.0);
    assert!(s.value(1.05) > 0.0);
    assert!(s.settled(10.0, 0.002));
}

// ── decay, bounce, rubber band, damp, shake ──
#[test]
fn decay_coasts_to_its_rest_point() {
    let d = Decay::new(10.0, 200.0, 4.0);
    assert!(near(d.rest(), 60.0, 1e-3));
    assert!(near(d.position(100.0), d.rest(), 1e-3));
    assert!(d.velocity(1.0) < d.velocity(0.0));
    assert!(d.duration(0.5) > 0.0 && d.position(d.duration(0.5)) < d.rest() + 0.2);
}

#[test]
fn nearest_snaps_to_points() {
    assert_eq!(nearest(&[0.0, 100.0, 200.0], 130.0), 100.0);
    assert_eq!(nearest(&[], 7.0), 7.0);
}

#[test]
fn a_dropped_ball_bounces_lower_each_time_then_rests() {
    let b = Bounce::new(100.0, 2000.0, 0.5);
    assert!(near(b.at(0.0), 100.0, 1e-3));
    let fall = (2.0f32 * 100.0 / 2000.0).sqrt();
    assert!(b.at(fall) < 0.5, "hits the ground");
    let peak1 = (0..200)
        .map(|i| b.at(fall + i as f32 * 0.002))
        .fold(0.0f32, f32::max);
    let t2 = fall + 2.0 * (2000.0f32 * fall * 0.5) / 2000.0;
    let peak2 = (0..200)
        .map(|i| b.at(t2 + i as f32 * 0.002))
        .fold(0.0f32, f32::max);
    assert!(peak1 > peak2 && peak2 > 0.0);
    assert_eq!(b.at(b.duration() + 5.0), 0.0);
    assert!(
        (0..1000).all(|i| b.at(i as f32 * 0.01) >= -1e-3),
        "never below the ground"
    );
}

#[test]
fn rubber_band_resists_and_is_bounded() {
    assert_eq!(rubber_band(0.0, 100.0, 0.55), 0.0);
    let (a, b, c) = (
        rubber_band(50.0, 100.0, 0.55),
        rubber_band(500.0, 100.0, 0.55),
        rubber_band(50000.0, 100.0, 0.55),
    );
    assert!(a < 50.0 && a < b && b < c && c < 100.0);
    assert!(rubber_band(-50.0, 100.0, 0.55) < 0.0, "sign kept");
}

#[test]
fn damp_is_frame_rate_independent() {
    let run = |fps: u32| {
        let mut x = 0.0;
        for _ in 0..fps {
            x = damp(x, 1.0, 5.0, 1.0 / fps as f32);
        }
        x
    };
    assert!(near(run(30), run(240), 1e-3));
    assert!(near(run(60), 1.0 - (-5.0f32).exp(), 1e-3));
}

#[test]
fn shake_starts_at_zero_and_dies_out() {
    let s = Shake::new(9.0, 7.0);
    assert!(near(s.at(0.0), 0.0, 1e-6));
    assert!(s.at(s.duration() + 0.1).abs() < 0.06);
    assert!((0..100).any(|i| s.at(i as f32 * 0.01).abs() > 0.5));
}

// ── clip composition ──
fn ramp(d: f32) -> Clip<f32> {
    Clip::ease(0.0, 1.0, d, Easing::Linear)
}

#[test]
fn clips_hold_their_end_and_clamp_their_start() {
    let c = ramp(1.0);
    assert!(
        near(c.at(-5.0), 0.0, 1e-6) && near(c.at(0.5), 0.5, 1e-6) && near(c.at(9.0), 1.0, 1e-6)
    );
    assert!(c.done(1.0) && !c.done(0.9));
}

#[test]
fn then_delay_repeat_pingpong_reverse_speed() {
    let seq = ramp(1.0).then(ramp(1.0).map(|x| x + 1.0));
    assert!(near(seq.duration(), 2.0, 1e-6) && near(seq.at(1.5), 1.5, 1e-6));
    let d = ramp(1.0).delay(0.5);
    assert!(near(d.at(0.25), 0.0, 1e-6) && near(d.at(1.0), 0.5, 1e-6));
    let r = ramp(1.0).repeat(3);
    assert!(
        near(r.at(2.25), 0.25, 1e-5) && near(r.duration(), 3.0, 1e-6) && near(r.at(9.0), 1.0, 1e-6)
    );
    let pp = ramp(1.0).ping_pong();
    assert!(near(pp.at(1.5), 0.5, 1e-6) && near(pp.duration(), 2.0, 1e-6));
    assert!(near(ramp(1.0).reverse().at(0.25), 0.75, 1e-6));
    let f = ramp(1.0).speed(2.0);
    assert!(near(f.duration(), 0.5, 1e-6) && near(f.at(0.25), 0.5, 1e-6));
    assert!(ramp(1.0).forever().duration().is_infinite());
    assert!(near(ramp(1.0).forever().at(7.25), 0.25, 1e-4));
}

#[test]
fn zip_runs_in_parallel_and_the_shorter_holds() {
    let z = ramp(1.0).zip(ramp(2.0));
    assert!(near(z.duration(), 2.0, 1e-6));
    let (a, b) = z.at(1.0);
    assert!(near(a, 1.0, 1e-6) && near(b, 0.5, 1e-6));
    let (a, _) = z.at(1.8);
    assert!(near(a, 1.0, 1e-6), "short one holds its end");
}

#[test]
fn stagger_offsets_each_clip() {
    let v = Clip::stagger(4, 0.1, |_| ramp(1.0));
    assert_eq!(v.len(), 4);
    assert!(near(v[0].at(0.5), 0.5, 1e-6) && near(v[3].at(0.5), 0.2, 1e-6));
    assert!(near(v[3].duration(), 1.3, 1e-5));
}

#[test]
fn keyframes_interpolate_between_stops_with_per_segment_easing() {
    let k = Clip::keyframes(
        1.0,
        vec![
            (0.0, 0.0f32, Easing::Linear),
            (0.5, 10.0, Easing::Linear),
            (1.0, 0.0, Easing::Linear),
        ],
    );
    assert!(
        near(k.at(0.25), 5.0, 1e-4) && near(k.at(0.5), 10.0, 1e-4) && near(k.at(0.75), 5.0, 1e-4)
    );
}

#[test]
fn physics_are_clips_so_they_compose() {
    let s = Clip::spring(Spring::from_response(0.3, 0.0), 0.0, 1.0, 0.0);
    assert!(s.duration() > 0.1 && s.duration() < 3.0);
    assert!(near(s.at(s.duration()), 1.0, 0.01));
    let both = s
        .clone()
        .then(Clip::spring(Spring::from_response(0.3, 0.0), 1.0, 0.0, 0.0));
    assert!(near(both.at(both.duration()), 0.0, 0.01));
    let fling = Clip::decay(Decay::new(0.0, 300.0, 4.0));
    assert!(near(fling.at(fling.duration()), 75.0, 1.0));
}

#[test]
fn lerp_covers_tuples_and_arrays() {
    assert_eq!((0.0, 10.0).lerp(&(10.0, 20.0), 0.5), (5.0, 15.0));
    assert_eq!([0.0, 2.0].lerp(&[2.0, 4.0], 0.5), [1.0, 3.0]);
}

#[test]
fn player_follows_the_hosts_clock() {
    let mut p = Player::new(ramp(1.0), 10.0);
    assert!(near(p.value(10.5), 0.5, 1e-6) && !p.done(10.5) && p.done(11.0));
    p.restart(20.0);
    assert!(near(p.value(20.25), 0.25, 1e-6));
}

#[test]
fn animator_plays_the_entry_then_rests() {
    let m = Motion::default().entry(EntryKind::Rise);
    let a = Animator::new(m, 5.0);
    assert!(a.frame(5.0).opacity < 0.05 && a.frame(5.0).dy > 0.0);
    assert!(!a.done(5.05));
    let end = a.frame(50.0);
    assert!(end.is_identity() && a.done(50.0));
}

// ── poses ──
#[test]
fn poses_layer_and_reduce() {
    let p = Pose::scaled(0.9)
        .layer(Pose::scaled(0.9))
        .layer(Pose::offset(3.0, 1.0));
    assert!(near(p.scale, 0.81, 1e-6) && p.dx == 3.0);
    assert_eq!(Pose::IDENTITY.layer(Pose::IDENTITY), Pose::IDENTITY);
    assert!(Pose::IDENTITY.is_identity());
    let r = Pose {
        dy: 5.0,
        scale: 2.0,
        rotate: 9.0,
        glow: 0.7,
        opacity: 0.5,
        dx: 4.0,
    }
    .reduced();
    assert_eq!((r.dx, r.dy, r.scale, r.rotate), (0.0, 0.0, 1.0, 0.0));
    assert!(r.glow == 0.7 && r.opacity == 0.5, "state feedback survives");
}

// ── mixins ──
#[test]
fn reactive_mixins_follow_state_and_settle() {
    let mut r = Reactions::new(Personality::Snappy)
        .with(Reactive::HoverLift)
        .with(Reactive::PressSquash);
    assert!(r.pose(0.0).is_identity() && r.settled(0.0));
    r.set(Trigger::Hover, true, 0.0);
    assert!(!r.settled(0.05));
    let up = r.pose(3.0);
    assert!(up.dy < -1.0 && up.glow > 0.3 && near(up.scale, 1.0, 1e-3));
    r.set(Trigger::Press, true, 3.0);
    assert!(r.pose(6.0).scale < 0.97);
    r.set(Trigger::Hover, false, 6.0);
    r.set(Trigger::Press, false, 6.0);
    assert!(r.pose(12.0).is_identity() && r.settled(12.0));
}

#[test]
fn reduced_motion_keeps_feedback_but_not_movement() {
    let mut r = Reactions::new(Personality::Playful)
        .with(Reactive::HoverLift)
        .reduced(true);
    r.set(Trigger::Hover, true, 0.0);
    let p = r.pose(3.0);
    assert!(p.dy == 0.0 && p.glow > 0.3);
    let c = shake(Personality::Smooth).reduced();
    assert!((0..50).all(|i| c.at(i as f32 * 0.01).dx == 0.0));
}

#[test]
fn personality_changes_the_feel() {
    let q = |p: Personality| Spring::zeta(&p.spring());
    assert!(near(q(Personality::Snappy), 1.0, 0.01), "critically damped");
    assert!(q(Personality::Playful) < q(Personality::Smooth));
    assert!(Personality::Playful.amount() > Personality::Snappy.amount());
}

#[test]
fn one_shot_mixins_start_and_end_at_rest() {
    let p = Personality::Smooth;
    for (name, c) in [
        ("shake", shake(p)),
        ("nudge", nudge(p)),
        ("bounce", attention_bounce(p)),
        ("flash", flash_on_change()),
        ("wobble", wobble(p)),
    ] {
        assert!(c.duration() > 0.05, "{name}");
        let end = c.at(c.duration());
        assert!(
            near(end.dx, 0.0, 0.3)
                && near(end.dy, 0.0, 0.3)
                && near(end.rotate, 0.0, 0.5)
                && end.glow < 0.01,
            "{name} ends at rest: {end:?}"
        );
    }
    let pop = pop_in(p);
    assert!(pop.at(0.0).opacity < 0.01 && pop.at(0.0).scale < 0.85);
    assert!(pop.at(pop.duration()).is_identity() || near(pop.at(pop.duration()).scale, 1.0, 0.01));
    assert!(flash_on_change().at(0.0).glow > 0.99);
}

#[test]
fn counters_and_progress_helpers() {
    let c = count_up(0.0, 100.0, 1.0);
    assert!(near(c.at(0.0), 0.0, 1e-6) && near(c.at(1.0), 100.0, 1e-4) && c.at(0.5) > 50.0);
    assert!(near(check_draw(0.4).at(0.4), 1.0, 1e-6));
    let t = typewriter(10, 20.0);
    assert!(near(t.duration(), 0.5, 1e-6) && near(t.at(0.25), 5.0, 1e-4));
    let m = marquee(200.0, 40.0, 120.0);
    assert!(
        m.duration().is_infinite()
            && near(m.at(1.0), -120.0, 1e-3)
            && near(m.at(2.0 + 0.0), -240.0 % 240.0, 1e-3)
    );
    let s = skeleton_shimmer(1.5);
    assert!(near(s.at(0.75), 0.5, 1e-6) && near(s.at(1.5 + 0.75), 0.5, 1e-4));
    let (a, b) = indeterminate(1.0).at(0.5);
    assert!(b > a && near(b, 0.8, 1e-4));
    let (a, b) = indeterminate(1.0).at(0.9999);
    assert!(near(a, 1.0, 1e-2) && near(b, 1.0, 1e-2));
}

#[test]
fn gestures() {
    let p = Personality::Smooth;
    // slow short drag springs back, fast long fling dismisses
    match swipe_release(30.0, 50.0, 300.0, p) {
        Release::Snap(c) => assert!(near(c.at(c.duration()), 0.0, 0.05)),
        Release::Dismiss(_) => panic!("should snap back"),
    }
    match swipe_release(120.0, 1500.0, 300.0, p) {
        Release::Dismiss(c) => assert!(c.at(c.duration()) > 300.0),
        Release::Snap(_) => panic!("should dismiss"),
    }
    match swipe_release(-120.0, -1500.0, 300.0, p) {
        Release::Dismiss(c) => assert!(
            c.at(c.duration()) < -300.0,
            "dismisses the way it was going"
        ),
        Release::Snap(_) => panic!(),
    }
    let pts = [0.0, 300.0, 600.0];
    let c = page_snap(&pts, 100.0, 1200.0, p);
    assert!(near(c.at(c.duration()), 300.0, 1.0) || near(c.at(c.duration()), 600.0, 1.0));
    let still = page_snap(&pts, 100.0, 0.0, p);
    assert!(near(still.at(still.duration()), 0.0, 0.5));
    assert!(overscroll(400.0, 100.0) < 100.0);
    let (off, armed) = pull_to_refresh(80.0, 64.0);
    assert!(armed && off < 80.0 && !pull_to_refresh(20.0, 64.0).1);
    let f = flip((40.0, -20.0), p);
    assert!(near(f.at(0.0).0, 40.0, 1e-3) && near(f.at(f.duration()).1, 0.0, 0.2));
}

#[test]
fn pointer_mixins() {
    assert!(
        magnetic(100.0, 0.0, 50.0, 0.3).is_identity(),
        "outside the radius"
    );
    let m = magnetic(10.0, 0.0, 50.0, 0.5);
    assert!(m.dx > 0.0 && m.dx < 10.0 && m.dy == 0.0);
    let centred = magnetic(0.0, 0.0, 50.0, 0.5);
    assert!(centred.is_identity());
    let t = tilt(1.0, -1.0, 8.0);
    assert!(t.rotate == 8.0 && t.dy < 0.0);
    assert_eq!(tilt(5.0, 0.0, 8.0).rotate, 8.0, "clamped");
}
