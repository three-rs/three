extern crate cgmath;
extern crate mint;
extern crate three;

use cgmath::prelude::*;

fn main() {
    let mut win = three::Window::new("Three-rs shapes example");
    let mut cam = win.factory.perspective_camera(75.0, 1.0 .. 50.0);
    cam.set_position([0.0, 0.0, 10.0]);

    let mut mbox = {
        let geometry = three::Geometry::cuboid(3.0, 2.0, 1.0);
        let material = three::material::Wireframe { color: 0x00FF00 };
        win.factory.mesh(geometry, material)
    };
    mbox.set_position([-3.0, -3.0, 0.0]);
    mbox.set_parent(&win.scene);

    let mut mcyl = {
        let geometry = three::Geometry::cylinder(1.0, 2.0, 2.0, 5);
        let material = three::material::Wireframe { color: 0xFF0000 };
        win.factory.mesh(geometry, material)
    };
    mcyl.set_position([3.0, -3.0, 0.0]);
    mcyl.set_parent(&win.scene);

    let mut msphere = {
        let geometry = three::Geometry::uv_sphere(2.0, 5, 5);
        let material = three::material::Wireframe { color: 0xFF0000 };
        win.factory.mesh(geometry, material)
    };
    msphere.set_position([-3.0, 3.0, 0.0]);
    msphere.set_parent(&win.scene);

    let mut mline = {
        let geometry = three::Geometry::with_vertices(vec![
            [-2.0, -1.0, 0.0].into(),
            [0.0, 1.0, 0.0].into(),
            [2.0, -1.0, 0.0].into(),
        ]);
        let material = three::material::Line { color: 0x0000FF };
        win.factory.mesh(geometry, material)
    };
    mline.set_position([3.0, 3.0, 0.0]);
    mline.set_parent(&win.scene);

    let mut angle = cgmath::Rad::zero();
    while win.update() && !win.input.hit(three::KEY_ESCAPE) {
        let dt = win.input.delta_time();
        let radians_per_second = 1.5;
        if win.input.hit(three::Key::Left) {
            angle -= cgmath::Rad(radians_per_second * dt);
        }
        if win.input.hit(three::Key::Right) {
            angle += cgmath::Rad(radians_per_second * dt);
        }
        let q = cgmath::Quaternion::from_angle_y(angle);
        mbox.set_orientation(q);
        mcyl.set_orientation(q);
        msphere.set_orientation(q);
        mline.set_orientation(q);
        win.render(&cam);
    }
}
