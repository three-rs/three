extern crate three;

fn main() {
    let mut cam = three::PerspectiveCamera::new(75.0, 0.0, 1.0, 50.0);
    cam.position = three::Position::new(0.0, 0.0, 10.0);
    let mut win = three::Window::new("Three-rs materials example", cam);

    let mut light = win.factory.point_light(0xffffff, 0.5);
    light.transform_mut().disp = three::Vector::new(0.0, 5.0, 5.0);
    win.scene.add(&light);

    let geometry = three::Geometry::new_box(2.0, 2.0, 2.0);
    let mut materials = vec![
        three::Material::MeshBasic{ color: 0xffffff, wireframe: false },
        three::Material::MeshLambert{ color: 0xffffff },
        //three::Material::MeshPhong{ color: 0xffffff },
    ];
    let count = materials.len();

    let _cubes: Vec<_> = materials.drain(..).enumerate().map(|(i, mat)| {
        let offset = 4.0 * (i as f32 + 0.5 - 0.5 * count as f32);
        let mut mesh = win.factory.mesh(geometry.clone(), mat);
        mesh.transform_mut().disp = three::Vector::new(offset, 0.0, 0.0);
        win.scene.add(&mesh);
        mesh
    }).collect();

    let speed = 1.5;
    while let Some(events) = win.update() {
        if events.keys.contains(&three::Key::Left) {
            light.transform_mut().disp.x -= speed * events.time_delta;
        }
        if events.keys.contains(&three::Key::Right) {
            light.transform_mut().disp.x += speed * events.time_delta;
        }

        win.render();
    }
}
