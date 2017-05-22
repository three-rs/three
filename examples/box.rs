extern crate cgmath;
extern crate three;

use cgmath::prelude::*;

fn main() {
    let mut cam = three::PerspectiveCamera::new(75.0, 0.0, 1.0, 50.0);
    cam.position = three::Position::new(0.0, 0.0, 5.0);
    let mut win = three::Window::new("Three-rs box mesh drawing example", cam);

    let geometry = three::Geometry::new_box(1.0, 1.0, 1.0);
    let material = three::Material::MeshBasic { color: 0x00ff00 };
    let mut mesh = win.factory.mesh(geometry, material);
    mesh.attach(&mut win.scene, None);

    let mut angle = cgmath::Rad::zero();
    let speed = 1.5;
    while let Some(events) = win.update() {
        let old_angle = angle;
        if events.keys.contains(&three::Key::Left) {
            angle -= cgmath::Rad(speed * events.time_delta);
        }
        if events.keys.contains(&three::Key::Right) {
            angle += cgmath::Rad(speed * events.time_delta);
        }
        if angle != old_angle {
            mesh.transform_mut().rot = three::Orientation::from_axis_angle(
                cgmath::Vector3::unit_y(), angle);
        }

        win.render();
    }
}
