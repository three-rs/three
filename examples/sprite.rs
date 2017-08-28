extern crate three;

struct Animator {
    cell_size: [u16; 2],
    cell_counts: [u16; 2],
    duration: f32,
    repeat: bool,
    sprite: three::Sprite,
    current: [u16; 2],
    timer: three::Timer,
}

impl Animator {
    fn update_uv(&mut self) {
        let base = [
            (self.current[0] * self.cell_size[0]) as i16,
            (self.current[1] * self.cell_size[1]) as i16,
        ];
        self.sprite.set_texel_range(base, self.cell_size);
    }

    fn update(&mut self, switch_row: Option<u16>, input: &three::Input) {
        if let Some(row) = switch_row {
            self.current = [0, row];
            self.timer = input.time();
            self.update_uv();
        } else if self.timer.get(input) >= self.duration &&
               (self.repeat || self.current[0] < self.cell_counts[0]) {
            self.timer = input.time();
            self.current[0] += 1;
            if self.current[0] < self.cell_counts[0] {
                self.update_uv();
            } else if self.repeat {
                self.current[0] = 0;
                self.update_uv();
            }
        }
    }
}

fn main() {
    let shaders_path: String = format!("{}/data/shaders", env!("CARGO_MANIFEST_DIR"));
    let shaders_path_str: &str = shaders_path.as_str();
    let mut win = three::Window::builder("Three-rs sprite example", shaders_path_str).build();
    let cam = win.factory.orthographic_camera([0.0, 0.0], 10.0, -10.0, 10.0);

    let pikachu_path: String = format!("{}/test_data/pikachu_anim.png", env!("CARGO_MANIFEST_DIR"));
    let pikachu_path_str: &str = pikachu_path.as_str();
    let material = three::Material::Sprite {
        map: win.factory.load_texture(pikachu_path_str),
    };
    let mut sprite = win.factory.sprite(material);
    sprite.set_scale(8.0);
    win.scene.add(&sprite);

    let mut anim = Animator {
        cell_size: [96, 96],
        cell_counts: [5, 13],
        duration: 0.1,
        repeat: true,
        current: [0, 0],
        timer: win.input.time(),
        sprite,
    };
    anim.update_uv();

    while win.update() && !three::KEY_ESCAPE.is_hit(&win.input) {
        let row = three::AXIS_LEFT_RIGHT.delta_hits(&win.input).map(|mut diff| {
            let total = anim.cell_counts[1] as i8;
            while diff < 0 { diff += total };
            (anim.current[1] + diff as u16) % total as u16
        });
        anim.update(row, &win.input);

        win.render(&cam);
    }
}
