extern crate three;

fn main() {
    let title = "Getting started with three-rs";
    let shaders = concat!(env!("CARGO_MANIFEST_DIR"), "/data/shaders");
    let mut window = three::Window::builder(title, shaders).build();

    let vertices = vec![
        [-0.5, -0.5, -0.5].into(),
        [0.5, -0.5, -0.5].into(),
        [0.0, 0.5, -0.5].into(),
    ];
    let geometry = three::Geometry::with_vertices(vertices);
    let material = three::Material::MeshBasic {
        color: 0xFFFF00,
        wireframe: false,
        map: None,
    };
    let mesh = window.factory.mesh(geometry, material);

    window.scene.add(&mesh);
    window.scene.background = three::Background::Color(0xC6F0FF);

    let center = [0.0, 0.0];
    let yextent = 1.0;
    let zrange = -1.0 .. 1.0;
    let camera = window.factory.orthographic_camera(center, yextent, zrange);

    while window.update() {
        window.render(&camera);
    }
}
