extern crate three;

use three::Object;

fn main() {
    let mut window = three::Window::new("Three-rs glTF animation example");
    let light = window.factory.directional_light(0xFFFFFF, 0.4);
    light.look_at([1.0, -5.0, 10.0], [0.0, 0.0, 0.0], None);
    window.scene.add(&light);
    window.scene.background = three::Background::Color(0xC6F0FF);

    // Load the glTF file.
    let default = concat!(env!("CARGO_MANIFEST_DIR"), "/test_data/BrainStem/BrainStem.gltf");
    let path = std::env::args().nth(1).unwrap_or(default.into());
    let gltf = window.factory.load_gltf(&path);

    // Instantiate the contents of the file.
    let instance = window.factory.instantiate_gltf_scene(&gltf, 0);
    window.scene.add(&instance);

    // Instantiate all of the animations in the glTF file and start playing them.
    let mut mixer = three::animation::Mixer::new();
    for anim_def in &gltf.animations {
        let clip = window.factory.instantiate_gltf_animation(&instance, anim_def).unwrap();
        mixer.action(clip);
    }

    let camera = window.factory.perspective_camera(45.0, 0.1 .. 100.0);
    window.scene.add(&camera);

    let mut controls = three::controls::Orbit::builder(&camera)
        .position([0.0, -3.0, 3.0])
        .target([0.0, 0.0, 1.0])
        .build();

    while window.update() && !window.input.hit(three::KEY_ESCAPE) {
        mixer.update(window.input.delta_time());
        controls.update(&window.input);
        window.render(&camera);
    }
}
