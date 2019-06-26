extern crate cgmath;
extern crate mint;
extern crate three;

use cgmath::prelude::*;
use three::Object;

fn main() {
    let mut win = three::Window::new("Three-rs shapes example");
    let cam = win.factory.perspective_camera(75.0, 1.0 .. 50.0);
    cam.set_position([0.0, 0.0, 10.0]);

    let mbox = {
        let geometry = three::Geometry::cuboid(3.0, 2.0, 1.0);
        let material = three::material::Wireframe { color: 0x00FF00 };
        win.factory.mesh(geometry, material)
    };
    mbox.set_position([-3.0, -3.0, 0.0]);
    win.scene.add(&mbox);

    let mcyl = {
        let geometry = three::Geometry::cylinder(1.0, 2.0, 2.0, 5);
        let material = three::material::Wireframe { color: 0xFF0000 };
        win.factory.mesh(geometry, material)
    };
    mcyl.set_position([3.0, -3.0, 0.0]);
    win.scene.add(&mcyl);

    let msphere = {
        let geometry = three::Geometry::uv_sphere(2.0, 5, 5);
        let material = three::material::Wireframe { color: 0xFF0000 };
        win.factory.mesh(geometry, material)
    };
    msphere.set_position([-3.0, 3.0, 0.0]);
    win.scene.add(&msphere);

    // test removal from scene
    win.scene.remove(&mcyl);
    win.scene.remove(&mbox);
    win.scene.add(&mcyl);
    win.scene.add(&mbox);

    let mline = {
        let geometry = three::Geometry::with_vertices(vec![[-2.0, -1.0, 0.0].into(), [0.0, 1.0, 0.0].into(), [2.0, -1.0, 0.0].into()]);
        let material = three::material::Line { color: 0x0000FF };
        win.factory.mesh(geometry, material)
    };
    mline.set_position([3.0, 3.0, 0.0]);
    win.scene.add(&mline);

    let mut angle = cgmath::Rad::zero();
    while win.update() && !win.input.hit(three::KEY_ESCAPE) {
        if let Some(diff) = win.input.timed(three::AXIS_LEFT_RIGHT) {
            angle += cgmath::Rad(1.5 * diff);
            let q = cgmath::Quaternion::from_angle_y(angle);
            mbox.set_orientation(q);
            mcyl.set_orientation(q);
            msphere.set_orientation(q);
            mline.set_orientation(q);
        }
        win.render(&cam);
    }
}
