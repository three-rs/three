extern crate three;

fn main() {
    let mut window = three::Window::new("Three-rs glTF animation example");
    let mut light = window.factory.directional_light(0xFFFFFF, 0.4);
    light.look_at([1.0, 1.0, 1.0], [0.0, 0.0, 0.0], None);
    window.scene.add(&light);
    window.scene.background = three::Background::Color(0xC6F0FF);

    let default = concat!(env!("CARGO_MANIFEST_DIR"), "/test_data/BoxAnimated.gltf");
    let path = std::env::args().nth(1).unwrap_or(default.into());
    let gltf = window.factory.load_gltf(&path);
    window.scene.add(&gltf.group);

    let mut mixer = three::animation::Mixer::new();
    for clip in gltf.clips {
        mixer.action(clip);
    }

    let mut camera = window.factory.perspective_camera(60.0, 0.1 .. 10.0);
    camera.set_position([0.0, 1.0, 5.0]);
    while window.update() && !three::KEY_ESCAPE.is_hit(&window.input) {
        mixer.update(window.input.delta_time());
        window.render(&camera);
    }
}
