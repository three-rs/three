extern crate three;

fn main() {
    let mut win = three::Window::new("Three-rs lights example");
    let mut cam = win.factory.perspective_camera(45.0, 1.0 .. 50.0);
    cam.look_at([-4.0, 15.0, 10.0], [0.0, 0.0, 2.0], None);

    let mut hemisphere_light = win.factory.hemisphere_light(0xffffff, 0x8080ff, 0.5);
    let mut ambient_light = win.factory.ambient_light(0xffffffff, 0.5);
    let mut point_light = win.factory.point_light(0xffffff, 0.9);
    point_light.set_position([15.0, 35.0, 35.0]);

    let mut dir_light = win.factory.directional_light(0xffffff, 0.9);
    dir_light.look_at([15.0, 35.0, 35.0], [0.0, 0.0, 2.0], None);
    let shadow_map = win.factory.shadow_map(1024, 1024);
    let _debug_shadow = win.renderer
        .debug_shadow_quad(&shadow_map, 1, [10, 10], [256, 256]);
    dir_light.set_shadow(shadow_map, 40.0, 1.0 .. 200.0);

    let mut lights: [&mut three::Object; 4] = [
        &mut hemisphere_light,
        &mut ambient_light,
        &mut point_light,
        &mut dir_light,
    ];
    for l in lights.iter_mut() {
        l.set_parent(&win.scene);
        l.set_visible(false);
    }

    let mut sphere = {
        let geometry = three::Geometry::uv_sphere(3.0, 20, 20);
        let material = three::material::Phong {
            color: 0xffA0A0,
            glossiness: 80.0,
        };
        win.factory.mesh(geometry, material)
    };
    sphere.set_position([0.0, 0.0, 2.5]);
    sphere.set_parent(&win.scene);

    let mut plane = {
        let geometry = three::Geometry::plane(100.0, 100.0);
        let material = three::material::Lambert {
            color: 0xA0ffA0,
            flat: false,
        };
        win.factory.mesh(geometry, material)
    };
    plane.set_position([0.0, -30.0, 0.0]);
    plane.set_parent(&win.scene);

    let mut light_id = 0i8;
    lights[0].set_visible(true);
    while win.update() && !win.input.hit(three::KEY_ESCAPE) {
        if let Some(axis_hits) = win.input.delta(three::AXIS_LEFT_RIGHT) {
            lights[light_id as usize].set_visible(false);
            light_id += axis_hits;
            while light_id < 0 {
                light_id += lights.len() as i8;
            }
            light_id %= lights.len() as i8;
            lights[light_id as usize].set_visible(true);
        }

        win.render(&cam);
    }
}
