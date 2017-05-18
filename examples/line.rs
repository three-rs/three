extern crate three;
extern crate glutin;

fn main() {
    let builder = glutin::WindowBuilder::new()
                                        .with_title("Three-rs line drawing example");
    let event_loop = glutin::EventsLoop::new();
    let (mut renderer, mut factory) = three::Renderer::new(builder, &event_loop);

    let mut camera = three::PerspectiveCamera::new(45.0, renderer.get_aspect(), 1.0, 500.0);
    camera.position = three::Position::new(0.0, 0.0, 100.0);
    camera.look_at(three::Position::new(0.0, 0.0, 0.0));

    let geometry = three::Geometry::from_vertices(vec![
        three::Position::new(-10.0, 0.0, 0.0),
        three::Position::new(0.0, 10.0, 0.0),
        three::Position::new(10.0, 0.0, 0.0),
    ]);
    let material = three::Material::LineBasic { color: 0x0000ff };
    let line = factory.line(geometry, material);

    let mut scene = three::Scene::new();
    scene.add(&line);

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

        renderer.render(&scene, &camera);
    }
}
