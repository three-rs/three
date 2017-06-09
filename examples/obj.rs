extern crate three;

use std::env;

fn main() {
    let mut args = env::args();
    let path = args.nth(1).unwrap_or("test_data/car.obj".to_string());

    let mut win = three::Window::new("Three-rs obj loading example", "data/shaders");
    let mut cam = win.factory.perspective_camera(60.0, 0.0, 1.0, 10.0);
    cam.transform_mut().look_at([0.0, 2.0, 5.0].into(),
                                [0.0, 0.0, 0.0].into(),
                                Some([0.0, 1.0, 0.0].into()));

    let mut dir_light = win.factory.directional_light(0xffffff, 0.9);
    dir_light.transform_mut().look_at([15.0, 35.0, 35.0].into(),
                                      [0.0, 0.0, 2.0].into(),
                                      None);
    win.scene.add(&dir_light);

    let mut root = win.factory.group();
    win.scene.add(&root);
    let (group_map, _meshes) = win.factory.load_obj(&path);
    for g in group_map.values() {
        root.add(g);
    }

    while let Some(events) = win.update() {
        let mut angle = 0.0;
        if events.keys.contains(&three::Key::Left) {
            angle = -events.time_delta;
        }
        if events.keys.contains(&three::Key::Right) {
            angle = events.time_delta;
        }
        if angle != 0.0 {
            root.transform_mut().rotate(0.0, 1.5 * angle, 0.0);
        }

        win.render(&cam);
    }
}
