extern crate cgmath;
extern crate env_logger;
extern crate mint;
extern crate rand;
extern crate three;

mod plane;
mod sky;

use cgmath::prelude::*;
use three::Object;

const COLOR_BACKGROUND: three::Color = 0xf0e0b6;
const COLOR_RED: three::Color = 0xf25346;
const COLOR_WHITE: three::Color = 0xd8d0d1;
const COLOR_BROWN: three::Color = 0x59332e;
//const COLOR_PINK: three::Color = 0xF5986E;
const COLOR_BROWN_DARK: three::Color = 0x23190f;
const COLOR_BLUE: three::Color = 0x68c3c0;

fn main() {
    env_logger::init().unwrap();
    let mut rng = rand::thread_rng();

    let mut win = three::Window::new("Three-rs Aviator demo");
    win.scene.background = three::Background::Color(COLOR_BACKGROUND);

    let mut cam = win.factory.perspective_camera(60.0, 1.0 .. 1000.0);
    cam.set_position([0.0, 100.0, 200.0]);
    cam.set_parent(&win.scene);

    //TODO: win.scene.fog = Some(three::Fog::new(...));
    //TODO: Phong materials

    let mut hemi_light = win.factory.hemisphere_light(0xaaaaaa, 0x000000, 0.9);
    hemi_light.set_parent(&win.scene);
    let mut dir_light = win.factory.directional_light(0xffffff, 0.9);
    dir_light.look_at([150.0, 350.0, 350.0], [0.0, 0.0, 0.0], None);
    let shadow_map = win.factory.shadow_map(2048, 2048);
    dir_light.set_shadow(shadow_map, 400.0, 1.0 .. 1000.0);
    dir_light.set_parent(&win.scene);
    let mut ambient_light = win.factory.ambient_light(0xdc8874, 0.5);
    ambient_light.set_parent(&win.scene);

    let mut sea = {
        let geo = three::Geometry::cylinder(600.0, 600.0, 800.0, 40);
        let material = three::material::Lambert {
            color: COLOR_BLUE,
            flat: true,
        };
        win.factory.mesh(geo, material)
    };
    let sea_base_q = cgmath::Quaternion::from_angle_x(-cgmath::Rad::turn_div_4());
    sea.set_transform([0.0, -600.0, 0.0], sea_base_q, 1.0);
    sea.set_parent(&win.scene);

    let mut sky = sky::Sky::new(&mut rng, &mut win.factory);
    sky.group.set_position([0.0, -600.0, 0.0]);
    sky.group.set_parent(&win.scene);

    let mut airplane = plane::AirPlane::new(&mut win.factory);
    airplane
        .group
        .set_transform([0.0, 100.0, 0.0], [0.0, 0.0, 0.0, 1.0], 0.25);
    airplane.group.set_parent(&win.scene);

    let timer = win.input.time();
    while win.update() && !win.input.hit(three::KEY_ESCAPE) {
        use cgmath::{Quaternion, Rad};
        // assume the original velocities are given for 60fps
        let time = 60.0 * timer.get(&win.input);

        airplane.update(time, win.input.mouse_pos_ndc());

        let sea_angle = Rad(0.005 * time);
        let sea_q = Quaternion::from_angle_z(sea_angle) * sea_base_q;
        sea.set_orientation(sea_q);
        let sky_angle = Rad(0.01 * time);
        let sky_q = Quaternion::from_angle_z(sky_angle);
        sky.group.set_orientation(sky_q);

        win.render(&cam);
    }
}
