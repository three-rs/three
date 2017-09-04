
extern crate cgmath;
extern crate three;
extern crate vec_map;

use cgmath::prelude::*;

struct State {
    yaw: f32,
    pitch: f32,
    look_speed: f32,
    move_speed: f32,
    position: cgmath::Vector3<f32>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.0,
            look_speed: 0.03,
            move_speed: 0.05,
            position: [0.0, 0.0, 0.0].into(),
        }
    }
}

fn main() {
    let shaders = concat!(env!("CARGO_MANIFEST_DIR"), "/data/shaders");
    let mut win = three::Window::new("Three-rs glTF example", &shaders).build();
    let mut cam = win.factory.perspective_camera(60.0, 0.001, 100.0);
    let mut st = State::default();
    let mut light = win.factory.directional_light(0xFFFFFF, 7.0);
    light.look_at([1.0, 1.0, 1.0], [0.0, 0.0, 0.0], None);
    win.scene.add(&light);
    win.scene.background = three::Background::Color(0xC6F0FF);

    let default = concat!(env!("CARGO_MANIFEST_DIR"), "/test_data/Lantern.gltf");
    let path = std::env::args().nth(1).unwrap_or(default.into());
    let (group, mut cameras, _meshes) = win.factory.load_gltf(&path);
    win.scene.add(&group);

    let mut cam = if cameras.len() > 0 {
        cameras.swap_remove(0)
    } else {
        let default = win.factory.perspective_camera(60.0, 0.001 .. 100.0);
        win.scene.add(&default);
        default
    };

    let init = cam.sync(&win.scene).world_transform;
    let mut controls = three::controls::FirstPerson::builder(&cam)
        .position(init.position)
        .build();
    while win.update() && !three::KEY_ESCAPE.is_hit(&win.input) {
        controls.update(&win.input);
        win.render(&cam); 
    }
}
