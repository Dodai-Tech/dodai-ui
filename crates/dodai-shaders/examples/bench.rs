//! `cargo run --release -p dodai-shaders --example bench` — ms per 320x180 frame.
use dodai_shaders::*;
use std::time::Instant;

fn main() {
    let mut rows = vec![];
    for s in catalog() {
        let ctx = Ctx::new(320, 180, 1.5, s.family.palette());
        let chain: Vec<_> = if s.is_generator() {
            vec![s]
        } else {
            vec![by_id(s.family.default_source()).unwrap(), s]
        };
        render(&chain, &ctx);
        let t = Instant::now();
        for _ in 0..5 {
            render(&chain, &ctx);
        }
        rows.push((t.elapsed().as_secs_f64() * 200.0, s.id));
    }
    rows.sort_by(|a, b| b.0.total_cmp(&a.0));
    let total: f64 = rows.iter().map(|r| r.0).sum();
    for (ms, id) in &rows {
        println!("{ms:7.2} ms  {id}");
    }
    println!("total {total:.1} ms, avg {:.2}", total / rows.len() as f64);
}
