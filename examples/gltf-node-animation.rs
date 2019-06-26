extern crate three;

use three::Object;

fn main() {
    let mut window = three::Window::new("Three-rs glTF animation example");
    let light = window.factory.directional_light(0xFFFFFF, 0.4);
    light.look_at([1.0, -5.0, 10.0], [0.0, 0.0, 0.0], None);
    window.scene.add(&light);
    window.scene.background = three::Background::Color(0xC6F0FF);

    let default = concat!(env!("CARGO_MANIFEST_DIR"), "/test_data/BoxAnimated/BoxAnimated.gltf");
    let path = std::env::args().nth(1).unwrap_or(default.into());

    // Load the contents of the glTF files. Scenes loaded from the file are returned as
    // `Template` objects, which can be used to instantiate the actual objects for rendering.
    let templates = window.factory.load_gltf(&path);

    // Instantiate the contents of the template, and then add it to the scene.
    let (instance, animations) = window.factory.instantiate_template(&templates[0]);
    window.scene.add(&instance);

    // Start playing all the animations from the template.
    let mut mixer = three::animation::Mixer::new();
    for animation in animations {
        mixer.action(animation);
    }

    // Create a camera with which to render the scene, and control it with the built-in
    // orbit controller, set to orbit the model.
    let camera = window.factory.perspective_camera(60.0, 0.1 .. 100.0);
    let mut controls = three::controls::Orbit::builder(&camera).position([3.0, 3.0, 3.0]).target([0.0, 1.0, 0.0]).build();

    // Run the main loop, updating the camera controller, animations, and rendering the scene
    // every frame.
    while window.update() && !window.input.hit(three::KEY_ESCAPE) {
        mixer.update(window.input.delta_time());
        controls.update(&window.input);
        window.render(&camera);
    }
}
