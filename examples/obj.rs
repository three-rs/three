extern crate three;

fn main() {
    let mut win = three::Window::new("Three-rs obj loading example", "data/shaders");
    let mut cam = win.factory.perspective_camera(60.0, 0.0, 1.0, 10.0);
    cam.transform_mut().disp = three::Vector::new(0.0, 2.0, 5.0);

    let mut root = win.factory.group();
    root.transform_mut().scale = 20.0;
    win.scene.add(&root);
    let (group_map, _meshes) = win.factory.load_obj("test_data/bunny.obj");
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
