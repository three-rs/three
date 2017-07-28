
extern crate gltf;
extern crate three;

fn load_mesh(mesh: gltf::mesh::Mesh, factory: &mut three::Factory) -> three::Mesh {
    let primitive = mesh.primitives().nth(0).unwrap();
    let mut faces = vec![];
    match primitive.indices().unwrap() {
        gltf::mesh::Indices::U8(mut i) => {
            while let (Some(a), Some(b), Some(c)) = (i.next(), i.next(), i.next()) {
                faces.push([a as u32, b as u32, c as u32])
            }
        },
        gltf::mesh::Indices::U16(mut i) => {
            while let (Some(a), Some(b), Some(c)) = (i.next(), i.next(), i.next()) {
                faces.push([a as u32, b as u32, c as u32])
            }
        },
        gltf::mesh::Indices::U32(mut i) => {
            while let (Some(a), Some(b), Some(c)) = (i.next(), i.next(), i.next()) {
                faces.push([a, b, c]);
            }
        },
    };
    let vertices = primitive.positions().unwrap().map(|x| x.into()).collect();
    let normals = primitive.normals().unwrap().map(|x| x.into()).collect();
    let tangents = primitive.tangents().unwrap().map(|x| x.into()).collect();
    let mut tex_coords = vec![];
    match primitive.tex_coords(0).unwrap() {
        gltf::mesh::TexCoords::U8(iter) => {
            for x in iter {
                let y = [(x[0] as f32).abs() / 255.0, (x[1] as f32).abs() / 255.0];
                tex_coords.push(y.into());
            }
        },
        gltf::mesh::TexCoords::U16(iter) => {
            for x in iter {
                let y = [(x[0] as f32).abs() / 65535.0, (x[1] as f32).abs() / 65535.0];
                tex_coords.push(y.into());
            }
        },
        gltf::mesh::TexCoords::F32(iter) => {
            for x in iter {
                let y = [x[0].abs(), x[1].abs()];
                tex_coords.push(y.into());
            }
        },
    }
    let geometry = three::Geometry {
        base_shape: three::GeometryShape {
            vertices: vertices,
            normals: normals,
            tangents: tangents,
            tex_coords: tex_coords,
        },
        faces: faces,
        .. three::Geometry::empty()
    };
    let material = {
        let mat = primitive.material().unwrap();
        let pbr = mat.pbr_metallic_roughness().unwrap();
        let mut load = |texture: &gltf::texture::Texture| {
            let image = texture.source().data().to_rgba();
            let (width, height) = (image.width() as u16, image.height() as u16);
            factory.load_texture_from_memory(width, height, &image)
        };
        three::Material::MeshPbr {
            base_color_factor: pbr.base_color_factor(),
            metallic_roughness: [pbr.metallic_factor(), pbr.roughness_factor()],
            occlusion_strength: mat.occlusion_texture().map_or(1.0, |t| t.strength()),
            emissive_factor: mat.emissive_factor(),
            normal_scale: mat.normal_texture().map_or(1.0, |t| t.scale()),
            
            base_color_map: pbr.base_color_texture().map(|t| load(&t)),
            normal_map: mat.normal_texture().map(|t| load(&t)),
            emissive_map: mat.emissive_texture().map(|t| load(&t)),
            metallic_roughness_map: pbr.metallic_roughness_texture().map(|t| load(&t)),
            occlusion_map: mat.occlusion_texture().map(|t| load(&t)),
        }
    };
    factory.mesh(geometry, material)
}

fn import(path_str: &str, factory: &mut three::Factory) -> three::Mesh {
    let gltf = gltf::Import::from_path(path_str).sync().unwrap();
    let mesh = gltf.meshes().nth(0).unwrap();
    load_mesh(mesh, factory)
}

fn main() {
    let mut win = three::Window::new("Three-rs PBR example", "data/shaders").build();
    let mut cam = win.factory.perspective_camera(75.0, 0.01, 100.0);
    let mut yaw: f32 = 0.8;
    let mut distance: f32 = 3.9;
    cam.set_position([distance * yaw.cos(), 0.0, distance * yaw.sin()]);

    let mut light = win.factory.directional_light(0xFFFFFF, 7.0);
    light.look_at([1.0, 1.0, 1.0], [0.0, 0.0, 0.0], Some([0.0, 1.0, 0.0].into()));
    win.scene.add(&light);
    win.scene.background = three::Background::Color(0xC6F0FF);

    let path = std::env::args().nth(1).unwrap_or(format!("test_data/Avocado.gltf"));
    let mesh = import(&path, &mut win.factory);
    win.scene.add(&mesh);

    while win.update() && !three::KEY_ESCAPE.is_hit(&win.input) {
        if three::Button::Key(three::Key::Left).is_hit(&win.input) {
            yaw -= 0.05;
        }
        if three::Button::Key(three::Key::Right).is_hit(&win.input) {
            yaw += 0.05;
        }
        if three::Button::Key(three::Key::Up).is_hit(&win.input) {
            distance -= 0.05;
        }
        if three::Button::Key(three::Key::Down).is_hit(&win.input) {
            distance += 0.05;
        }
        if three::Button::Key(three::Key::P).is_hit(&win.input) {
            println!("yaw: {}, distance: {}", yaw, distance);
        }
        let (x, y, z) = (yaw.sin(), 0.0, yaw.cos());
        cam.look_at(
            [distance * x, y, distance * z],
            [0.0, 0.0, 0.0],
            Some([0.0, 1.0, 0.0].into()),
        );
        win.render(&cam);
    }
}
