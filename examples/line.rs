extern crate three;

fn main() {
    let cam = three::OrthographicCamera::new(-20.0, 20.0, 20.0, -10.0, -10.0, 10.0);
    let mut win = three::Window::new("Three-rs line drawing example", cam);

    let geometry = three::Geometry::from_vertices(vec![
        three::Position::new(-10.0, 0.0, 0.0),
        three::Position::new(0.0, 10.0, 0.0),
        three::Position::new(10.0, 0.0, 0.0),
    ]);
    let material = three::Material::LineBasic { color: 0x0000ff };
    let mut line = win.factory.mesh(geometry, material);
    line.attach(&mut win.scene, None);

    while let Some(_events) = win.update() {
        win.render();
    }
}
