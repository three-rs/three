extern crate three;

use std::env;

fn main() {
    let mut args = env::args();
    let path = args.nth(1).unwrap_or("test_data/car.obj".to_string());

    let mut win = three::Window::new("Three-rs obj loading example", "data/shaders");
    let cam = win.factory.perspective_camera(60.0, 1.0, 10.0);
    let mut controls = three::OrbitControls::new(&cam, [0.0, 2.0, -5.0], [0.0, 0.0, 0.0]);

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
