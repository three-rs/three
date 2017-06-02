extern crate three;

fn main() {
    let mut win = three::Window::new("Three-rs lights example");
    let mut cam = win.factory.perspective_camera(75.0, 0.0, 1.0, 50.0);
    cam.transform_mut().look_at(three::Position::new(0.0, 0.0, 10.0),
                                three::Position::new(0.0, 0.0, 2.0));

    let mut dir_light = win.factory.directional_light(0xffffff, 0.9);
    dir_light.transform_mut().look_at(three::Position::new(15.0, 35.0, 35.0),
                                      three::Position::new(0.0, 0.0, 0.0));
    let shadow_map = win.factory.shadow_map(1024, 1024);
    let _debug_shadow = win.renderer.debug_shadow_quad(&shadow_map, 1, [10, 10], [256, 256]);
    dir_light.set_shadow(shadow_map, 80.0, 80.0, 1.0, 200.0);
    win.scene.add(&dir_light);

    let mut sphere = {
        let geometry = three::Geometry::new_sphere(2.0, 5, 5);
        let material = three::Material::MeshLambert { color: 0xffffff };
        win.factory.mesh(geometry, material)
    };
    sphere.transform_mut().disp.z = 2.5;
    win.scene.add(&sphere);

    let plane = {
        let geometry = three::Geometry::new_plane(100.0, 100.0);
        let material = three::Material::MeshLambert { color: 0xffffff };
        win.factory.mesh(geometry, material)
    };
    win.scene.add(&plane);

    while let Some(events) = win.update() {
        if events.keys.contains(&three::Key::Left) {
            //TODO
        }
        if events.keys.contains(&three::Key::Right) {
            //TODO
        }

        win.render(&cam);
    }
}
