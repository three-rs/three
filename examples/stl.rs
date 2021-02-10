extern crate three;
extern crate mint;
extern crate froggy;

use std::env;
use three::Geometry;
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
                vertices.push(Point3{x: stl_vertices[0].get_x(), y: stl_vertices[0].get_y(),
                    z: stl_vertices[0].get_z()});
                vertices.push(Point3{x: stl_vertices[1].get_x(), y: stl_vertices[1].get_y(),
                    z: stl_vertices[1].get_z()});
                vertices.push(Point3{x: stl_vertices[2].get_x(), y: stl_vertices[2].get_y(),
                    z: stl_vertices[2].get_z()});
            }
        }
        _ => panic!("Failed to parse the STL file {}", path),
    }

    let geometry = Geometry::with_vertices(vertices);

     // Upload the triangle data to the GPU.
    let mut window = three::Window::new("Three-rs obj loading example");
    let upload_geometry = window.factory.upload_geometry(geometry);

     // Create multiple meshes with the same GPU data and material.
     let material = three::material::Basic {
         color: 0xFFFF00,
         map: None,
     };

    window.factory.create_instanced_mesh(&upload_geometry, material.clone());

    let cam = window.factory.perspective_camera(60.0, 1.0 .. 1000.0);
    let mut controls = three::controls::Orbit::builder(&cam)
        .position([0.0, 2.0, -5.0])
        .target([0.0, 0.0, 0.0])
        .build();

    while window.update() && !window.input.hit(three::KEY_ESCAPE) {
        controls.update(&window.input);
        window.render(&cam);
    }
}
