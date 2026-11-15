//! See-through fills and frosted glass: the app-wide surface mix and a component's own
//! [`Surface`] make a fill as transparent as [`dodai_core::Glass`] says; with the `frost` feature a
//! blurred copy of the app's backdrop (a picture the app sets, else the design's canvas mesh) is
//! drawn under frosted surfaces, lined up with the sharp one behind the app.

use dodai_core::{Glass, Surface, SurfaceClass, SurfaceFx};
use iced::{Element, Size, Vector};

use crate::IcedTheme;

/// What the glass of an [`IcedTheme`] needs besides the theme.
#[derive(Debug, Clone)]
pub(crate) struct GlassState {
    /// The app-wide surface mix (`None`: the design's own, see [`Theme::default_surface`]).
    pub(crate) mix: Option<SurfaceFx>,
    /// Glass every button this theme draws takes (see [`IcedTheme::with_glass`]).
    pub(crate) own: Option<Glass>,
    /// Window size and the scroll offset of the page frosted surfaces sit on.
    pub(crate) window: Size,
    pub(crate) scroll: Vector,
    #[cfg(feature = "frost")]
    pub(crate) backdrop: Option<frost::Backdrop>,
    /// The app set a picture (else the backdrop follows the design's mesh).
    #[cfg(feature = "frost")]
    pub(crate) picture: bool,
}

impl Default for GlassState {
    fn default() -> Self {
        Self {
            mix: None,
            own: None,
            window: Size::new(1280.0, 800.0),
            scroll: Vector::ZERO,
            #[cfg(feature = "frost")]
            backdrop: None,
            #[cfg(feature = "frost")]
            picture: false,
        }
    }
}

impl IcedTheme {
    /// Set the app-wide surface mix: containers it frosts turn see-through. Until it is set the
    /// design's own glass applies ([`Theme::default_surface`]: Dodai is frosted, the others not).
    pub fn set_surface_mix(&mut self, fx: SurfaceFx) {
        self.glass.mix = Some(fx);
    }

    /// The app-wide surface mix.
    pub fn surface_mix(&self) -> SurfaceFx {
        self.glass
            .mix
            .unwrap_or_else(|| self.theme.default_surface())
    }

    /// The glass of a `class` surface with the component's own `over` on top of the mix.
    pub fn glass(&self, class: SurfaceClass, over: &Surface) -> Glass {
        dodai_core::surface_glass(&self.theme, class, over, self.surface_mix())
    }

    /// The glass of an input field under the app-wide mix (see [`field`](Self::field)).
    pub fn field_glass(&self) -> Glass {
        self.glass(SurfaceClass::Input, &Surface::INHERIT)
    }

    /// A text input or pick list (styled with [`text_input`](Self::text_input) or
    /// [`pick_list`](Self::pick_list)) on the glass the surface mix gives input fields: a blurred
    /// copy of the backdrop under its see-through fill. Unchanged without the mix.
    pub fn field<'a, Message: 'a>(
        &self,
        content: impl Into<Element<'a, Message>>,
    ) -> Element<'a, Message> {
        self.frosted_round(content, self.field_glass(), self.radius(), 0.0)
    }

    /// This theme with the glass of a button's own `over` surface (frost, fill opacity): the
    /// buttons it draws get see-through fills; put them in [`frosted`](Self::frosted) with
    /// [`button_glass`](Self::button_glass) for the blur under them.
    pub fn with_glass(&self, over: &Surface) -> Self {
        let mut lt = self.clone();
        lt.glass.own = Some(self.button_glass(over)).filter(|g| !g.is_solid());
        lt
    }

    /// The glass of a button with its own `over` surface.
    pub fn button_glass(&self, over: &Surface) -> Glass {
        self.glass(SurfaceClass::Button, over)
    }

    /// Tell the glass the window size and how far the page under frosted surfaces is scrolled,
    /// so their blurred backdrop lines up with the sharp one (call it when either changes).
    pub fn set_viewport(&mut self, window: Size, scroll: Vector) {
        self.glass.window = window;
        self.glass.scroll = scroll;
    }

    /// `content` (a surface drawn with `glass`'s see-through fill) over a blurred copy of the
    /// backdrop when the glass is frosted and the `frost` feature is on; else `content` as it is.
    pub fn frosted<'a, Message: 'a>(
        &self,
        content: impl Into<Element<'a, Message>>,
        glass: Glass,
    ) -> Element<'a, Message> {
        // A canvas-drawn (chamfered) control keeps room for its glow around the shape.
        let inset = if self.theme.shape.kind() == dodai_core::ShapeKind::Chamfered {
            crate::geometry::margin(self.theme.effects.glow_radius(), 0.0)
        } else {
            0.0
        };
        self.frosted_round(content, glass, self.radius(), inset)
    }

    /// The glass inset by `inset` px (room a canvas frame keeps for glow and shadow) of a box
    /// drawn with `s` (see [`IcedTheme::shaped_box`]).
    pub(crate) fn box_inset(&self, s: &crate::BoxStyle) -> f32 {
        if self.theme.shape.kind() == dodai_core::ShapeKind::Chamfered {
            self.box_frame(s).margin()
        } else {
            0.0
        }
    }

    /// [`frosted`](Self::frosted) with the glass cut to corners of `radius`, `inset` px inside
    /// the content's bounds.
    pub(crate) fn frosted_round<'a, Message: 'a>(
        &self,
        content: impl Into<Element<'a, Message>>,
        glass: Glass,
        radius: f32,
        inset: f32,
    ) -> Element<'a, Message> {
        #[cfg(feature = "frost")]
        if glass.frosted() {
            if let Some(b) = self.backdrop_for(&glass) {
                return frost::FrostUnder {
                    content: content.into(),
                    image: b,
                    window: self.glass.window,
                    scroll: self.glass.scroll,
                    radius,
                    inset,
                    opacity: glass.amount,
                }
                .into();
            }
        }
        let _ = (glass, radius, inset);
        content.into()
    }
}

#[cfg(feature = "frost")]
pub(crate) mod frost {
    //! The blurred backdrop and the widget that draws it under a surface.

    use std::sync::{Arc, Mutex};

    use iced::advanced::image::{self as adv_image, Renderer as _};
    use iced::advanced::layout::{self, Layout};
    use iced::advanced::widget::{tree, Operation, Tree, Widget};
    use iced::advanced::Renderer as _;
    use iced::advanced::{mouse, overlay, renderer, Clipboard, Shell};
    use iced::widget::image::Handle;
    use iced::{Element, Event, Length, Rectangle, Size, Vector};

    use crate::IcedTheme;

    /// A blurred copy by `(blur, saturation)` bits.
    type SoftCopy = ((u32, u32), Handle);

    /// The picture behind the app and its blurred copies (one per blur, made on first use).
    #[derive(Debug, Clone)]
    pub(crate) struct Backdrop {
        rgba: Arc<Vec<u8>>,
        size: (usize, usize),
        /// The sharp picture.
        pub(crate) sharp: Handle,
        /// `(blur, saturation)` → blurred copy.
        soft: Arc<Mutex<Vec<SoftCopy>>>,
        /// The design's mesh: tiny and smooth, it is its own blur.
        mesh: bool,
    }

    impl Backdrop {
        fn new(rgba: Vec<u8>, w: usize, h: usize, mesh: bool) -> Self {
            Self {
                sharp: Handle::from_rgba(w as u32, h as u32, rgba.clone()),
                rgba: Arc::new(rgba),
                size: (w, h),
                soft: Arc::default(),
                mesh,
            }
        }

        /// The copy blurred by `blur` px with `saturation`.
        fn soft(&self, blur: f32, saturation: f32) -> Handle {
            if self.mesh {
                return self.sharp.clone();
            }
            let key = (blur.to_bits(), saturation.to_bits());
            let Ok(mut cache) = self.soft.lock() else {
                return self.sharp.clone();
            };
            if let Some((_, h)) = cache.iter().find(|(k, _)| *k == key) {
                return h.clone();
            }
            let (w, h) = self.size;
            let mut r = dodai_frost::blurred(&self.rgba, w, h, blur);
            dodai_frost::saturate(&mut r.pixels, saturation);
            let handle = Handle::from_rgba(r.size.0 as u32, r.size.1 as u32, r.pixels);
            cache.push((key, handle.clone()));
            handle
        }
    }

    impl IcedTheme {
        /// Put `rgba` (`w`×`h`) behind the app: [`backdrop_view`](Self::backdrop_view) draws it
        /// and frosted surfaces blur it. `None` goes back to the design's canvas mesh (nothing
        /// for a design with a flat canvas).
        pub fn set_backdrop(&mut self, picture: Option<(Vec<u8>, usize, usize)>) {
            self.glass.backdrop = picture.map(|(px, w, h)| Backdrop::new(px, w, h, false));
            self.glass.picture = self.glass.backdrop.is_some();
            if !self.glass.picture {
                self.refresh_mesh();
            }
        }

        /// The design's canvas mesh as the backdrop, unless the app set a picture.
        pub(crate) fn refresh_mesh(&mut self) {
            if self.glass.picture {
                return;
            }
            let (w, h) = dodai_core::MESH_SIZE;
            self.glass.backdrop = dodai_core::backdrop_mesh(&self.theme, w, h)
                .map(|px| Backdrop::new(px, w, h, true));
        }

        /// The backdrop stretched over its parent, to put under the whole app (`None` for a
        /// flat canvas without a picture).
        pub fn backdrop_view<'a, Message: 'a>(&self) -> Option<Element<'a, Message>> {
            let b = self.glass.backdrop.as_ref()?;
            Some(
                iced::widget::image(b.sharp.clone())
                    .content_fit(iced::ContentFit::Fill)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into(),
            )
        }

        pub(crate) fn backdrop_for(&self, g: &dodai_core::Glass) -> Option<Handle> {
            let b = self.glass.backdrop.as_ref()?;
            Some(b.soft(g.blur, g.saturation))
        }
    }

    /// `content` over the blurred backdrop, cropped to the content's bounds.
    pub(crate) struct FrostUnder<'a, Message> {
        pub(crate) content: Element<'a, Message>,
        pub(crate) image: Handle,
        pub(crate) window: Size,
        pub(crate) scroll: Vector,
        pub(crate) radius: f32,
        pub(crate) inset: f32,
        pub(crate) opacity: f32,
    }

    impl<Message> Widget<Message, iced::Theme, iced::Renderer> for FrostUnder<'_, Message> {
        fn tag(&self) -> tree::Tag {
            self.content.as_widget().tag()
        }

        fn state(&self) -> tree::State {
            self.content.as_widget().state()
        }

        fn children(&self) -> Vec<Tree> {
            self.content.as_widget().children()
        }

        fn diff(&self, tree: &mut Tree) {
            self.content.as_widget().diff(tree);
        }

        fn size(&self) -> Size<Length> {
            self.content.as_widget().size()
        }

        fn layout(
            &mut self,
            tree: &mut Tree,
            renderer: &iced::Renderer,
            limits: &layout::Limits,
        ) -> layout::Node {
            self.content.as_widget_mut().layout(tree, renderer, limits)
        }

        fn operate(
            &mut self,
            tree: &mut Tree,
            layout: Layout<'_>,
            renderer: &iced::Renderer,
            operation: &mut dyn Operation,
        ) {
            self.content
                .as_widget_mut()
                .operate(tree, layout, renderer, operation);
        }

        fn update(
            &mut self,
            tree: &mut Tree,
            event: &Event,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            renderer: &iced::Renderer,
            clipboard: &mut dyn Clipboard,
            shell: &mut Shell<'_, Message>,
            viewport: &Rectangle,
        ) {
            self.content.as_widget_mut().update(
                tree, event, layout, cursor, renderer, clipboard, shell, viewport,
            );
        }

        fn mouse_interaction(
            &self,
            tree: &Tree,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            viewport: &Rectangle,
            renderer: &iced::Renderer,
        ) -> mouse::Interaction {
            self.content
                .as_widget()
                .mouse_interaction(tree, layout, cursor, viewport, renderer)
        }

        fn draw(
            &self,
            tree: &Tree,
            renderer: &mut iced::Renderer,
            theme: &iced::Theme,
            style: &renderer::Style,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            viewport: &Rectangle,
        ) {
            // The page is drawn translated by minus its scroll: the window's origin sits at
            // `scroll` here, where the sharp backdrop starts.
            let window =
                Rectangle::new(iced::Point::new(self.scroll.x, self.scroll.y), self.window);
            let image = adv_image::Image {
                handle: self.image.clone(),
                filter_method: adv_image::FilterMethod::Linear,
                rotation: iced::Radians(0.0),
                border_radius: self.radius.into(),
                opacity: self.opacity,
                snap: false,
            };
            // On layers of their own: iced draws a layer's images over its quads, and not every
            // backend honours an image's clip bounds.
            let bounds = layout.bounds().shrink(self.inset);
            if let Some(clip) = bounds.intersection(viewport) {
                renderer.with_layer(clip, |renderer| {
                    renderer.draw_image(image, window, bounds);
                });
            }
            renderer.with_layer(*viewport, |renderer| {
                self.content
                    .as_widget()
                    .draw(tree, renderer, theme, style, layout, cursor, viewport);
            });
        }

        fn overlay<'b>(
            &'b mut self,
            tree: &'b mut Tree,
            layout: Layout<'b>,
            renderer: &iced::Renderer,
            viewport: &Rectangle,
            translation: Vector,
        ) -> Option<overlay::Element<'b, Message, iced::Theme, iced::Renderer>> {
            self.content
                .as_widget_mut()
                .overlay(tree, layout, renderer, viewport, translation)
        }
    }

    impl<'a, Message: 'a> From<FrostUnder<'a, Message>> for Element<'a, Message> {
        fn from(w: FrostUnder<'a, Message>) -> Self {
            Element::new(w)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::*;

    #[test]
    fn the_mix_and_an_own_surface_reach_the_glass() {
        let mut lt = IcedTheme::new(cyberpunk());
        assert!(lt.glass(SurfaceClass::Panel, &Surface::INHERIT).is_solid());
        lt.set_surface_mix(SurfaceFx {
            frost: 1.0,
            ..SurfaceFx::NONE
        });
        assert!(lt.glass(SurfaceClass::Panel, &Surface::INHERIT).frosted());
        let own = lt.glass(SurfaceClass::Button, &Surface::new().opacity(0.5));
        assert_eq!(own.alpha(1.0), 0.5);
    }

    #[cfg(feature = "frost")]
    #[test]
    fn the_backdrop_is_the_picture_else_the_designs_mesh() {
        let mut lt = IcedTheme::new(cyberpunk());
        assert!(lt.backdrop_view::<()>().is_none(), "a flat canvas has none");
        lt.set_backdrop(Some((vec![255; 8 * 4 * 4], 8, 4)));
        assert!(lt.backdrop_view::<()>().is_some());
        let g = lt.glass(SurfaceClass::Panel, &Surface::new().frost(1.0));
        assert!(lt.backdrop_for(&g).is_some());
        lt.set_backdrop(None);
        assert!(lt.backdrop_view::<()>().is_none());
    }
}
