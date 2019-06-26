extern crate three;
extern crate nuklear;

use three::{gui, Object};
use nuklear::*;

struct OptionsGuiState {
    model_scale: f32,
    light: bool,
    anim_speed: f32,
    anim_time: f32,
}

// This struct will store all of our fonts and images, and implement a trait which will allow it to
// interact with the existing backend for Nuklear GUI that renders atop three-rs.
struct Media {
    body_font: FontID,
    title_font: FontID,
    avocado: nuklear::Image,
}
impl gui::nuklear_backend::MediaStorage for Media {
    fn build<F: gfx::Factory<three::render::BackendResources>>(
        f: &mut F,
        mut load: gui::nuklear_backend::ResourceLoader,
    ) -> Self {
        let mut cfg = FontConfig::with_size(0.0);
        cfg.set_oversample_h(3);
        cfg.set_oversample_v(2);
        cfg.set_glyph_range(font_cyrillic_glyph_ranges());
        cfg.set_ttf(include_bytes!("../data/fonts/DejaVuSans.ttf"));

        // using the method on loader which wraps Nuklear's loading stuff
        // to load a size 18 font.
        let body_font = load.font_with_size(&mut cfg, 18.0);

        // doing the same thing as above like you see in Nuklear's documentation
        // (good for certain customizations our loader doesn't support.)
        cfg.set_ttf_data_owned_by_atlas(false);
        cfg.set_size(22.0);
        let title_font = load.font_atlas.add_font_with_config(&cfg).unwrap();

        // https://github.com/snuk182/nuklear-rust/issues/17
        Self {
            title_font: body_font,
            body_font: title_font,
            avocado: load.image(f, concat!(env!("CARGO_MANIFEST_DIR"), "/test_data/Avocado/Avocado_baseColor.png")),
        }
    }

    fn first_font(&self, atlas: &FontAtlas) -> UserFont {
        atlas.font(self.body_font).unwrap().handle().clone()
    }
}

const LIGHT_INTENSITY: f32 = 0.4;
fn main() {
    let mut window = three::Window::<gui::NuklearBackend<Media>>::new("Three-rs glTF animation example");
    let config = &mut window.gui.config;
    config.set_circle_segment_count(22);
    config.set_curve_segment_count(22);
    config.set_arc_segment_count(22);
    config.set_global_alpha(1.0f32);
    config.set_shape_aa(AntiAliasing::On);
    config.set_line_aa(AntiAliasing::On);

    let light = window.factory.directional_light(0xFFFFFF, LIGHT_INTENSITY);
    light.look_at([1.0, -5.0, 10.0], [0.0, 0.0, 0.0], None);
    window.scene.add(&light);
    window.scene.background = three::Background::Color(0xC6F0FF);

    let default = concat!(env!("CARGO_MANIFEST_DIR"), "/test_data/BrainStem/BrainStem.gltf");
    let path = std::env::args().nth(1).unwrap_or(default.into());

    // Load the contents of the glTF files. Scenes loaded from the file are returned as
    // `Template` objects, which can be used to instantiate the actual objects for rendering.
    let templates = window.factory.load_gltf(&path);

    // Instantiate the contents of the template, and then add it to the scene.
    let (instance, animations) = window.factory.instantiate_template(&templates[0]);
    window.scene.add(&instance);

    // it's necessary to know the total length of the animation to know how to fill the progress bar.
    let total_anim_time: f32 = *animations
        // get the tracks in the last animation
        .last().unwrap().tracks
        // find the last track in that last animation's tracks
        .last().unwrap().0
        // the last keyframe's time index is also the length of the animation.
        .times.last().unwrap();

    // Begin playing all the animations instantiated from the template.
    let mut mixer = three::animation::Mixer::new();
    for animation in animations {
        mixer.action(animation);
    }

    // Create a camera with which to render the scene, and control it with the built-in
    // orbit controller, set to orbit the model.
    let camera = window.factory.perspective_camera(45.0, 0.1 .. 100.0);
    let mut controls = three::controls::Orbit::builder(&camera)
        .position([0.0, 3.0, -1.0])
        .target([0.0, 0.0, -1.0])
        .up([0.0, 0.0, -1.0])
        .build();

    // Here we provide default values for our GUI state.
    let mut state = OptionsGuiState {
        model_scale: 1.0,
        light: true,
        anim_speed: 1.0,
        anim_time: 0.0,
    };

    // Run the main loop, updating the camera controller, animations, and rendering the scene
    // every frame.
    while window.update() && !window.input.hit(three::KEY_ESCAPE) {
        let ctx = &mut window.gui.ctx;
        let media = &mut window.gui.media;
        let font_atlas = &mut window.gui.font_atlas;

        // how many seconds of the animation should go by this frame
        let animation_elapsed = window.input.delta_time() * state.anim_speed;

        ctx.style_set_font(font_atlas.font(media.title_font).unwrap().handle());
        if ctx.begin(
            nk_string!("Manipulate Model"),
            Rect {
                x: 600.0,
                y: 350.0,
                w: 275.0,
                h: 280.0
            },
            PanelFlags::Border as Flags
                | PanelFlags::Movable as Flags
                | PanelFlags::Title as Flags
                //| PanelFlags::NoScrollbar as Flags,
        ) {
            // Misc. Model Rendering UI!
            ctx.layout_row_dynamic(30.0, 2);
            ctx.style_set_font(font_atlas.font(media.body_font).unwrap().handle());
            
            // Model Scale Slider
            ctx.text("Model Scale: ", TextAlignment::Right as Flags);
            if ctx.slider_float(0.0, &mut state.model_scale, 2.0, 0.005) {
                // gotta max it so that three-rs doesn't crash because scale is 0
                instance.set_scale(state.model_scale.max(0.01));
            }
            
            // Light Switch :D
            ctx.text("Light: ", TextAlignment::Right as Flags);
            if ctx.checkbox_text("", &mut state.light) {
                use three::light::Light;
                light.set_intensity(if state.light { LIGHT_INTENSITY } else { 0.0 });
            }
            

            // Animation Heading!
            // a different row styling is used here because only one column will occupy this row. 
            ctx.layout_row_dynamic(30.0, 1);
            ctx.style_set_font(font_atlas.font(media.title_font).unwrap().handle());
            ctx.text("Animation", TextAlignment::Left as Flags);


            // Animation UI!
            ctx.layout_row_dynamic(30.0, 2);
            ctx.style_set_font(font_atlas.font(media.body_font).unwrap().handle());

            // Animation Speed Slider
            ctx.text("Speed: ", TextAlignment::Right as Flags);
            ctx.slider_float(0.0, &mut state.anim_speed, 2.0, 0.005);
            // update progress bar value
            state.anim_time = if state.anim_time > total_anim_time {
                // reset if complete
                0.0
            } else {
                // otherwise increment by same value fed to mixer.update()
                state.anim_time + animation_elapsed
            };

            // Animation Progress Bar
            let mut anim_prog = (state.anim_time/total_anim_time * 100.0).round() as usize;
            ctx.text("Progress: ", TextAlignment::Right as Flags);
            ctx.progress(&mut anim_prog, 100, false);


            // Random Avacado Image
            // I wanted to show loading and using an image, 
            // but couldn't think of a more sensible way to use one :D
            ctx.layout_row_dynamic(256.0, 1);
            ctx.image(media.avocado.clone());
        }
        ctx.end();
        mixer.update(animation_elapsed);
        controls.update(&window.input);
        window.render(&camera);
        window.gui.ctx.clear();
    }
}
