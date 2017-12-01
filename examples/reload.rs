extern crate notify;
extern crate three;

use std::{env, fs, io};
use std::sync::mpsc;

use notify::Watcher;
use std::path::{Path, PathBuf};
use std::time::Duration;
use three::Object;

const MANDELBROT_VERTEX_SHADER_CODE: &'static str = r#"
#version 150 core
#include <locals>
#include <globals>

in vec4 a_Position;
in vec2 a_TexCoord;

out vec2 v_TexCoord;

void main() {
    v_TexCoord = mix(u_UvRange.xy, u_UvRange.zw, a_TexCoord);
    gl_Position = u_ViewProj * u_World * a_Position;
}
"#;

const MANDELBROT_PIXEL_SHADER_CODE: &'static str = r#"
#version 150 core

in vec2 v_TexCoord;
out vec4 Target0;

uniform sampler2D t_Map;

const float SCALE = 3.0;
const vec2 CENTER = vec2(0.5, 0.0);
const int ITER = 100;

void main() {
    vec2 c;
    c.x = 1.3333 * (v_TexCoord.x - 0.5) * SCALE - CENTER.x;
    c.y = (v_TexCoord.y - 0.5) * SCALE - CENTER.y;

    int i;
    vec2 z = c;
    for (i = 0; i < ITER; ++i) {
        float x = (z.x * z.x - z.y * z.y) + c.x;
        float y = (z.y * z.x + z.x * z.y) + c.y;
        if ((x * x + y * y) > 4.0) {
            break;
        }
        z.x = x;
        z.y = y;
    }

    vec2 t = vec2(i == ITER ? 0.0 : float(i) / 100.0, 0.5);
    Target0 = texture(t_Map, t);
}
"#;

fn main() {
    let dir = env::args()
        .nth(1)
        .map(PathBuf::from)
        .or(env::current_dir().ok())
        .unwrap();

    use io::Write;
    let _ = fs::create_dir_all(&dir);
    fs::File::create(dir.join("sprite_vs.glsl"))
        .unwrap()
        .write_all(MANDELBROT_VERTEX_SHADER_CODE.as_bytes())
        .unwrap();
    fs::File::create(dir.join("sprite_ps.glsl"))
        .unwrap()
        .write_all(MANDELBROT_PIXEL_SHADER_CODE.as_bytes())
        .unwrap();

    println!("Edit sprite_vs.glsl or sprite_ps.glsl and review.");

    let mut win = three::Window::new("Three-rs shader reloading example");
    let cam = win.factory
        .orthographic_camera([0.0, 0.0], 1.0, -1.0 .. 1.0);

    let (tx, rx) = mpsc::channel();
    let mut watcher = notify::watcher(tx, Duration::from_secs(1)).unwrap();
    watcher
        .watch(&dir, notify::RecursiveMode::NonRecursive)
        .unwrap();

    let map_path = concat!(env!("CARGO_MANIFEST_DIR"), "/test_data/gradient.png");
    let map = win.factory.load_texture(map_path);
    let material = three::material::Sprite { map };
    let mut sprite = win.factory.sprite(material);
    sprite.set_scale(1.0);
    sprite.set_parent(&win.scene);

    let mut reload = true;
    while win.update() && !win.input.hit(three::KEY_ESCAPE) {
        while let Ok(event) = rx.try_recv() {
            use notify::DebouncedEvent::{Create, Write};
            match event {
                Create(_) | Write(_) => reload = true,
                _ => {}
            }
        }
        if reload {
            reload = false;
            let source_set = three::render::source::Set {
                sprite: three::render::source::Sprite::user(&dir).unwrap(),
                ..Default::default()
            };
            match three::render::PipelineStates::new(&source_set, &mut win.factory) {
                Ok(pipeline_states) => win.renderer.reload(pipeline_states),
                Err(err) => println!("{:#?}", err),
            }
        }
        win.render(&cam);
    }
}

/// Reads the entire contents of a file into a `String`.
pub fn read_file_to_string(path: &Path) -> io::Result<String> {
    use self::io::Read;
    let file = fs::File::open(path)?;
    let len = file.metadata()?.len() as usize;
    let mut contents = String::with_capacity(len);
    let _ = io::BufReader::new(file).read_to_string(&mut contents)?;
    Ok(contents)
}
