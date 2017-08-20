use glutin;
use glutin::GlContext;

use camera::{Camera, Projection};
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

pub struct WindowBuilder {
    dimensions: (u32, u32),
    fullscreen: bool,
    multisampling: u16,
    shader_path: String,
    title: String,
    vsync: bool,
}

impl WindowBuilder {
    pub fn dimensions(&mut self, width: u32, height: u32) -> &mut Self {
        self.dimensions = (width, height);
        self
    }

    pub fn fullscreen(&mut self, option: bool) -> &mut Self {
        self.fullscreen = option;
        self
    }

    pub fn multisampling(&mut self, option: u16) -> &mut Self {
        self.multisampling = option;
        self
    }

    pub fn vsync(&mut self, option: bool) -> &mut Self {
        self.vsync = option;
        self
    }

    pub fn build(&mut self) -> Window {
        use glutin::get_primary_monitor;

        let builder = if self.fullscreen {
            glutin::WindowBuilder::new().with_fullscreen(get_primary_monitor())
        } else {
            glutin::WindowBuilder::new()
        };

        let builder = builder.clone()
            .with_dimensions(self.dimensions.0, self.dimensions.1)
            .with_title(self.title.clone());

        let context = glutin::ContextBuilder::new()
            .with_vsync(self.vsync)
            .with_multisampling(self.multisampling);

        let event_loop = glutin::EventsLoop::new();
        let (renderer, window, mut factory) = Renderer::new(builder,
                                                            context,
                                                            &event_loop,
                                                            &self.shader_path);
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
    /// Create new `Window` with specific title.
    pub fn new(title: &str, shader_path: &str) -> WindowBuilder {
        WindowBuilder {
            dimensions: (1024, 768),
            fullscreen: false,
            multisampling: 0,
            shader_path: shader_path.to_owned(),
            title: title.to_owned(),
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
            use glutin::WindowEvent::{Resized, Closed, KeyboardInput,
                MouseInput, MouseMoved, MouseWheel};
            match event {
                glutin::Event::WindowEvent { event, ..} => match event {
                    Resized(..) => renderer.resize(window),
                    Closed => running = false,
                    KeyboardInput {
                        input: glutin::KeyboardInput {
                            state,
                            virtual_keycode: Some(keycode), ..
                        }, ..
                    } => input.keyboard_input(state, keycode),
                    MouseInput {
                        state,
                        button, .. } => input.mouse_input(state, button),
                    MouseMoved {
                        position: (x, y), ..
                    } => input.mouse_moved(renderer.map_to_ndc(x as f32, y as f32)),
                    MouseWheel { delta, .. } => input.mouse_wheel_input(delta),
                    _ => {}
                },
                _ => {}
            }
        });

        running
    }

    /// Render the current scene with specific [`Camera`](struct.Camera.html).
    pub fn render<P: Projection>(&mut self, camera: &Camera<P>) {
        self.renderer.render(&self.scene, camera);
    }
}
