extern crate three;

fn main() {
    let shaders_path: String = format!("{}/data/shaders", env!("CARGO_MANIFEST_DIR"));
    let shaders_path_str: &str = shaders_path.as_str();
    let mut win = three::Window::builder("Three-rs materials example", shaders_path_str).build();
    let mut cam = win.factory.perspective_camera(75.0, 1.0 .. 50.0);
    cam.set_position([0.0, 0.0, 10.0]);

    let mut light = win.factory.point_light(0xffffff, 0.5);
    let mut pos = [0.0, 5.0, 5.0];
    light.set_position(pos);
    win.scene.add(&light);

    let geometry = three::Geometry::cylinder(1.0, 2.0, 2.0, 5);
    let mut materials = vec![
        three::Material::MeshBasic { color: 0xffffff, map: None, wireframe: false },
        three::Material::MeshLambert { color: 0xffffff, flat: true },
        three::Material::MeshLambert { color: 0xffffff, flat: false },
        three::Material::MeshPhong { color: 0xffffff, glossiness: 80.0 },
        three::Material::MeshPbr {
            base_color_factor: [0.2, 0.2, 0.2, 1.0],
            metallic_roughness: [0.5, 0.5],
            occlusion_strength: 0.2,
            emissive_factor: [0.0, 0.0, 0.0],
            normal_scale: 1.0,
            base_color_map: None,
            normal_map: None,
            emissive_map: None,
            metallic_roughness_map: None,
            occlusion_map: None,
        },
    ];
    let count = materials.len();

    let _cubes: Vec<_> = materials.drain(..).enumerate().map(|(i, mat)| {
        let offset = 4.0 * (i as f32 + 0.5 - 0.5 * count as f32);
        let mut mesh = win.factory.mesh(geometry.clone(), mat);
        mesh.set_position([offset, 0.0, 0.0]);
        win.scene.add(&mesh);
        mesh
    }).collect();

    while win.update() && !three::KEY_ESCAPE.is_hit(&win.input) {
        if let Some(diff) = three::AXIS_LEFT_RIGHT.timed(&win.input) {
            pos[0] += 5.0 * diff;
            light.set_position(pos);
        }

        win.render(&cam);
    }
}
