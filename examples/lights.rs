extern crate three;

fn main() {
    let shaders_path: String = format!("{}/data/shaders", env!("CARGO_MANIFEST_DIR"));
    let shaders_path_str: &str = shaders_path.as_str();
    let mut win = three::Window::builder("Three-rs lights example", shaders_path_str).build();
    let mut cam = win.factory.perspective_camera(45.0, 1.0 .. 50.0);
    cam.look_at([-4.0, 15.0, 10.0], [0.0, 0.0, 2.0], None);

    let mut hemisphere_light = win.factory.hemisphere_light(0xffffff, 0x8080ff, 0.5);
    let mut ambient_light = win.factory.ambient_light(0xffffffff, 0.5);
    let mut point_light = win.factory.point_light(0xffffff, 0.9);
    point_light.set_position([15.0, 35.0, 35.0]);

    let mut dir_light = win.factory.directional_light(0xffffff, 0.9);
    dir_light.look_at([15.0, 35.0, 35.0], [0.0, 0.0, 2.0], None);
    let shadow_map = win.factory.shadow_map(1024, 1024);
    let _debug_shadow = win.renderer.debug_shadow_quad(
        &shadow_map,
        1,
        [10, 10],
        [256, 256],
    );
    dir_light.set_shadow(shadow_map, 40.0, 1.0 .. 200.0);

    let mut lights: [&mut three::Object; 4] = [
        &mut hemisphere_light,
        &mut ambient_light,
        &mut point_light,
        &mut dir_light,
    ];
    for l in lights.iter_mut() {
        win.scene.add(l);
        l.set_visible(false);
    }

    let mut sphere = {
        let geometry = three::Geometry::uv_sphere(3.0, 20, 20);
        let material = three::Material::MeshPhong {
            color: 0xffA0A0,
            glossiness: 80.0,
        };
        win.factory.mesh(geometry, material)
    };
    sphere.set_position([0.0, 0.0, 2.5]);
    win.scene.add(&sphere);

    let mut plane = {
        let geometry = three::Geometry::plane(100.0, 100.0);
        let material = three::Material::MeshLambert {
            color: 0xA0ffA0,
            flat: false,
        };
        win.factory.mesh(geometry, material)
    };
    plane.set_position([0.0, -30.0, 0.0]);
    win.scene.add(&plane);

    let mut light_id = 0i8;
    lights[0].set_visible(true);
    while win.update() && !three::KEY_ESCAPE.is_hit(&win.input) {
        if let Some(diff) = three::AXIS_LEFT_RIGHT.delta_hits(&win.input) {
            lights[light_id as usize].set_visible(false);
            light_id += diff;
            while light_id < 0 {
                light_id += lights.len() as i8;
            }
            light_id %= lights.len() as i8;
            lights[light_id as usize].set_visible(true);
        }

        win.render(&cam);
    }
}
