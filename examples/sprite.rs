extern crate three;

struct Animator {
    cell_size: [u16; 2],
    cell_counts: [u16; 2],
    duration: f32,
    repeat: bool,
    sprite: three::Sprite,
    current: [u16; 2],
    remainder: f32,
}

impl Animator {
    fn update(&mut self) {
        let base = [
            (self.current[0] * self.cell_size[0]) as i16,
            (self.current[1] * self.cell_size[1]) as i16,
        ];
        self.sprite.set_texel_range(base, self.cell_size);
    }

    fn start(&mut self, row: u16) {
        self.current = [0, row];
        self.remainder = 0.0;
        self.update();
    }

    fn time(&mut self, delta: f32) {
        self.remainder += delta;
        while self.remainder >= self.duration &&
              (self.repeat || self.current[0] < self.cell_counts[0]) {
            self.remainder -= self.duration;
            self.current[0] += 1;
            if self.current[0] < self.cell_counts[0] {
                self.update();
            } else if self.repeat {
                self.current[0] = 0;
                self.update();
            }
        }
    }
}

fn main() {
    let mut win = three::Window::new("Three-rs sprite example", "data/shaders");
    let cam = win.factory.orthographic_camera(-10.0, 10.0, 10.0, -10.0, -10.0, 10.0);

    let material = three::Material::Sprite {
        map: win.factory.load_texture("test_data/pikachu_anim.png"),
    };
    let mut sprite = win.factory.sprite(material);
    sprite.transform_mut().scale = 8.0;
    win.scene.add(&sprite);

    let mut anim = Animator {
        cell_size: [96, 96],
        cell_counts: [5, 13],
        duration: 0.1,
        repeat: true,
        current: [0, 0],
        remainder: 0.0,
        sprite,
    };
    let mut row = 0u16;
    anim.start(row);

    while let Some(events) = win.update() {
        if events.hit.contains(&three::Key::Left) {
            row = (row + anim.cell_counts[1] - 1) % anim.cell_counts[1];
            anim.start(row);
        }
        if events.hit.contains(&three::Key::Right) {
            row = (row + 1) % anim.cell_counts[1];
            anim.start(row);
        }
        anim.time(events.time_delta);

        win.render(&cam);
    }
}
