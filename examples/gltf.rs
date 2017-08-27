
extern crate cgmath;
extern crate gltf;
extern crate gltf_importer;
extern crate gltf_utils;
extern crate three;
extern crate image;
extern crate vec_map;

use cgmath::prelude::*;
use std::{fs, io};

use image::ImageFormat::{JPEG as Jpeg, PNG as Png};
use vec_map::VecMap;

fn load_mesh(
    mesh: gltf::Mesh,
    buffers: &gltf_importer::Buffers,
    factory: &mut three::Factory,
) -> three::Mesh {
    use gltf_utils::PrimitiveIterators;
    let primitive = mesh.primitives().nth(0).unwrap();
    let mut faces = vec![];
    let mut i = primitive.indices_u32(buffers).unwrap();
    while let (Some(a), Some(b), Some(c)) = (i.next(), i.next(), i.next()) {
        faces.push([a, b, c]);
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
        .tex_coords_f32(0, buffers)
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
        let mut load = |texture: gltf::texture::Texture| {
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
        };
        three::Material::MeshPbr {
            base_color_factor: pbr.base_color_factor(),
            metallic_roughness: [pbr.metallic_factor(), pbr.roughness_factor()],
            occlusion_strength: mat.occlusion_texture().map_or(1.0, |t| t.strength()),
            emissive_factor: mat.emissive_factor(),
            normal_scale: mat.normal_texture().map_or(1.0, |t| t.scale()),
            
            base_color_map: pbr.base_color_texture().map(|t| load(t.texture())),
            normal_map: mat.normal_texture().map(|t| load(t.texture())),
            emissive_map: mat.emissive_texture().map(|t| load(t.texture())),
            metallic_roughness_map: pbr.metallic_roughness_texture().map(|t| load(t.texture())),
            occlusion_map: mat.occlusion_texture().map(|t| load(t.texture())),
        }
    };
    factory.mesh(geometry, material)
}

struct State {
    yaw: f32,
    pitch: f32,
    look_speed: f32,
    move_speed: f32,
    position: cgmath::Vector3<f32>,
}

fn make_group(
    node: gltf::Node,
    buffers: &gltf_importer::Buffers,
    meshes: &mut VecMap<three::Mesh>,
    factory: &mut three::Factory,
) -> three::Group {
    fn recurse(
        node: gltf::Node,
        buffers: &gltf_importer::Buffers,
        parent: &mut three::Group,
        meshes: &mut VecMap<three::Mesh>,
        factory: &mut three::Factory,
    ) {
        let mut group = factory.group();
        group.set_transform(node.translation(), node.rotation(), node.scale()[1]);
        
        if let Some(entry) = node.mesh() {
            let index = entry.index();
            let mesh = load_mesh(entry, buffers, factory);
            group.add(&mesh);
            assert!(meshes.get(index).is_none());
            meshes.insert(index, mesh);
        }

        if node.children().len() > 0 {
            for child in node.children() {
                recurse(child, buffers, &mut group, meshes, factory);
            }
        }

        parent.add(&group);
    }

    let mut root = factory.group();
    recurse(node, buffers, &mut root, meshes, factory);
    root
}

fn main() {
    let shaders_path = format!("{}/data/shaders", env!("CARGO_MANIFEST_DIR"));
    let mut win = three::Window::new("Three-rs glTF example", &shaders_path).build();
    let mut cam = win.factory.perspective_camera(60.0, 0.001, 100.0);
    let mut st = State {
        yaw: 5.73,
        pitch: 0.0,
        look_speed: 0.03,
        move_speed: 0.05,
        position: [11.3, 12.5, 22.5].into(),
    };

    let mut light = win.factory.directional_light(0xFFFFFF, 7.0);
    light.look_at([1.0, 1.0, 1.0], [0.0, 0.0, 0.0], None);
    win.scene.add(&light);
    win.scene.background = three::Background::Color(0xC6F0FF);

    let default_path = "test_data/Lantern.gltf".into();
    let path = std::env::args().nth(1).unwrap_or(default_path);
    let (gltf, buffers) = gltf_importer::import(&path).expect("invalid glTF");

    // The head node in the glTF scene hierarchy.
    let head = gltf.nodes().nth(3).unwrap();
    // Meshes must persist for the lifetime of the rendered item.
    let mut meshes = VecMap::<three::Mesh>::with_capacity(gltf.meshes().len());
    // Groups *must* be created before meshes.
    let root = make_group(head, &buffers, &mut meshes, &mut win.factory);

    win.scene.add(&root);

    while win.update() && !three::KEY_ESCAPE.is_hit(&win.input) {
        if three::Button::Key(three::Key::Q).is_hit(&win.input) {
            st.yaw -= st.look_speed;
        }
        if three::Button::Key(three::Key::E).is_hit(&win.input) {
            st.yaw += st.look_speed;
        }
        if three::Button::Key(three::Key::R).is_hit(&win.input) {
            st.pitch -= st.look_speed;
        }
        if three::Button::Key(three::Key::F).is_hit(&win.input) {
            st.pitch += st.look_speed;
        }
        if three::Button::Key(three::Key::X).is_hit(&win.input) {
            st.position.y += st.move_speed;
        }
        if three::Button::Key(three::Key::Z).is_hit(&win.input) {
            st.position.y -= st.move_speed;
        }
        if three::Button::Key(three::Key::W).is_hit(&win.input) {
            st.position.x += st.move_speed * st.yaw.sin();
            st.position.z -= st.move_speed * st.yaw.cos();
        }
        if three::Button::Key(three::Key::S).is_hit(&win.input) {
            st.position.x -= st.move_speed * st.yaw.sin();
            st.position.z += st.move_speed * st.yaw.cos();
        }
        if three::Button::Key(three::Key::D).is_hit(&win.input) {
            st.position.x += st.move_speed * st.yaw.cos();
            st.position.z += st.move_speed * st.yaw.sin();
        }
        if three::Button::Key(three::Key::A).is_hit(&win.input) {
            st.position.x -= st.move_speed * st.yaw.cos();
            st.position.z -= st.move_speed * st.yaw.sin();
        }

        let yrot = cgmath::Quaternion::<f32>::from_angle_y(cgmath::Rad(-st.yaw));
        let xrot = cgmath::Quaternion::<f32>::from_angle_x(cgmath::Rad(-st.pitch));
        cam.set_transform(cgmath::Point3::from_vec(st.position), yrot * xrot, 1.0);
        win.render(&cam); 
    }
}
