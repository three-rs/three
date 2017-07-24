
extern crate bincode;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate three;

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Model<'a> {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    tangents: Vec<[f32; 4]>,
    tex_coords: Vec<[f32; 2]>,
    faces: Vec<[u16; 3]>,

    base_color_factor: [f32; 4],
    metallic_roughness: [f32; 2],
    emissive_factor: [f32; 3],
    normal_scale: f32,
    occlusion_strength: f32,
    #[serde(borrow)]
    base_color_map: Option<&'a str>,
    #[serde(borrow)]
    normal_map: Option<&'a str>,
    #[serde(borrow)]
    metallic_roughness_map: Option<&'a str>,
    #[serde(borrow)]
    emissive_map: Option<&'a str>,
    #[serde(borrow)]
    occlusion_map: Option<&'a str>,
}

fn main() {
    let mut win = three::Window::new("Three-rs PBR example", "data/shaders").build();
    let mut cam = win.factory.perspective_camera(75.0, 0.1, 1.0);
    cam.set_position([0.0, 0.0, 0.15]);

    let mut light = win.factory.point_light(0xffffff, 0.5);
    let pos = [0.0, 5.0, 5.0];
    light.set_position(pos);
    win.scene.add(&light);

    let model_data = {
        use std::io::Read;
        let mut buffer = vec![];
        let file = std::fs::File::open("test_data/Avocado.model").unwrap();
        let _ = std::io::BufReader::new(file).read_to_end(&mut buffer).unwrap();
        buffer
    };
    let model: Model = bincode::deserialize(&model_data).unwrap();

    let geometry = three::Geometry {
        base_shape: three::GeometryShape {
            vertices: model.positions.iter().cloned().map(|x| x.into()).collect(),
            normals: model.normals.iter().cloned().map(|x| x.into()).collect(),
            tangents: model.tangents.iter().cloned().map(|x| x.into()).collect(),
            tex_coords: model.tex_coords.iter().cloned().map(|x| [x[0], -x[1]].into()).collect(),
        },
        faces: model.faces.clone(),
        ..three::Geometry::empty()
    };
    let material = three::Material::MeshPbr {
        base_color_factor: model.base_color_factor,
        metallic_roughness: model.metallic_roughness,
        emissive_factor: model.emissive_factor,
        normal_scale: model.normal_scale,
        base_color_map: model.base_color_map.map(|x| win.factory.load_texture(x)),
        normal_map: model.normal_map.map(|x| win.factory.load_texture(x)),
        metallic_roughness_map: model.metallic_roughness_map
            .map(|x| win.factory.load_texture(x)),
        emissive_map: model.emissive_map.map(|x| win.factory.load_texture(x)),
        occlusion_map: model.occlusion_map.map(|x| win.factory.load_texture(x)),
        occlusion_strength: model.occlusion_strength,
    };
    let mesh = win.factory.mesh(geometry, material);
    win.scene.add(&mesh);

    let mut yaw: f32 = 0.0;
    while win.update() && !three::KEY_ESCAPE.is_hit(&win.input) {
        if three::Button::Key(three::Key::Left).is_hit(&win.input) {
            yaw -= 0.1;
        }
        if three::Button::Key(three::Key::Right).is_hit(&win.input) {
            yaw += 0.1;
        }
        let (x, y, z) = (yaw.sin(), 0.0, yaw.cos());
        cam.look_at(
            [0.15 * x, y, 0.15 * z],
            [0.0, 0.0, 0.0],
            Some([0.0, 1.0, 0.0].into()),
        );
        win.render(&cam);
    }
}
