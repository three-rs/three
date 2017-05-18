extern crate three;
extern crate cgmath;
extern crate glutin;

use cgmath::{Point3, Vector3};

fn main() {
    let builder = glutin::WindowBuilder::new()
                                        .with_title("Three-rs line drawing example");
    let event_loop = glutin::EventsLoop::new();
    let (window, mut renderer, mut factory) = three::Renderer::new(builder, &event_loop);

    let geometry = three::Geometry::from_vertices(vec![
            Vector3::new(-10.0, 0.0, 0.0),
            Vector3::new(0.0, 10.0, 0.0),
            Vector3::new(10.0, 0.0, 0.0),
    ]);
    let material = three::Material::LineBasic { color: 0x0000ff };
    let line = factory.line(geometry.clone(), material.clone());

    let mut other_line = factory.line(geometry, material);
    other_line.add(&line);

    let mut scene = three::Scene::new();
    scene.add(&other_line);
}
