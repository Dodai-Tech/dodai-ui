//! See-through fills and frosted glass for `bevy_ui`.
//!
//! Put [`UiGlass`] on a [`Surface`] entity (the panel, kit and button helpers do): its fill turns
//! as see-through as [`dodai_core::Glass`] says, from the app-wide mix in [`DodaiGlass`] and the
//! component's own [`dodai_core::Surface`]. When the glass is frosted and there is a backdrop
//! (a picture set with [`DodaiGlass::set_picture`], else the design's canvas mesh), a child image
//! paints the fill over a blurred crop of the backdrop, lined up with the sharp backdrop that a
//! [`UiBackdrop`] node draws behind the app, and cut to the surface's shape.

use std::sync::Arc;

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::window::PrimaryWindow;
use dodai_core::{Glass, SurfaceClass, SurfaceFx, Theme};

use crate::surface::{cut_distance, resolve_cropped, Paint, Surface, SurfaceCrop};
use crate::theme::DodaiTheme;

/// Which kind of surface an entity is for the surface mix, and its own override.
#[derive(Component, Debug, Clone, Copy, PartialEq, Default)]
pub struct UiGlass {
    /// The kind of surface (`None`: only `over` counts, as for a control).
    pub class: Option<SurfaceClass>,
    /// The component's own surface override (frost, fill opacity).
    pub over: dodai_core::Surface,
    /// Floats over other components: the fill stays at least
    /// [`Glass::FLOATING_ALPHA`] opaque, as what is behind it cannot be blurred.
    pub floating: bool,
}

impl UiGlass {
    /// A `class` surface following the app-wide mix.
    pub fn of(class: SurfaceClass) -> Self {
        Self {
            class: Some(class),
            over: dodai_core::Surface::INHERIT,
            floating: false,
        }
    }

    /// Marked as floating over other components.
    pub fn floating(self) -> Self {
        Self {
            floating: true,
            ..self
        }
    }

    /// With the component's own surface override.
    pub fn with(self, over: dodai_core::Surface) -> Self {
        Self { over, ..self }
    }
}

/// Marks the node that draws the backdrop (stretched over it; put it behind the app, full size).
#[derive(Component, Debug, Clone, Copy, Default)]
#[require(Node, ImageNode)]
pub struct UiBackdrop;

/// The child image of a frosted surface, and what it was drawn for.
#[derive(Component, Debug, Clone, PartialEq)]
pub(crate) struct FrostImage {
    key: Vec<u32>,
}

/// The app-wide surface mix and the picture behind the app.
#[derive(Resource, Debug, Clone, Default)]
pub struct DodaiGlass {
    mix: Option<SurfaceFx>,
    picture: Option<Picture>,
    /// The backdrop in use (the picture, else the theme's mesh) and its sharp image.
    backdrop: Option<Picture>,
    sharp: Option<Handle<Image>>,
    /// Blurred copies by `(blur, saturation)` bits.
    soft: Vec<((u32, u32), Arc<dodai_frost::Reduced>)>,
    /// What the backdrop was made for: `Some(None)` the picture, `Some(Some(id))` theme `id`'s
    /// mesh, `None` not made yet.
    made_for: Option<Option<&'static str>>,
    /// Counts the backdrops made (panes redraw when it moves on).
    made: u32,
}

/// RGBA pixels and their size.
#[derive(Debug, Clone, PartialEq)]
struct Picture {
    rgba: Arc<Vec<u8>>,
    size: (usize, usize),
    mesh: bool,
}

impl DodaiGlass {
    /// Set the app-wide surface mix. Until it is set the design's own glass applies
    /// ([`Theme::default_surface`]: Dodai is frosted, the others not).
    pub fn set_mix(&mut self, fx: SurfaceFx) {
        if self.mix != Some(fx) {
            self.mix = Some(fx);
        }
    }

    /// The surface mix the app set (empty until it sets one, see [`mix_for`](Self::mix_for)).
    pub fn mix(&self) -> SurfaceFx {
        self.mix.unwrap_or(SurfaceFx::NONE)
    }

    /// The surface mix of `theme`'s surfaces: the app's, or the design's own glass.
    pub fn mix_for(&self, theme: &Theme) -> SurfaceFx {
        self.mix.unwrap_or_else(|| theme.default_surface())
    }

    /// Put `rgba` (`w`×`h`) behind the app; `None` goes back to the design's canvas mesh.
    pub fn set_picture(&mut self, picture: Option<(Vec<u8>, usize, usize)>) {
        self.picture = picture.map(|(px, w, h)| Picture {
            rgba: Arc::new(px),
            size: (w, h),
            mesh: false,
        });
        self.made_for = None;
    }

    /// The glass of `g` under `theme`.
    pub fn glass(&self, theme: &Theme, g: &UiGlass) -> Glass {
        let glass = self.glass_unfloored(theme, g);
        if g.floating {
            glass.floating()
        } else {
            glass
        }
    }

    fn glass_unfloored(&self, theme: &Theme, g: &UiGlass) -> Glass {
        match g.class {
            Some(c) => dodai_core::surface_glass(theme, c, &g.over, self.mix_for(theme)),
            None => {
                dodai_core::surface_glass(theme, SurfaceClass::Button, &g.over, SurfaceFx::NONE)
            }
        }
    }

    /// Whether frosted surfaces have a backdrop to blur.
    pub fn has_backdrop(&self) -> bool {
        self.backdrop.is_some()
    }

    /// The backdrop blurred for `g`.
    fn soft(&mut self, g: &Glass) -> Option<Arc<dodai_frost::Reduced>> {
        let b = self.backdrop.as_ref()?;
        let key = (g.blur.to_bits(), g.saturation.to_bits());
        if let Some((_, r)) = self.soft.iter().find(|(k, _)| *k == key) {
            return Some(r.clone());
        }
        let (w, h) = b.size;
        // The mesh is tiny and smooth: it is its own blur.
        let sigma = if b.mesh { 0.0 } else { g.blur };
        let mut r = dodai_frost::blurred(&b.rgba, w, h, sigma);
        dodai_frost::saturate(&mut r.pixels, g.saturation);
        let r = Arc::new(r);
        self.soft.push((key, r.clone()));
        Some(r)
    }
}

fn image(rgba: Vec<u8>, w: usize, h: usize) -> Image {
    Image::new(
        Extent3d {
            width: w as u32,
            height: h as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        rgba,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

/// Keep the backdrop in step with the picture and the theme, and draw it on [`UiBackdrop`]s.
pub(crate) fn sync_backdrop(
    theme: Res<DodaiTheme>,
    mut glass: ResMut<DodaiGlass>,
    mut images: ResMut<Assets<Image>>,
    mut q: Query<(&mut ImageNode, &mut Visibility), With<UiBackdrop>>,
) {
    let id = theme.theme().id();
    let want = if glass.picture.is_some() {
        None
    } else {
        Some(id)
    };
    if glass.made_for != Some(want) {
        // Marks the resource changed: surfaces re-resolve their fills.
        let g = &mut *glass;
        g.made_for = Some(want);
        g.made = g.made.wrapping_add(1);
        g.soft.clear();
        g.backdrop = g.picture.clone().or_else(|| {
            let (w, h) = dodai_core::MESH_SIZE;
            dodai_core::backdrop_mesh(theme.theme(), w, h).map(|px| Picture {
                rgba: Arc::new(px),
                size: (w, h),
                mesh: true,
            })
        });
        g.sharp = g
            .backdrop
            .as_ref()
            .map(|b| images.add(image(b.rgba.to_vec(), b.size.0, b.size.1)));
    }
    for (mut node, mut vis) in &mut q {
        match &glass.sharp {
            Some(h) => {
                if node.image != *h {
                    node.image = h.clone();
                    node.image_mode = bevy::ui::widget::NodeImageMode::Stretch;
                }
                vis.set_if_neq(Visibility::Inherited);
            }
            None => {
                vis.set_if_neq(Visibility::Hidden);
            }
        }
    }
}

/// Signed distance (px, positive inside) to the edge of a `w`×`h` box with per-corner radii
/// `[top-left, top-right, bottom-right, bottom-left]`.
fn rounded_distance(x: f32, y: f32, w: f32, h: f32, r: [f32; 4]) -> f32 {
    let d = x.min(w - x).min(y).min(h - y);
    let corner = |rad: f32, cx: f32, cy: f32, inside: bool| {
        if rad > 0.0 && inside {
            Some(rad - (x - cx).hypot(y - cy))
        } else {
            None
        }
    };
    let [tl, tr, br, bl] = r;
    corner(tl, tl, tl, x < tl && y < tl)
        .or(corner(tr, w - tr, tr, x > w - tr && y < tr))
        .or(corner(br, w - br, h - br, x > w - br && y > h - br))
        .or(corner(bl, bl, h - bl, x < bl && y > h - bl))
        .unwrap_or(d)
}

/// The frosted pane of one surface: `fill` over the blurred backdrop `soft` (at `amount`), for a
/// surface at `rect` (window px, `window` the window size) drawn `out` px large, cut to `inside`.
pub(crate) fn frost_pixels(
    soft: &dodai_frost::Reduced,
    rect: Rect,
    window: Vec2,
    out: (usize, usize),
    fill: dodai_core::Color,
    amount: f32,
    inside: impl Fn(f32, f32) -> bool,
) -> Vec<u8> {
    let (sw, sh) = soft.size;
    let (ow, oh) = out;
    let mut px = vec![0u8; ow * oh * 4];
    let fa = f32::from(fill.a) / 255.0;
    let out_a = fa + amount * (1.0 - fa);
    let fill_rgb = [fill.r, fill.g, fill.b].map(f32::from);
    for j in 0..oh {
        for i in 0..ow {
            let (u, v) = ((i as f32 + 0.5) / ow as f32, (j as f32 + 0.5) / oh as f32);
            if !inside(u * rect.width(), v * rect.height()) || out_a <= 0.0 {
                continue;
            }
            // Where this pixel is in the soft copy (the backdrop is stretched over the window).
            let wx = (rect.min.x + u * rect.width()) / window.x * sw as f32 - 0.5;
            let wy = (rect.min.y + v * rect.height()) / window.y * sh as f32 - 0.5;
            let blur = bilinear(&soft.pixels, sw, sh, wx, wy);
            let o = (j * ow + i) * 4;
            for c in 0..3 {
                let v = (fill_rgb[c] * fa + blur[c] * amount * (1.0 - fa)) / out_a;
                px[o + c] = v.round().clamp(0.0, 255.0) as u8;
            }
            px[o + 3] = (out_a * 255.0).round() as u8;
        }
    }
    px
}

fn bilinear(px: &[u8], w: usize, h: usize, x: f32, y: f32) -> [f32; 3] {
    let x = x.clamp(0.0, (w - 1) as f32);
    let y = y.clamp(0.0, (h - 1) as f32);
    let (x0, y0) = (x.floor() as usize, y.floor() as usize);
    let (x1, y1) = ((x0 + 1).min(w - 1), (y0 + 1).min(h - 1));
    let (fx, fy) = (x - x0 as f32, y - y0 as f32);
    let at = |x: usize, y: usize, c: usize| f32::from(px[(y * w + x) * 4 + c]);
    std::array::from_fn(|c| {
        let top = at(x0, y0, c) * (1.0 - fx) + at(x1, y0, c) * fx;
        let bottom = at(x0, y1, c) * (1.0 - fx) + at(x1, y1, c) * fx;
        top * (1.0 - fy) + bottom * fy
    })
}

/// Draw (or remove) the frosted pane of every [`UiGlass`] surface after layout.
#[allow(clippy::type_complexity)]
pub(crate) fn frost_layers(
    mut commands: Commands,
    theme: Res<DodaiTheme>,
    mut glass: ResMut<DodaiGlass>,
    mut images: ResMut<Assets<Image>>,
    window: Option<Single<&Window, With<PrimaryWindow>>>,
    surfaces: Query<(
        Entity,
        &UiGlass,
        &Surface,
        &ComputedNode,
        &UiGlobalTransform,
        Option<&SurfaceCrop>,
        Option<&Children>,
    )>,
    mut panes: Query<(&mut FrostImage, &mut ImageNode, &mut Node)>,
) {
    let Some(window) = window else {
        return;
    };
    let win = Vec2::new(
        window.physical_width() as f32,
        window.physical_height() as f32,
    );
    let t = theme.theme();
    for (e, ug, s, cn, tf, crop, children) in &surfaces {
        let pane = children.and_then(|c| c.iter().find(|c| panes.contains(*c)));
        let g = glass.glass(t, ug);
        let fill = s.fill.filter(|_| g.frosted() && glass.has_backdrop());
        let (Some(fill), true) = (fill, cn.size.x >= 1.0 && cn.size.y >= 1.0) else {
            if let Some(p) = pane {
                commands.entity(p).despawn();
            }
            continue;
        };
        let size = cn.size;
        let centre = tf.translation;
        let rect = Rect::from_center_size(centre, size);
        // Off screen: nothing to draw until it scrolls in.
        if rect.max.y < 0.0 || rect.min.y > win.y || rect.max.x < 0.0 || rect.min.x > win.x {
            continue;
        }
        let fill = g.color(fill);
        let key = vec![
            rect.min.x.round() as u32,
            rect.min.y.round() as i32 as u32,
            size.x.round() as u32,
            size.y.round() as u32,
            win.x as u32,
            win.y as u32,
            u32::from_le_bytes([fill.r, fill.g, fill.b, fill.a]),
            g.amount.to_bits(),
            g.blur.to_bits(),
            glass.made,
        ];
        if let Some(p) = pane {
            if panes.get(p).is_ok_and(|(f, ..)| f.key == key) {
                continue;
            }
        }
        // The blurred copies are a cache: making one is no change.
        let Some(soft) = glass.bypass_change_detection().soft(&g) else {
            continue;
        };
        // The pane is drawn at half the surface's pixels: it is a blur.
        let out = (
            (size.x / 2.0).ceil().max(1.0) as usize,
            (size.y / 2.0).ceil().max(1.0) as usize,
        );
        let bw = cn.border.min_inset.x.max(0.0);
        let r = resolve_cropped(t, s, crop.and_then(|c| c.0));
        let radii = [
            cn.border_radius.top_left.x,
            cn.border_radius.top_right.x,
            cn.border_radius.bottom_right.x,
            cn.border_radius.bottom_left.x,
        ];
        let (w, h) = (size.x, size.y);
        let inside = move |x: f32, y: f32| match r.paint {
            Paint::Chamfer {
                chamfer,
                corners,
                scoop,
            } => cut_distance(x, y, w, h, chamfer, corners, scoop) > bw,
            _ => rounded_distance(x, y, w, h, radii) > bw,
        };
        let px = frost_pixels(&soft, rect, win, out, fill, g.amount, inside);
        let handle = images.add(image(px, out.0, out.1));
        let inv = cn.inverse_scale_factor;
        let node = Node {
            position_type: PositionType::Absolute,
            left: Val::Px(-cn.border.min_inset.x * inv),
            top: Val::Px(-cn.border.min_inset.y * inv),
            width: Val::Px(w * inv),
            height: Val::Px(h * inv),
            ..default()
        };
        let mut img = ImageNode::new(handle);
        img.image_mode = bevy::ui::widget::NodeImageMode::Stretch;
        match pane.and_then(|p| panes.get_mut(p).ok()) {
            Some((mut f, mut i, mut n)) => {
                f.key = key;
                *i = img;
                *n = node;
            }
            None => {
                let child = commands
                    .spawn((FrostImage { key }, img, node, Pickable::IGNORE))
                    .id();
                commands.entity(e).insert_children(0, &[child]);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cyberpunk() -> Theme {
        dodai_core::themes::all()
            .into_iter()
            .find(|t| t.meta.family == "cyberpunk")
            .expect("cyberpunk")
    }

    #[test]
    fn the_mix_reaches_classes_and_an_own_surface_any_surface() {
        let t = cyberpunk();
        let mut g = DodaiGlass::default();
        let panel = UiGlass::of(SurfaceClass::Panel);
        assert!(g.glass(&t, &panel).is_solid());
        g.set_mix(SurfaceFx {
            frost: 1.0,
            ..SurfaceFx::NONE
        });
        assert!(g.glass(&t, &panel).frosted());
        let own = UiGlass::default().with(dodai_core::Surface::new().opacity(0.5));
        assert_eq!(g.glass(&t, &own).alpha(1.0), 0.5);
    }

    #[test]
    fn the_pane_is_the_fill_over_the_blur_and_cut_to_the_shape() {
        let soft = dodai_frost::Reduced {
            pixels: [0u8, 0, 255, 255].repeat(4),
            size: (2, 2),
            full: (2, 2),
        };
        let fill = dodai_core::Color::rgba(255, 0, 0, 128);
        let rect = Rect::new(0.0, 0.0, 10.0, 10.0);
        let px = frost_pixels(&soft, rect, Vec2::splat(10.0), (2, 1), fill, 1.0, |x, _| {
            x < 5.0
        });
        assert_eq!(px[3], 255, "a frosted pane is opaque");
        assert!(px[0] > 100 && px[2] > 100, "red over blue");
        assert_eq!(&px[4..8], &[0, 0, 0, 0], "cut away outside the shape");
    }

    #[test]
    fn rounded_corners_cut_the_pane() {
        let r = [4.0; 4];
        assert!(rounded_distance(0.5, 0.5, 20.0, 10.0, r) < 0.0);
        assert!(rounded_distance(10.0, 5.0, 20.0, 10.0, r) > 0.0);
    }
}
