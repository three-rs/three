extern crate three;

use three::Object;

fn main() {
    let mut window = three::Window::new("Three-rs glTF animation example");
    let mut light = window.factory.directional_light(0xFFFFFF, 0.4);
    light.look_at([1.0, 1.0, 1.0], [0.0, 0.0, 0.0], None);
    light.set_parent(&window.scene);
    window.scene.background = three::Background::Color(0xC6F0FF);

    let default = concat!(env!("CARGO_MANIFEST_DIR"), "/test_data/BoxAnimated.gltf");
    let path = std::env::args().nth(1).unwrap_or(default.into());
    let mut gltf = window.factory.load_gltf(&path);
    gltf.set_parent(&window.scene);

    let mut mixer = three::animation::Mixer::new();
    for clip in gltf.clips {
        mixer.action(clip);
    }

    let mut camera = window.factory.perspective_camera(60.0, 0.1 .. 100.0);
    camera.set_position([0.0, 1.0, 0.2]);

    let mut controls = three::controls::Orbit::builder(&camera)
        .position([0.0, 1.0, 0.2])
        .target([0.0, 0.0, 0.0])
        .build();
    
    while window.update() && !window.input.hit(three::KEY_ESCAPE) {
        mixer.update(window.input.delta_time());
        controls.update(&window.input);
        window.render(&camera);
    }
}
