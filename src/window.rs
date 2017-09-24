//! Primitives for creating and controlling [`Window`](struct.Window.html).

use std::path::{Path, PathBuf};

use glutin;
use glutin::GlContext;
use mint;

use camera::Camera;
use factory::Factory;
use input::Input;
use render::Renderer;
use scene::Scene;


/// `Window` is the core entity of every `three-rs` application.
///
/// It provides [user input](struct.Window.html#method.update),
/// [`Factory`](struct.Factory.html) and [`Renderer`](struct.Renderer.html).
pub struct Window {
    event_loop: glutin::EventsLoop,
    window: glutin::GlWindow,
    /// See [`Input`](struct.Input.html).
    pub input: Input,
    /// See [`Renderer`](struct.Renderer.html).
    pub renderer: Renderer,
    /// See [`Factory`](struct.Factory.html).
    pub factory: Factory,
    /// See [`Scene`](struct.Scene.html).
    pub scene: Scene,
}

/// Builder for creating new [`Window`](struct.Window.html) with desired parameters.
pub struct Builder {
    dimensions: (u32, u32),
    fullscreen: bool,
    multisampling: u16,
    shader_path: PathBuf,
    title: String,
    vsync: bool,
}

impl Builder {
    /// Set the size of the viewport (the resolution) in pixels. Defaults to 1024x768.
    pub fn dimensions(
        &mut self,
        width: u32,
        height: u32,
    ) -> &mut Self {
        self.dimensions = (width, height);
        self
    }

    /// Whether enable fullscreen mode or not. Defauls to `false`.
    pub fn fullscreen(
        &mut self,
        option: bool,
    ) -> &mut Self {
        self.fullscreen = option;
        self
    }

    /// Sets the multisampling level to request. A value of `0` indicates that multisampling must
    /// not be enabled. Must be the power of 2. Defaults to `0`.
    pub fn multisampling(
        &mut self,
        option: u16,
    ) -> &mut Self {
        self.multisampling = option;
        self
    }

    /// Whether to enable vertical synchronization or not. Defaults to `true`.
    pub fn vsync(
        &mut self,
        option: bool,
    ) -> &mut Self {
        self.vsync = option;
        self
    }

    /// Create new `Window` with desired parameters.
    pub fn build(&mut self) -> Window {
        use glutin::get_primary_monitor;

        let builder = if self.fullscreen {
            glutin::WindowBuilder::new().with_fullscreen(get_primary_monitor())
        } else {
            glutin::WindowBuilder::new()
        };

        let builder = builder
            .clone()
            .with_dimensions(self.dimensions.0, self.dimensions.1)
            .with_title(self.title.clone());

        let context = glutin::ContextBuilder::new()
            .with_vsync(self.vsync)
            .with_multisampling(self.multisampling);

        let event_loop = glutin::EventsLoop::new();
        let (renderer, window, mut factory) = Renderer::new(builder, context, &event_loop, &self.shader_path);
        let scene = factory.scene();
        Window {
            event_loop,
            window,
            input: Input::new(),
            renderer,
            factory,
            scene,
        }
    }
}

impl Window {
    /// Create new `Builder` with standard parameters.
    pub fn builder<T: Into<String>, P: AsRef<Path>>(
        title: T,
        shader_path: P,
    ) -> Builder {
        Builder {
            dimensions: (1024, 768),
            fullscreen: false,
            multisampling: 0,
            shader_path: shader_path.as_ref().to_owned(),
            title: title.into(),
            vsync: true,
        }
    }

    /// `update` method returns `false` if the window was closed.
    pub fn update(&mut self) -> bool {
        let mut running = true;
        let renderer = &mut self.renderer;
        let input = &mut self.input;
        input.reset();

        self.window.swap_buffers().unwrap();
        let window = &self.window;

        self.event_loop.poll_events(|event| {
            use glutin::WindowEvent::{Closed, KeyboardInput, MouseInput, MouseMoved, MouseWheel, Resized};
            match event {
                glutin::Event::WindowEvent { event, .. } => match event {
                    Resized(..) => renderer.resize(window),
                    Closed => running = false,
                    KeyboardInput {
                        input: glutin::KeyboardInput {
                            state,
                            virtual_keycode: Some(keycode),
                            ..
                        },
                        ..
                    } => input.keyboard_input(state, keycode),
                    MouseInput { state, button, .. } => input.mouse_input(state, button),
                    MouseMoved {
                        position: (x, y), ..
                    } => input.mouse_moved(
                        [x as f32, y as f32].into(),
                        renderer.map_to_ndc([x as f32, y as f32]),
                    ),
                    MouseWheel { delta, .. } => input.mouse_wheel_input(delta),
                    _ => {}
                },
                glutin::Event::DeviceEvent { event, .. } => match event {
                    glutin::DeviceEvent::Motion { axis, value } => {
                        let delta = if axis == 0 {
                            [value as f32, 0.0].into()
                        } else if axis == 1 {
                            [0.0, value as f32].into()
                        } else {
                            return;
                        };
                        input.mouse_moved_raw(delta);
                    }
                    _ => {}
                },
                _ => {}
            }
        });

        running
    }

    /// Render the current scene with specific [`Camera`](struct.Camera.html).
    pub fn render(
        &mut self,
        camera: &Camera,
    ) {
        self.renderer.render(&self.scene, camera);
    }

    /// Get current window size in pixels.
    pub fn size(&self) -> mint::Vector2<f32> {
        let size = self.window
            .get_inner_size_pixels()
            .expect("Can't get window size");
        [size.0 as f32, size.1 as f32].into()
    }

    /// Set cursor visibility
    pub fn show_cursor(
        &self,
        enable: bool,
    ) {
        let _ = if enable {
            self.window.set_cursor_state(glutin::CursorState::Normal)
        } else {
            self.window.set_cursor_state(glutin::CursorState::Hide)
        };
    }

    /// Returns underlaying `glutin::GlWindow`.
    #[cfg(feature = "opengl")]
    pub fn glutin_window(&self) -> &glutin::GlWindow {
        &self.window
    }
}
