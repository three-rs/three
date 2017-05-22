extern crate three;

fn main() {
    let cam = three::OrthographicCamera::new(-10.0, 10.0, 10.0, -10.0, -10.0, 10.0);
    let mut win = three::Window::new("Three-rs sprite example", cam);

    let material = three::Material::Sprite {
        map: win.factory.load_texture("data/map/pikachu.gif"),
    };
    let mut sprite = win.factory.sprite(material);
    sprite.transform_mut().scale = 8.0;
    sprite.attach(&mut win.scene, None);

    while let Some(_events) = win.update() {
        win.render();
    }
}
