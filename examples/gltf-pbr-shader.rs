extern crate three;

use three::{camera::Camera, Object};

fn main() {
    let mut win = three::Window::new("Three-rs glTF example");
    let light = win.factory.directional_light(0xFFFFFF, 7.0);
    light.look_at([1.0, 1.0, 1.0], [0.0, 0.0, 0.0], None);
    win.scene.add(&light);
    win.scene.background = three::Background::Color(0xC6F0FF);

    let default = concat!(env!("CARGO_MANIFEST_DIR"), "/test_data/Lantern/Lantern.gltf");
    let path = std::env::args().nth(1).unwrap_or(default.into());
    println!("Loading {:?} (this may take a while)", path);

    // Load the contents of the glTF files. Scenes loaded from the file are returned as
    // `Template` objects, which can be used to instantiate the actual objects for rendering.
    let templates = win.factory.load_gltf(&path);

    // Instantiate the contents of the template, and then add it to the scene.
    let (instance, _) = win.factory.instantiate_template(&templates[0]);
    win.scene.add(&instance);

    // Attempt to find a camera in the instantiated template to use as the perspective for
    // rendering.
    let cam = win.scene.sync_guard().find_child_of_type::<Camera>(&instance);

    // If we didn't find a camera in the glTF scene, create a default one to use.
    let cam = cam.unwrap_or_else(|| {
        let default = win.factory.perspective_camera(60.0, 0.001 .. 100.0);
        win.scene.add(&default);
        default
    });

    // Create a skybox for the scene.
    let skybox_path = three::CubeMapPath { front: "test_data/skybox/posz.jpg", back: "test_data/skybox/negz.jpg", up: "test_data/skybox/posy.jpg", down: "test_data/skybox/negy.jpg", left: "test_data/skybox/negx.jpg", right: "test_data/skybox/posx.jpg" };
    let skybox = win.factory.load_cubemap(&skybox_path);
    win.scene.background = three::Background::Skybox(skybox);

    // Determine the current position of the camera so that we can use it to initialize the
    // camera controller.
    let init = win.scene.sync_guard().resolve_world(&cam).transform;

    // Create a first person camera controller, starting at the camera's current position.
    let mut controls = three::controls::FirstPerson::builder(&cam).position(init.position).move_speed(4.0).build();

    // Run the main loop, updating the camera controller and rendering the scene every frame.
    while win.update() && !win.input.hit(three::KEY_ESCAPE) {
        controls.update(&win.input);
        win.render(&cam);
    }
}
