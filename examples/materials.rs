extern crate three;

fn main() {
    let mut win = three::Window::new("Three-rs materials example", "data/shaders");
    let mut cam = win.factory.perspective_camera(75.0, 0.0, 1.0, 50.0);
    cam.transform_mut().position = [0.0, 0.0, 10.0].into();

    let mut light = win.factory.point_light(0xffffff, 0.5);
    light.transform_mut().position = [0.0, 5.0, 5.0].into();
    win.scene.add(&light);

    let geometry = three::Geometry::new_cylinder(1.0, 2.0, 2.0, 5);
    let mut materials = vec![
        three::Material::MeshBasic{ color: 0xffffff, map: None, wireframe: false },
        three::Material::MeshLambert{ color: 0xffffff },
        three::Material::MeshPhong{ color: 0xffffff, glossiness: 80.0 },
    ];
    let count = materials.len();

    let _cubes: Vec<_> = materials.drain(..).enumerate().map(|(i, mat)| {
        let offset = 4.0 * (i as f32 + 0.5 - 0.5 * count as f32);
        let mut mesh = win.factory.mesh(geometry.clone(), mat);
        mesh.transform_mut().position = [offset, 0.0, 0.0].into();
        win.scene.add(&mesh);
        mesh
    }).collect();

    let speed = 5.0;
    while let Some(events) = win.update() {
        if events.keys.contains(&three::Key::Left) {
            light.transform_mut().position.x -= speed * events.time_delta;
        }
        if events.keys.contains(&three::Key::Right) {
            light.transform_mut().position.x += speed * events.time_delta;
        }

        win.render(&cam);
    }
}
