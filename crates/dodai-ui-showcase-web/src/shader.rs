//! Shaders on the page. The browser build has no GPU renderer: the `<img>` carries the same
//! query as the server build and the host maps it to a picture rendered beforehand.

use dodai_ui_core::Theme;
use dodai_ui_showcase_model::node::ShaderSpec;

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
        "<img src=\"/shader?{q}&t=0\" data-q=\"{q}\" width=\"{}\" height=\"{h}\" style=\"flex:none;display:block\" alt=\"\">",
        spec.width
    )
}
