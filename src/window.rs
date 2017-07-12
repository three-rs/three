use glutin;

use {Camera, Scene};
use camera::Projection;
use input::Input;
use render::Renderer;
use factory::Factory;


/// `Window` is the core entity of every `three-rs` application.
///
/// It provides [user input](struct.Window.html#method.update),
/// [`Factory`](struct.Factory.html) and [`Renderer`](struct.Renderer.html).
pub struct Window {
    event_loop: glutin::EventsLoop,
    window: glutin::Window,
    /// See [`Input`](struct.Input.html).
    pub input: Input,
    /// See [`Renderer`](struct.Renderer.html).
    pub renderer: Renderer,
    /// See [`Factory`](struct.Factory.html).
    pub factory: Factory,
    /// See [`Scene`](struct.Scene.html).
    pub scene: Scene,
}

impl Window {
    /// Create new `Window` with specific title.
    pub fn new(title: &str, shader_path: &str) -> Self {
        let builder = glutin::WindowBuilder::new()
                             .with_title(title)
                             .with_vsync();
        let event_loop = glutin::EventsLoop::new();
        let (renderer, window, mut factory) = Renderer::new(builder, &event_loop, shader_path);
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

    /// `update` method returns `false` if the window was closed.
    pub fn update(&mut self) -> bool {
        let mut running = true;
        let renderer = &mut self.renderer;
        let input = &mut self.input;
        input.reset();

        self.window.swap_buffers().unwrap();
        let window = &self.window;

        self.event_loop.poll_events(|glutin::Event::WindowEvent {event, ..}| {
            use glutin::WindowEvent::*;
            match event {
                Resized(..) => renderer.resize(window),
                Closed => running = false,
                KeyboardInput(state, _, Some(key), _) => input.keyboard_input(state, key),
                MouseInput(state, button) => input.mouse_input(state, button),
                MouseMoved(x, y) => input.mouse_moved(renderer.map_to_ndc(x, y)),
                MouseWheel(delta, _) => input.mouse_wheel(delta),
                _ => ()
            }
        });

        running
    }

    /// Render the current scene with specific [`Camera`](struct.Camera.html).
    pub fn render<P: Projection>(&mut self, camera: &Camera<P>) {
        self.renderer.render(&self.scene, camera);
    }
}
