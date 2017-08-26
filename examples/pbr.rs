
extern crate gltf;
extern crate gltf_importer;
extern crate gltf_utils;
extern crate image;
extern crate three;

use image::ImageFormat::{JPEG as Jpeg, PNG as Png};
use std::{fs, io};

fn load_texture(
    texture: &AsRef<gltf::texture::Texture>,
    buffers: &gltf_importer::Buffers,
    factory: &mut three::Factory,
) -> three::Texture<[f32; 4]> {
    let texture = texture.as_ref();
    let image = match texture.source().data() {
        gltf::image::Data::View { view, mime_type } => {
            let format = match mime_type {
                "image/png" => Png,
                "image/jpeg" => Jpeg,
                _ => unreachable!(),
            };
            let data = buffers.view(&view).unwrap();
            image::load_from_memory_with_format(&data, format)
                .unwrap()
                .to_rgba()
        },
        gltf::image::Data::Uri { uri, mime_type } => {
            let path = format!("test_data/{}", uri);
            if let Some(ty) = mime_type {
                let format = match ty {
                    "image/png" => Png,
                    "image/jpeg" => Jpeg,
                    _ => unreachable!(),
                };
                let file = fs::File::open(&path).unwrap();
                let reader = io::BufReader::new(file);
                image::load(reader, format)
                    .unwrap()
                    .to_rgba()
            } else {
                image::open(&path)
                    .unwrap()
                    .to_rgba()
            }
        },
    };
    let (width, height) = (image.width() as u16, image.height() as u16);
    use gltf::texture::{MagFilter, MinFilter, WrappingMode};
    use three::{FilterMethod, WrapMode};
    let params = texture.sampler();
    // gfx does not support separate min/mag
    // filters yet, so for now we'll use min_filter for both.
    let _mag_filter = match params.mag_filter() {
        None | Some(MagFilter::Nearest) => FilterMethod::Scale,
        Some(MagFilter::Linear) => FilterMethod::Bilinear,
    };
    let min_filter = match params.min_filter() {
        None | Some(MinFilter::Nearest) => FilterMethod::Scale,
        Some(MinFilter::Linear) => FilterMethod::Bilinear,
        // Texture mipmapping must be implemented before
        // this option may be used.
        _ => unimplemented!(),
    };
    let wrap_s = match params.wrap_s() {
        WrappingMode::ClampToEdge => WrapMode::Clamp,
        WrappingMode::MirroredRepeat => WrapMode::Mirror,
        WrappingMode::Repeat => WrapMode::Tile,
    };
    let wrap_t = match params.wrap_t() {
        WrappingMode::ClampToEdge => WrapMode::Clamp,
        WrappingMode::MirroredRepeat => WrapMode::Mirror,
        WrappingMode::Repeat => WrapMode::Tile,
    };
    let sampler = factory.sampler(min_filter, wrap_s, wrap_t);
    factory.load_texture_from_memory(width, height, &image, sampler)
}

fn load_mesh(
    mesh: gltf::mesh::Mesh,
    buffers: &gltf_importer::Buffers,
    factory: &mut three::Factory,
) -> three::Mesh {
    use gltf_utils::PrimitiveIterators;
    let primitive = mesh.primitives().nth(0).unwrap();
    let mut faces = vec![];
    let mut i = primitive.indices_u32(buffers).unwrap();
    while let (Some(a), Some(b), Some(c)) = (i.next(), i.next(), i.next()) {
        faces.push([a as u32, b as u32, c as u32])
    }
    let vertices = primitive
        .positions(buffers)
        .unwrap()
        .map(|x| x.into())
        .collect();
    let normals = primitive
        .normals(buffers)
        .unwrap()
        .map(|x| x.into())
        .collect();
    let tangents = primitive
        .tangents(buffers)
        .unwrap()
        .map(|x| x.into())
        .collect();
    let tex_coords = primitive
        .tex_coords_f32(buffers, 0)
        .unwrap()
        .map(|x| x.into())
        .collect();
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
        let mat = primitive.material();
        let pbr = mat.pbr_metallic_roughness();
        three::Material::MeshPbr {
            base_color_factor: pbr.base_color_factor(),
            metallic_roughness: [pbr.metallic_factor(), pbr.roughness_factor()],
            occlusion_strength: mat.occlusion_texture().map_or(1.0, |t| t.strength()),
            emissive_factor: mat.emissive_factor(),
            normal_scale: mat.normal_texture().map_or(1.0, |t| t.scale()),
            
            base_color_map: pbr.base_color_texture().map(|t| load_texture(&t, buffers, factory)),
            normal_map: mat.normal_texture().map(|t| load_texture(&t, buffers, factory)),
            emissive_map: mat.emissive_texture().map(|t| load_texture(&t, buffers, factory)),
            metallic_roughness_map: pbr.metallic_roughness_texture().map(|t| load_texture(&t, buffers, factory)),
            occlusion_map: mat.occlusion_texture().map(|t| load_texture(&t, buffers, factory)),
        }
    };
    factory.mesh(geometry, material)
}

fn main() {
    let shaders = concat!(env!("CARGO_MANIFEST_DIR"), "/data/shaders");
    let mut win = three::Window::new("Three-rs PBR example", shaders).build();
    let mut cam = win.factory.perspective_camera(75.0, 0.01, 100.0);
    let mut yaw: f32 = 0.8;
    let mut distance: f32 = 3.9;
    cam.set_position([distance * yaw.cos(), 0.0, distance * yaw.sin()]);

    let mut light = win.factory.directional_light(0xFFFFFF, 7.0);
    light.look_at([1.0, 1.0, 1.0], [0.0, 0.0, 0.0], Some([0.0, 1.0, 0.0].into()));
    win.scene.add(&light);
    win.scene.background = three::Background::Color(0xC6F0FF);

    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/test_data/SciFiHelmet.gltf");
    let (gltf, buffers) = gltf_importer::import(path).expect("invalid glTF");
    let mesh = load_mesh(gltf.meshes().nth(0).unwrap(), &buffers, &mut win.factory);
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
