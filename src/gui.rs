//! Because GUI needs to have access to the rendering context and to the window's input, and
//! three-rs controls both of those, GUI backends open three-rs up to GUI libraries. This allows
//! one to easily use any GUI library that has a three-rs gui backend implemented for it.

use gfx::{handle::RenderTargetView, CommandBuffer, Encoder, Factory};
use render::ColorFormat;

use render::BackendResources;

/// A GuiBackend typically contains a renderer and whatever else is needed to draw GUI over
/// everything else that three-rs renders. GuiBackends also handle input and can even prevent input
/// from being sent to three-rs. Finally, they may store other kind of structs that are necessary
/// to store the resources or ids or whatever else is needed to facilitate the creation of the GUI,
/// if necessary.
pub trait GuiBackend {
    /// Initialize the GuiBackend. Each gfx rendering backend, like OpenGL, has its own rendering
    /// resources. Since a large part of GUI is rendering, these are necessary for the creation of
    /// almost all rendering backends.
    fn init<F: Factory<BackendResources>>(
        factory: &mut F,
        rtv: RenderTargetView<BackendResources, ColorFormat>,
    ) -> Self;

    /// Draw the GUI. For most GUI's, this will simply call a rendering backend provided by the
    /// GUI library authors for `gfx-pre II`; that's what three-rs uses in the background.
    fn render<F: Factory<BackendResources>, B: CommandBuffer<BackendResources>>(
        &mut self,
        factory: &mut F,
        encoder: &mut Encoder<BackendResources, B>,
        size: glutin::dpi::LogicalSize,
        scale: f64,
    );

    /// Each backend is handed each event right before the GUI gets them.
    fn process_event(&mut self, event: &glutin::Event);

    /// Called right before polling for events starts.
    /// This is for GUIs backends that may be based on C and rely on state being held.
    /// In some GUI backends, it would make sense for this to be a no-op.
    fn input_begin(&mut self);

    /// Called right after polling for events ends.
    /// See `input_end`.
    fn input_end(&mut self);

    /// This allows the GUI backend to prevent three.js from receiving input.
    /// When manipulating toggles and dragging windows in GUI, you don't want three-rs to send
    /// these to your camera controller. The same is often true for text inputs. In the context of
    /// a game, as an example, if they're typing something into a chat's text input, you wouldn't
    /// want that input to also move them in-game whenever one of WSAD are typed as part of the message.
    /// pressed.
    fn captured_input(&self) -> bool;
}

/// A GUI backend that can be used when no GUI is desired. This helps us navigate the type system
/// when GUI isn't always necessary. None of these methods actually do anything, and NoBackend has
/// no fields.
//#[Debug + Clone]
pub struct NoBackend;
impl GuiBackend for NoBackend {
    fn init<F: Factory<BackendResources>>(
        _: &mut F,
        _: RenderTargetView<BackendResources, ColorFormat>,
    ) -> Self {
        NoBackend
    }

    fn render<F: Factory<BackendResources>, B: CommandBuffer<BackendResources>>(
        &mut self,
        _: &mut F,
        _: &mut Encoder<BackendResources, B>,
        _: glutin::dpi::LogicalSize,
        _: f64,
    ) {
    }

    fn process_event(&mut self, _: &glutin::Event) {}
    fn input_begin(&mut self) {}
    fn input_end(&mut self) {}
    fn captured_input(&self) -> bool {false}
}

#[cfg(feature = "nuklear")]
pub use self::nuklear_backend::NuklearBackend;
#[cfg(feature = "nuklear")]
/// Facilitates rendering Nuklear UI over a three-rs scene.
/// Made available through the `--nuklear` feature.
/// Note that this requires nightly Rust.
pub mod nuklear_backend {
    use super::GuiBackend;
    use render::{BackendResources, ColorFormat};
    
    use std::fs::*;
    use std::io::BufReader;

    use nuklear::*;
    use nuklear_backend_gfx::{Drawer, GfxBackend};

    use gfx::{handle::RenderTargetView, CommandBuffer, Encoder, Factory};

    const MAX_VERTEX_MEMORY: usize = 512 * 1024;
    const MAX_ELEMENT_MEMORY: usize = 128 * 1024;
    const MAX_COMMANDS_MEMORY: usize = 64 * 1024;

    /// The Nuklear backend needs to possess the font_atlas and other font and image creation
    /// resources so that it can render all of the UI, but it's also imperative for the user to be
    /// able to instantiate and use their own fonts and images. To facilitate this, we have the
    /// MediaStorage trait. The user can implement it on a struct which they want to store all of
    /// their fonts and other media, and then pass it to `Window::new<gui::NuklearBackend<Media>>`
    /// like so.
    pub trait MediaStorage {
        /// This method hands the user everything they need to allocate fonts and images for use in
        /// their GUI, wrapped up in a handy ResourceLoader to do away with the boilerplate. One
        /// can also access all of the important fields on the loader itself to use Nuklear's
        /// default loading system which allows for much more customization.
        fn build<F: Factory<BackendResources>>(f: &mut F, load: ResourceLoader) -> Self;
        /// Nuklear requires the user to supply a font in order to create the GUI context. This
        /// trait is intended to allow the user to have almost complete control over their font
        /// initialization, but Nuklear requires at least one font to be provided.
        fn first_font(&self, atlas: &FontAtlas) -> UserFont;
    }

    /// The Resources loader is passed to the MediaStorage implementation the user provides.
    /// It has methods to simplify loading fonts and images.
    pub struct ResourceLoader<'a> {
        /// The FontAtlas is needed to register new fonts in Nuklear's system. Therefore, it is
        /// provided for the construction of the struct implementing MediaStorage.
        pub font_atlas: &'a mut FontAtlas,
        /// Needed to add textures
        pub drawer: &'a mut Drawer<BackendResources>,
    }
    impl<'a> ResourceLoader<'a> {
        /// Load an image from the provided filename.
        /// Only supports the `.png` format.
        pub fn image<F: Factory<BackendResources>>(
            &mut self,
            factory: &mut F,
            filename: &'a str,
        ) -> nuklear::Image {
            let img = image::load(BufReader::new(File::open(filename).unwrap()), image::PNG).unwrap().to_rgba();

            let (w, h) = img.dimensions();
            let mut hnd = self.drawer.add_texture(factory, &img, w, h);

            Image::with_id(hnd.id().unwrap())
        }

        /// Load a font at a given size.
        /// This mutates the state on the FontConfig provided, meaning that if another font is
        /// allocated afterwards it will have the same size as the size specified here.
        pub fn font_with_size(
            &mut self,
            cfg: &mut FontConfig,
            size: f32,
        ) -> FontID {
            cfg.set_ttf_data_owned_by_atlas(false);
            cfg.set_size(size);
            self.font_atlas.add_font_with_config(cfg).unwrap()
        }
    }

    /// Facilitates rendering Nuklear UI over a three-rs scene.
    pub struct NuklearBackend<M: MediaStorage> {
        /// Nuklear renderer
        drawer: Drawer<BackendResources>,
        /// Mouse movement on the x axis
        mx: i32,
        /// Mouse movement on the y axis
        my: i32,
        /// Handles Nuklear memory
        pub ctx: Context,
        /// This config allows for the configuration of various Nuklear rendering parameters.
        pub config: ConvertConfig,
        /// All of the fonts and their corresponding IDs are stored here.
        pub font_atlas: FontAtlas,
        /// The images generated to facilitate font rendering are stored here.
        pub font_tex: Handle,
        /// This struct stores the fonts and images that the user wishes to use in their GUI.
        pub media: M,
    }
    impl<M: MediaStorage> Drop for NuklearBackend<M> {
        fn drop(&mut self) {
            unsafe {
                self.font_tex = ::std::mem::zeroed();
            }
        }
    }
    impl<M: MediaStorage> GuiBackend for NuklearBackend<M> {
        fn init<F: Factory<BackendResources>>(
            factory: &mut F,
            rtv: RenderTargetView<BackendResources, ColorFormat>,
        ) -> Self {
            let mut allo = Allocator::new_vec();
            let mut drawer = Drawer::new(factory, rtv, 36, MAX_VERTEX_MEMORY, MAX_ELEMENT_MEMORY, Buffer::with_size(&mut allo, MAX_COMMANDS_MEMORY), GfxBackend::OpenGlsl150);
            let mut atlas = FontAtlas::new(&mut allo);

            let load = ResourceLoader {
                font_atlas: &mut atlas,
                drawer: &mut drawer,
            };

            let media = <M>::build::<F>(factory, load);

            let font_tex = {
                let (b, w, h) = atlas.bake(FontAtlasFormat::Rgba32);
                drawer.add_texture(factory, b, w, h)
            };

            let mut null = DrawNullTexture::default();

            atlas.end(font_tex, Some(&mut null));
            //atlas.cleanup();

            let ctx = Context::new(&mut allo, &media.first_font(&atlas));

            let mut config = ConvertConfig::default();
            config.set_null(null.clone());

            Self { drawer, ctx, config, media, font_tex, font_atlas: atlas, mx: 0, my: 0}
        }

        fn render<F: gfx::Factory<BackendResources>, B: CommandBuffer<BackendResources>>(
            &mut self,
            factory: &mut F,
            encoder: &mut Encoder<BackendResources, B>,
            size: glutin::dpi::LogicalSize,
            scale: f64,
        ) {
            let scale = Vec2 { x: scale as f32, y: scale as f32 };
            self.drawer.draw(&mut self.ctx, &mut self.config, encoder, factory, size.width as u32, size.height as u32, scale);
        }

        fn input_begin(&mut self) {
            self.ctx.input_begin();
        }

        fn input_end(&mut self) {
            self.ctx.input_end();
        }

        fn captured_input(&self) -> bool {
            self.ctx.item_is_any_active()
        }

        // shamelessly stolen from
        // https://github.com/snuk182/nuklear-test/blob/master/src/main.rs
        fn process_event(&mut self, event: &glutin::Event) {
            if let glutin::Event::WindowEvent { event, .. } = event {
                match event {
                    glutin::WindowEvent::ReceivedCharacter(c) => {
                        self.ctx.input_unicode(*c);
                    }
                    glutin::WindowEvent::KeyboardInput {
                        input: glutin::KeyboardInput { state, virtual_keycode, .. },
                        ..
                    } => {
                        if let Some(k) = virtual_keycode {
                            let key = match k {
                                glutin::VirtualKeyCode::Back => Key::Backspace,
                                glutin::VirtualKeyCode::Delete => Key::Del,
                                glutin::VirtualKeyCode::Up => Key::Up,
                                glutin::VirtualKeyCode::Down => Key::Down,
                                glutin::VirtualKeyCode::Left => Key::Left,
                                glutin::VirtualKeyCode::Right => Key::Right,
                                _ => Key::None,
                            };

                            self.ctx.input_key(key, *state == glutin::ElementState::Pressed);
                        }
                    }
                    glutin::WindowEvent::CursorMoved { position: glutin::dpi::LogicalPosition{ x, y }, .. } => {
                        self.mx = *x as i32;
                        self.my = *y as i32;
                        self.ctx.input_motion(*x as i32, *y as i32);
                    }
                    glutin::WindowEvent::MouseInput { state, button, .. } => {
                        let button = match button {
                            glutin::MouseButton::Left => Button::Left,
                            glutin::MouseButton::Middle => Button::Middle,
                            glutin::MouseButton::Right => Button::Right,
                            _ => Button::Max,
                        };

                        self.ctx.input_button(button, self.mx, self.my, *state == glutin::ElementState::Pressed)
                    }
                    glutin::WindowEvent::MouseWheel { delta, .. } => {
                        if let glutin::MouseScrollDelta::LineDelta(x, y) = delta {
                            self.ctx.input_scroll(Vec2 { x: x * 22f32, y: y * 22f32 });
                        }
                    }
                    _ => (),
                }
            }
        }
    }
}
/*
#[cfg(feature = "conrod")]
pub mod conrod {
    use super::*;
    use gfx::{Resources, Factory, handle::ShaderResourceView, format::{Formatted, TextureFormat}};
    use std::fmt::Debug;

    use conrod_core::image::Map as ImgMap;

    //#[Debug + Clone]
    pub struct ConrodBackend<'a, T: TextureFormat + Debug + Clone, R: Resources> {
        image_map: ImgMap<(ShaderResourceView<R, <T as Formatted>::View>, (u32, u32))>,
        renderer: conrod_gfx::Renderer<'a, R>,
    }
    impl GuiBackend for ConrodBackend<'a, T, R> {
        fn init<R: Resources, F: Factory<R>>(
            factory: &mut F,
            rtv: &RenderTargetView<R, ColorFormat>,
            dpi: f64,
        ) -> Self {
            let renderer = conrod_gfx::Renderer::new(factory, rtv, dpi).unwrap();
            //let image_map = ImgMap::new::<(ShaderResourceView<Re, <T as Formatted>::View>, (u32, u32))>();
            let image_map = ImgMap::new();
            Self {
                image_map,
                renderer,
            }
        }
    }
}*/
