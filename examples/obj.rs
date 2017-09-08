extern crate three;

use std::env;

fn main() {
    let mut args = env::args();
    let obj_path: String = format!("{}/test_data/car.obj", env!("CARGO_MANIFEST_DIR"));
    let path = args.nth(1).unwrap_or(obj_path);

    let shaders_path: String = format!("{}/data/shaders", env!("CARGO_MANIFEST_DIR"));
    let shaders_path_str: &str = shaders_path.as_str();
    let mut win = three::Window::builder("Three-rs obj loading example", shaders_path_str).build();
    let cam = win.factory.perspective_camera(60.0, 1.0 .. 10.0);
    let mut controls = three::controls::Orbit::builder(&cam)
        .position([0.0, 2.0, -5.0])
        .target([0.0, 0.0, 0.0])
        .build();

    let mut dir_light = win.factory.directional_light(0xffffff, 0.9);
    dir_light.look_at([15.0, 35.0, 35.0], [0.0, 0.0, 2.0], None);
    win.scene.add(&dir_light);

    let mut root = win.factory.group();
    win.scene.add(&root);
    let (group_map, _meshes) = win.factory.load_obj(&path);
    for g in group_map.values() {
        root.add(g);
    }

    while win.update() && !three::KEY_ESCAPE.is_hit(&win.input) {
        controls.update(&win.input);
        win.render(&cam);
    }
}
