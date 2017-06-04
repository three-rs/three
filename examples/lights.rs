extern crate three;

fn main() {
    let mut win = three::Window::new("Three-rs lights example", "data/shaders");
    let mut cam = win.factory.perspective_camera(45.0, 0.0, 1.0, 50.0);
    cam.transform_mut().look_at(three::Position::new(-4.0, 15.0, 10.0),
                                three::Position::new(0.0, 0.0, 2.0));

    let mut hemisphere_light = win.factory.hemisphere_light(0xffffff, 0x8080ff, 0.5);
    let mut ambient_light = win.factory.ambient_light(0xffffffff, 0.5);
    let mut point_light = win.factory.point_light(0xffffff, 0.9);
    point_light.transform_mut().disp = three::Vector::new(15.0, 35.0, 35.0);

    let mut dir_light = win.factory.directional_light(0xffffff, 0.9);
    dir_light.transform_mut().look_at(three::Position::new(15.0, 35.0, 35.0),
                                      three::Position::new(0.0, 0.0, 2.0));
    let shadow_map = win.factory.shadow_map(1024, 1024);
    let _debug_shadow = win.renderer.debug_shadow_quad(&shadow_map, 1, [10, 10], [256, 256]);
    dir_light.set_shadow(shadow_map, 80.0, 80.0, 1.0, 200.0);

    let mut lights: [&mut three::LightObject; 4] = [&mut hemisphere_light,
        &mut ambient_light, &mut point_light, &mut dir_light];
    for l in lights.iter_mut() {
        win.scene.add(l);
        l.set_visible(false);
    }

    let mut sphere = {
        let geometry = three::Geometry::new_sphere(3.0, 20, 20);
        let material = three::Material::MeshPhong { color: 0xffA0A0, glossiness: 80.0 };
        win.factory.mesh(geometry, material)
    };
    sphere.transform_mut().disp.z = 2.5;
    win.scene.add(&sphere);

    let mut plane = {
        let geometry = three::Geometry::new_plane(100.0, 100.0);
        let material = three::Material::MeshLambert { color: 0xA0ffA0 };
        win.factory.mesh(geometry, material)
    };
    plane.transform_mut().disp.y -= 30.0;
    win.scene.add(&plane);

    let mut light_id = 0i8;
    lights[0].set_visible(true);
    while let Some(events) = win.update() {
        let old_id = light_id;
        if events.hit.contains(&three::Key::Left) {
            light_id -= 1;
        }
        if events.hit.contains(&three::Key::Right) {
            light_id += 1;
        }
        if old_id != light_id {
            lights[old_id as usize].set_visible(false);
            light_id = (light_id + lights.len() as i8) % lights.len() as i8;
            lights[light_id as usize].set_visible(true);
        }

        win.render(&cam);
    }
}
