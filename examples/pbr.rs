
extern crate gltf;
extern crate three;

fn import(path_str: &str, factory: &mut three::Factory) -> three::Mesh {
    let gltf = gltf::Import::from_path(path_str).sync().unwrap();
    let mesh = gltf.meshes().nth(0).unwrap();
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
                let y = [x[0] as f32 / 255.0, x[1] as f32 / 255.0];
                tex_coords.push(y.into());
            }
        },
        gltf::mesh::TexCoords::U16(iter) => {
            for x in iter {
                let y = [x[0] as f32 / 65535.0, x[1] as f32 / 65535.0];
                tex_coords.push(y.into());
            }
        },
        gltf::mesh::TexCoords::F32(iter) => {
            for x in iter {
                tex_coords.push(x.into());
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
            let image = texture.source();
            let (width, height) = (image.width() as u16, image.height() as u16);
            let pixels = image.raw_pixels();
            factory.load_texture_from_memory(width, height, pixels)
        };
        three::Material::MeshPbr {
            base_color_factor: pbr.base_color_factor(),
            metallic_roughness: [pbr.metallic_factor(), pbr.roughness_factor()],
            occlusion_strength: mat.occlusion_texture().map_or(0.0, |t| t.strength()),
            emissive_factor: mat.emissive_factor(),
            normal_scale: mat.normal_texture().map_or(0.0, |t| t.scale()),
            
            base_color_map: pbr.base_color_texture().map(|t| load(&t)),
            normal_map: mat.normal_texture().map(|t| load(&t)),
            emissive_map: mat.emissive_texture().map(|t| load(&t)),
            metallic_roughness_map: pbr.metallic_roughness_texture().map(|t| load(&t)),
            occlusion_map: mat.occlusion_texture().map(|t| load(&t)),
        }
    };

    factory.mesh(geometry, material)
}

fn main() {
    let mut win = three::Window::new("Three-rs PBR example", "data/shaders").build();
    let mut cam = win.factory.perspective_camera(75.0, 0.1, 100.0);
    cam.set_position([0.0, 0.0, 10.0]);

    let mut light = win.factory.point_light(0xffffff, 0.5);
    let pos = [0.0, 5.0, 5.0];
    light.set_position(pos);
    win.scene.add(&light);
    win.scene.background = three::Background::Color(0xC6F0FF);

    let mesh = import("test_data/SciFiHelmet.gltf", &mut win.factory);
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
            [10.0 * x, y, 10.0 * z],
            [0.0, 0.0, 0.0],
            Some([0.0, 1.0, 0.0].into()),
        );
        win.render(&cam);
    }
}
