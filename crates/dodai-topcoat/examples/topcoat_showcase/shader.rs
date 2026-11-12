//! Shaders on the page: an `<img>` whose picture the server renders (GPU, or CPU without one)
//! at the moment it is asked for. Moving shaders are asked again a few times a second.

use dodai_core::Theme;
use dodai_shaders::player::{config_hash, Player, Poll, Request};
use dodai_showcase_model::node::ShaderSpec;
use std::time::{Duration, Instant};

/// The `<img>` of `spec` under `theme`.
pub fn view(spec: &ShaderSpec, theme: &Theme) -> String {
    let h = spec.width / spec.aspect;
    let q = format!(
        "chain={}&preset={}&themed={}&w={}&aspect={}&i={}&p={}&an={}&theme={}",
        spec.chain.join(","),
        spec.preset.unwrap_or(""),
        u8::from(spec.themed),
        spec.width,
        spec.aspect,
        spec.intensity,
        spec.pixel,
        u8::from(spec.animated),
        theme.id()
    );
    format!(
        "<img {} src=\"/shader?{q}&t=0\" data-q=\"{q}\" width=\"{}\" height=\"{h}\" style=\"flex:none;display:block\" alt=\"\">",
        if spec.animated { "data-shader" } else { "" },
        spec.width
    )
}

fn param<'a>(query: &'a str, key: &str) -> &'a str {
    query
        .split('&')
        .find_map(|kv| kv.strip_prefix(key).and_then(|r| r.strip_prefix('=')))
        .unwrap_or("")
}

fn number(query: &str, key: &str, default: f32) -> f32 {
    param(query, key).parse().unwrap_or(default)
}

/// The picture `query` asks for as PNG bytes (`None` when the shader is unknown or the picture
/// does not come in time).
pub fn png(query: &str) -> Option<Vec<u8>> {
    let theme = dodai_core::themes::all()
        .into_iter()
        .find(|t| t.id() == param(query, "theme"))?;
    let chain: Vec<&'static str> = param(query, "chain")
        .split(',')
        .filter(|s| !s.is_empty())
        .filter_map(|id| dodai_shaders::by_id(id).map(|s| s.id))
        .collect();
    let preset = dodai_shaders::presets::all()
        .find(|p| p.id == param(query, "preset"))
        .map(|p| p.id);
    let spec = ShaderSpec {
        chain,
        preset,
        themed: param(query, "themed") == "1",
        width: number(query, "w", 220.0),
        aspect: number(query, "aspect", 16.0 / 9.0),
        intensity: number(query, "i", 1.0),
        pixel: number(query, "p", 2.0),
        animated: param(query, "an") == "1",
    };
    let sp = spec.to_spec(&theme);
    if sp.chain.is_empty() {
        return None;
    }
    let time = number(query, "t", 0.0) / 1000.0 * sp.speed;
    let config = config_hash(&sp.chain, &sp.palette, sp.intensity, sp.transparent);
    let mut player = Player::new();
    let give_up = Instant::now() + Duration::from_secs(4);
    let frame = loop {
        let now = Instant::now();
        let poll = player.poll(&Request {
            chain: &sp.chain,
            palette: &sp.palette,
            size: (spec.width, spec.width / sp.aspect),
            scale: 1.0,
            pixel: sp.pixel,
            intensity: sp.intensity,
            time,
            fps: sp.fps,
            animated: sp.animated,
            transparent: sp.transparent,
            config,
            now,
        });
        if let Poll::New(f) = poll {
            break f;
        }
        if now > give_up {
            return None;
        }
        std::thread::sleep(Duration::from_millis(15));
    };
    let (w, h) = (frame.width, frame.height);
    let mut out = Vec::new();
    let mut enc = png::Encoder::new(&mut out, w, h);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header()
        .ok()?
        .write_image_data(&frame.rgba())
        .ok()?;
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_image_carries_its_query_and_moves_only_when_animated() {
        let t = dodai_core::themes::all().remove(0);
        let moving = view(&ShaderSpec::chain(&["neon_grid"]), &t);
        assert!(moving.contains("data-shader") && moving.contains("chain=neon_grid"));
        assert!(!view(&ShaderSpec::chain(&["neon_grid"]).still(), &t).contains("data-shader"));
    }

    #[test]
    fn query_values_are_read_by_name() {
        assert_eq!(param("a=1&bb=2", "bb"), "2");
        assert_eq!(param("a=1&bb=2", "b"), "");
        assert_eq!(number("w=12.5", "w", 0.0), 12.5);
    }
}
