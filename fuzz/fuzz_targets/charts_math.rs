#![no_main]
use dodai_charts as c;
use dodai_instruments as ic;
use libfuzzer_sys::fuzz_target;

// Chart and instrument maths take caller data (NaN, infinities, empty, huge periods): never panic, never hang.
fuzz_target!(|d: (Vec<f32>, Vec<(f32, f32)>, Vec<(f32, f32, f32, f32, Vec<u8>)>, f32, f32, u8, u8)| {
    let (v, pts, tasks, a, b, count, period) = d;
    let (count, period) = (count as usize, period as usize);
    let _ = c::nice_ticks(a, b, count);
    let _ = ic::gauge::odometer_positions(a, count);
    let _ = ic::gauge::cell_lit(count, period, a);
    let _ = ic::gauge::tick_fracs(count, period);
    let _ = ic::gauge::zone(a);
    let _ = ic::gauge::pill(a, b, a, b);
    let _ = ic::gauge::lit_bars(a, count);
    let _ = ic::gauge::hex_extent(count, period, a);
    let _ = c::fmt_num(a);
    let _ = c::pie_fractions(&v);
    let _ = c::norm(a, b, a);
    let _ = c::smooth_points(&pts, count.min(32));
    let _ = c::step_points(&pts);
    let _ = c::indicators::sma(&v, period);
    let _ = c::indicators::ema(&v, period);
    let _ = c::indicators::rsi(&v, period);
    let _ = c::bins::histogram(&v, Some(count));
    let _ = c::bins::auto_bin_count(&v);
    let _ = c::renko::renko(&v, a);
    let _ = c::stack::split(a, &v, b);
    let _ = c::stack::shared_gutter(&v);
    let _ = c::depth::depth_curves(&pts, &pts);
    let _ = c::depth::mid_price(&pts, &pts);
    let _ = c::axis::aligned_ticks((a, b), (b, a), count);
    let tasks: Vec<c::gantt::Task> = tasks
        .into_iter()
        .map(|(s, e, p, _, deps)| {
            let mut t = c::gantt::Task::new("t", s, e).progress(p);
            for d in deps {
                t = t.after(d as usize);
            }
            t
        })
        .collect();
    let _ = c::gantt::layout(&tasks);
    let candles: Vec<c::Candle> = v
        .chunks_exact(5)
        .map(|w| c::Candle { open: w[0], high: w[1], low: w[2], close: w[3], volume: w[4] })
        .collect();
    let _ = c::heikin_ashi(&candles);
    let _ = c::price_range(&candles);
});
