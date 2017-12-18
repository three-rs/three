extern crate three;

use three::Object;

fn main() {
    let mut window = three::Window::new("Three-rs glTF animation example");
    let light = window.factory.directional_light(0xFFFFFF, 0.4);
    light.look_at([1.0, 1.0, 1.0], [0.0, 0.0, 0.0], None);
    window.scene.add(&light);
    window.scene.background = three::Background::Color(0xC6F0FF);

    let default = concat!(env!("CARGO_MANIFEST_DIR"), "/test_data/BoxAnimated.gltf");
    let path = std::env::args().nth(1).unwrap_or(default.into());
    let gltf = window.factory.load_gltf(&path);
    window.scene.add(&gltf);

    let mut mixer = three::animation::Mixer::new();
    for clip in gltf.clips {
        mixer.action(clip);
    }

    let camera = window.factory.perspective_camera(60.0, 0.1 .. 10.0);
    camera.set_position([0.0, 1.0, 5.0]);

    let mut controls = three::controls::Orbit::builder(&camera)
        .position([0.0, 1.0, 9.5])
        .target([0.0, 0.0, 0.0])
        .build();

    while window.update() && !window.input.hit(three::KEY_ESCAPE) {
        mixer.update(window.input.delta_time());
        controls.update(&window.input);
        window.render(&camera);
    }
}
