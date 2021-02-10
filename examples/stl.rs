extern crate three;
extern crate mint;
extern crate froggy;

use std::env;
use three::{Geometry, Object};
use mint::Point3;
use froggy::WeakPointer;

fn main() {
    let mut args = env::args();
    let obj_path = concat!(env!("CARGO_MANIFEST_DIR"), "/test_data/sample.stl");
    let path = args.nth(1).unwrap_or(obj_path.into());
    let mut vertices = vec!();

    match stlv::parser::load_file(path.as_str()) {
        Ok(model) => {
            for triangle in (*model).iter() {
                let stl_vertices = triangle.vertices();
                vertices.push(Point3 {
                    x: stl_vertices[0].get_x(),
                    y: stl_vertices[0].get_y(),
                    z: stl_vertices[0].get_z(),
                });
                vertices.push(Point3 {
                    x: stl_vertices[1].get_x(),
                    y: stl_vertices[1].get_y(),
                    z: stl_vertices[1].get_z(),
                });
                vertices.push(Point3 {
                    x: stl_vertices[2].get_x(),
                    y: stl_vertices[2].get_y(),
                    z: stl_vertices[2].get_z(),
                });
            }
        }
        _ => panic!("Failed to parse the STL file {}", path),
    }

    let geometry = Geometry::with_vertices(vertices);

    // Upload the triangle data to the GPU.
    let mut window = three::Window::new("Loading STL...");

    // Create multiple meshes with the same GPU data and material.
    let material = three::material::Wireframe{color: 0xff0000};

    let mesh = window.factory.mesh(geometry, material);
    window.scene.add(&mesh);

    let cam = window.factory.perspective_camera(60.0, 1.0 .. 1000.0);
    let mut controls = three::controls::Orbit::builder(&cam)
        .position([0.0, 2.0, -5.0])
        .target([0.0, 0.0, 0.0])
        .build();

    let dir_light = window.factory.directional_light(0xffffff, 0.9);
    dir_light.look_at([15.0, 35.0, 35.0], [0.0, 0.0, 2.0], None);
    window.scene.add(&dir_light);

    while window.update() && !window.input.hit(three::KEY_ESCAPE) {
        controls.update(&window.input);
        window.render(&cam);
    }
}
