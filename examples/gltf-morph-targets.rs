extern crate three;

use three::Object;

fn main() {
    let mut window = three::Window::new("Three-rs glTF animation example");
    let light = window.factory.directional_light(0xFFFFFF, 0.4);
    light.look_at([1.0, 1.0, 1.0], [0.0, 0.0, 0.0], None);
    window.scene.add(&light);
    window.scene.background = three::Background::Color(0xC6F0FF);

    // Load the glTF file.
    let default = concat!(env!("CARGO_MANIFEST_DIR"), "/test_data/AnimatedMorphCube/AnimatedMorphCube.gltf");
    let path = std::env::args().nth(1).unwrap_or(default.into());
    let template = window.factory.load_gltf(&path).pop().unwrap();

    // Instantiate the contents of the file.
    let (instance, animations) = window.factory.instantiate_template(&template);
    window.scene.add(&instance);

    // Instantiate all of the animations in the glTF file and start playing them.
    let mut mixer = three::animation::Mixer::new();
    for animation in animations {
        mixer.action(animation);
    }

    let camera = window.factory.perspective_camera(60.0, 0.1 .. 20.0);

    let mut controls = three::controls::Orbit::builder(&camera)
        .position([-3.0, 3.0, -3.0])
        .up([0.0, 1.0, 0.0])
        .build();

    while window.update() && !window.input.hit(three::KEY_ESCAPE) {
        mixer.update(window.input.delta_time());
        controls.update(&window.input);
        window.render(&camera);
    }
}
