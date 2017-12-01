extern crate three;

use three::Object;

fn main() {
    let mut win = three::Window::new("Three-rs glTF example");
    let mut light = win.factory.directional_light(0xFFFFFF, 7.0);
    light.look_at([1.0, 1.0, 1.0], [0.0, 0.0, 0.0], None);
    light.set_parent(&win.scene);
    win.scene.background = three::Background::Color(0xC6F0FF);

    let default = concat!(env!("CARGO_MANIFEST_DIR"), "/test_data/Lantern.gltf");
    let path = std::env::args().nth(1).unwrap_or(default.into());
    let mut gltf = win.factory.load_gltf(&path);
    gltf.group.set_parent(&win.scene);

    let mut cam = if gltf.cameras.len() > 0 {
        gltf.cameras.swap_remove(0)
    } else {
        let mut default = win.factory.perspective_camera(60.0, 0.001 .. 100.0);
        default.set_parent(&win.scene);
        default
    };

    // To enable skybox remove this if expression.
    if false {
        let skybox_path = three::CubeMapPath {
            front: "test_data/skybox/posz.jpg",
            back: "test_data/skybox/negz.jpg",
            up: "test_data/skybox/posy.jpg",
            down: "test_data/skybox/negy.jpg",
            left: "test_data/skybox/negx.jpg",
            right: "test_data/skybox/posx.jpg",
        };
        let skybox = win.factory.load_cubemap(&skybox_path);
        win.scene.background = three::Background::Skybox(skybox);
    }

    let init = cam.sync(&win.scene).world_transform;
    let mut controls = three::controls::FirstPerson::builder(&cam)
        .position(init.position)
        .move_speed(4.0)
        .build();
    while win.update() && !win.input.hit(three::KEY_ESCAPE) {
        controls.update(&win.input);
        win.render(&cam);
    }
}
