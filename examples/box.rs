extern crate cgmath;
extern crate glutin;
extern crate three;

use cgmath::prelude::*;

fn main() {
    let builder = glutin::WindowBuilder::new()
                                        .with_title("Three-rs line drawing example");
    let event_loop = glutin::EventsLoop::new();
    let (mut renderer, mut factory) = three::Renderer::new(builder, &event_loop);

    let mut camera = three::PerspectiveCamera::new(75.0, renderer.get_aspect(), 1.0, 500.0);
    camera.position = three::Position::new(0.0, 0.0, 5.0);

    let geometry = three::Geometry::new_box(1.0, 1.0, 1.0);
    let material = three::Material::MeshBasic { color: 0x00ff00 };
    let mut mesh = factory.mesh(geometry, material);

    let mut scene = factory.scene();
    mesh.attach(&mut scene, None);

    let mut angle = 0f32;
    let mut running = true;
    while running {
        event_loop.poll_events(|glutin::Event::WindowEvent {event, ..}| {
            use glutin::WindowEvent as Event;
            use glutin::VirtualKeyCode as Key;
            match event {
                Event::Resized(..) => {
                    renderer.resize();
                    camera.projection.aspect = renderer.get_aspect();
                }
                Event::KeyboardInput(_, _, Some(Key::Escape), _) |
                Event::Closed => {
                    running = false
                }
                _ => ()
            }
        });

        angle += 0.005f32;
        mesh.transform_mut().rot = three::Orientation::from_axis_angle(
            cgmath::Vector3::unit_y(), cgmath::Rad(angle));

        scene.update();
        renderer.render(&scene, &camera);
    }
}
