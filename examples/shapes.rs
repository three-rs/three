extern crate cgmath;
extern crate three;

use cgmath::prelude::*;

fn main() {
    let mut win = three::Window::new("Three-rs shapes example", "data/shaders");
    let mut cam = win.factory.perspective_camera(75.0, 0.0, 1.0, 50.0);
    cam.transform_mut().disp = three::Vector::new(0.0, 0.0, 10.0);

    let mut mbox = {
        let geometry = three::Geometry::new_box(3.0, 2.0, 1.0);
        let material = three::Material::MeshBasic { color: 0x00ff00, map: None, wireframe: true };
        win.factory.mesh(geometry, material)
    };
    mbox.transform_mut().disp = cgmath::vec3(-3.0, -3.0, 0.0);
    win.scene.add(&mbox);

    let mut mcyl = {
        let geometry = three::Geometry::new_cylinder(1.0, 2.0, 2.0, 5);
        let material = three::Material::MeshBasic { color: 0xff0000, map: None, wireframe: true };
        win.factory.mesh(geometry, material)
    };
    mcyl.transform_mut().disp = cgmath::vec3(3.0, -3.0, 0.0);
    win.scene.add(&mcyl);

    let mut msphere = {
        let geometry = three::Geometry::new_sphere(2.0, 5, 5);
        let material = three::Material::MeshBasic { color: 0xff0000, map: None, wireframe: true };
        win.factory.mesh(geometry, material)
    };
    msphere.transform_mut().disp = cgmath::vec3(-3.0, 3.0, 0.0);
    win.scene.add(&msphere);

    let mut mline = {
        let geometry = three::Geometry::from_vertices(vec![
            three::Position::new(-2.0, -1.0, 0.0),
            three::Position::new(0.0, 1.0, 0.0),
            three::Position::new(2.0, -1.0, 0.0),
        ]);
        let material = three::Material::LineBasic { color: 0x0000ff };
        win.factory.mesh(geometry, material)
    };
    mline.transform_mut().disp = cgmath::vec3(3.0, 3.0, 0.0);
    win.scene.add(&mline);

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
            let rot = three::Orientation::from_axis_angle(cgmath::Vector3::unit_y(), angle);
            mbox.transform_mut().rot = rot;
            mcyl.transform_mut().rot = rot;
            mline.transform_mut().rot = rot;
            msphere.transform_mut().rot = rot;
        }

        win.render(&cam);
    }
}
