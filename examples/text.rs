extern crate three;

fn main() {
    let mut window = three::Window::new("Three-rs text example");

    window.scene.background = three::Background::Color(0x111111);

    let center = [0.0, 0.0];
    let yextent = 1.0;
    let zrange = -1.0 .. 1.0;
    let camera = window.factory.orthographic_camera(center, yextent, zrange);

    let deja_vu = window.factory.load_font(format!(
        "{}/data/fonts/DejaVuSans.ttf",
        env!("CARGO_MANIFEST_DIR")
    ));
    let karla = window.factory.load_font_karla();

    let mut counter_text = window.factory.ui_text(&deja_vu, "");
    counter_text.set_font_size(20.0);
    window.scene.add(&counter_text);

    let mut greeting = window.factory.ui_text(&karla, "Hello World!");
    greeting.set_font_size(80.0);
    greeting.set_pos([100.0, 100.0]);
    greeting.set_color(0xFF0000);
    window.scene.add(&greeting);

    let mut lenny = window.factory.ui_text(&deja_vu, "( ͡° ͜ʖ ͡°)");
    lenny.set_font_size(60.0);
    lenny.set_color(0x2222FF);
    window.scene.add(&lenny);

    let mut counter = 0;
    while window.update() {
        counter_text.set_text(format!("Counter: {}", counter));
        lenny.set_pos([(counter % 300) as f32, 200.0]);
        window.render(&camera);
        counter += 1;
    }
}
