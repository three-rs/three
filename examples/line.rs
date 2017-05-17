extern crate three;
extern crate cgmath;
extern crate glutin;

use cgmath::{Point3, Vector3};

fn main() {
    let builder = glutin::WindowBuilder::new()
                                        .with_title("Three-rs line drawing example");
    let event_loop = glutin::EventsLoop::new();
    let (window, mut renderer, mut factory) = three::Renderer::new(builder, &event_loop);
    let (width, height) = window.get_inner_size_pixels().unwrap();

    let mut camera = three::PerspectiveCamera::new(45.0, width as f32 / height as f32, 1.0, 500.0);
    camera.position = Point3::new(0.0, 0.0, 100.0);
    camera.look_at(Point3::new(0.0, 0.0, 0.0));

    let geometry = three::Geometry {
        vertices: vec![
            Vector3::new(-10.0, 0.0, 0.0),
            Vector3::new(0.0, 10.0, 0.0),
            Vector3::new(10.0, 0.0, 0.0),
        ],
    };
    let material = three::Material::LineBasic { color: 0x0000ff };
    let line = factory.line(&geometry, material);

    let mut scene = three::Scene::new();
    scene.add(line);

    let mut running = true;
    while running {
        event_loop.poll_events(|glutin::Event::WindowEvent {event, ..}| {
            use glutin::WindowEvent as Event;
            use glutin::VirtualKeyCode as Key;
            match event {
                Event::KeyboardInput(_, _, Some(Key::Escape), _) => running = false,
                _ => ()
            }
        });

        renderer.render(&scene, &camera);
        window.swap_buffers().unwrap();
    }
}
