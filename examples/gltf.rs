
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
    let (group, _meshes) = win.factory.load_gltf(&path);
    win.scene.add(&group);

    while win.update() && !three::KEY_ESCAPE.is_hit(&win.input) {
        if three::Button::Key(three::Key::Q).is_hit(&win.input) {
            st.yaw -= st.look_speed;
        }
        if three::Button::Key(three::Key::E).is_hit(&win.input) {
            st.yaw += st.look_speed;
        }
        if three::Button::Key(three::Key::R).is_hit(&win.input) {
            st.pitch -= st.look_speed;
        }
        if three::Button::Key(three::Key::F).is_hit(&win.input) {
            st.pitch += st.look_speed;
        }
        if three::Button::Key(three::Key::X).is_hit(&win.input) {
            st.position.y += st.move_speed;
        }
        if three::Button::Key(three::Key::Z).is_hit(&win.input) {
            st.position.y -= st.move_speed;
        }
        if three::Button::Key(three::Key::W).is_hit(&win.input) {
            st.position.x += st.move_speed * st.yaw.sin();
            st.position.z -= st.move_speed * st.yaw.cos();
        }
        if three::Button::Key(three::Key::S).is_hit(&win.input) {
            st.position.x -= st.move_speed * st.yaw.sin();
            st.position.z += st.move_speed * st.yaw.cos();
        }
        if three::Button::Key(three::Key::D).is_hit(&win.input) {
            st.position.x += st.move_speed * st.yaw.cos();
            st.position.z += st.move_speed * st.yaw.sin();
        }
        if three::Button::Key(three::Key::A).is_hit(&win.input) {
            st.position.x -= st.move_speed * st.yaw.cos();
            st.position.z -= st.move_speed * st.yaw.sin();
        }
        if three::Button::Key(three::Key::P).is_hit(&win.input) {
            println!("pos: {:?}, yaw: {}", st.position, st.yaw);
        }
        let yrot = cgmath::Quaternion::<f32>::from_angle_y(cgmath::Rad(-st.yaw));
        let xrot = cgmath::Quaternion::<f32>::from_angle_x(cgmath::Rad(-st.pitch));
        cam.set_transform(cgmath::Point3::from_vec(st.position), yrot * xrot, 1.0);
        win.render(&cam); 
    }
}
