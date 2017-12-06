extern crate cgmath;
extern crate mint;
extern crate three;

use cgmath::prelude::*;
use std::f32::consts::PI;
use three::Object;

fn make_tetrahedron_geometry() -> three::Geometry {
    let vertices = vec![
        mint::Point3 {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        },
        mint::Point3 {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        },
        mint::Point3 {
            x: (2.0 * PI / 3.0).sin(),
            y: 0.0,
            z: (2.0 * PI / 3.0).cos(),
        },
        mint::Point3 {
            x: (4.0 * PI / 3.0).sin(),
            y: 0.0,
            z: (4.0 * PI / 3.0).cos(),
        },
    ];
    let faces = vec![[0, 1, 2], [0, 2, 3], [0, 3, 1], [1, 3, 2]];
    three::Geometry {
        faces,
        base_shape: three::geometry::Shape {
            vertices,
            ..three::geometry::Shape::empty()
        },
        ..three::Geometry::empty()
    }
}

fn main() {
    let mut win = three::Window::new("Three-rs Mesh Update Example");
    let cam = win.factory.perspective_camera(60.0, 1.0 .. 10.0);
    let mut controls = three::controls::Orbit::builder(&cam)
        .position([0.0, 2.0, -5.0])
        .target([0.0, 0.0, 0.0])
        .build();

    let geometry = make_tetrahedron_geometry();
    let material = three::material::Wireframe { color: 0xFFFF00 };
    let mut mesh = win.factory.mesh_dynamic(geometry, material);
    let vertex_count = mesh.vertex_count();
    mesh.set_parent(&win.scene);

    let mut timer = win.input.time();
    let mut vi = 0;
    while win.update() && !win.input.hit(three::KEY_ESCAPE) {
        let elapsed_time = timer.get(&win.input);
        if elapsed_time > 1.0 {
            // Reset the timer.
            timer = win.input.time();
            // Update the vertex `vi`.
            let mut vmap = win.factory.map_vertices(&mut mesh);
            let dir = cgmath::Vector4::from(vmap[vi].pos).truncate();
            let pos = cgmath::Point3::from_vec(1.2 * dir);
            vmap[vi].pos = [pos.x, pos.y, pos.z, 1.0];
            // Increment vertex index.
            vi = (vi + 1) % vertex_count;
        }
        controls.update(&win.input);
        win.render(&cam);
    }
}
