use three;

use {COLOR_RED, COLOR_WHITE, COLOR_BROWN, COLOR_BROWN_DARK};


pub struct AirPlane {
    pub group: three::Group,
    _cockpit: three::Mesh,
    _engine: three::Mesh,
    _tail: three::Mesh,
    _wing: three::Mesh,
    propeller_group: three::Group,
    _propeller: three::Mesh,
    _blade: three::Mesh,
}

impl AirPlane {
    pub fn new(factory: &mut three::Factory) -> Self {
        let mut group = factory.group();

        let cockpit = {
            let mut geo = three::Geometry::new_box(80.0, 50.0, 50.0);
            for v in geo.vertices.iter_mut() {
                if v.x < 0.0 {
                    v.z += if v.y > 0.0 {-20.0} else {20.0};
                    v.y += if v.y > 0.0 {-10.0} else {30.0};
                }
            }
            factory.mesh(geo, three::Material::MeshLambert{ color: COLOR_RED })
        };
        group.add(&cockpit);
        let mut engine = factory.mesh(
            three::Geometry::new_box(20.0, 50.0, 50.0),
            three::Material::MeshLambert{ color: COLOR_WHITE }
        );
        engine.transform_mut().disp.x = 40.0;
        group.add(&engine);
        let mut tail = factory.mesh(
            three::Geometry::new_box(15.0, 20.0, 5.0),
            three::Material::MeshLambert{ color: COLOR_RED }
        );
        tail.transform_mut().disp = three::Vector::new(-35.0, 25.0, 0.0);
        group.add(&tail);
        let wing = factory.mesh(
            three::Geometry::new_box(40.0, 8.0, 150.0),
            three::Material::MeshLambert{ color: COLOR_RED }
        );
        group.add(&wing);

        let mut propeller_group = factory.group();
        propeller_group.transform_mut().disp = three::Vector::new(50.0, 0.0, 0.0);
        group.add(&propeller_group);
        let propeller = factory.mesh(
            three::Geometry::new_box(20.0, 10.0, 10.0),
            three::Material::MeshLambert{ color: COLOR_BROWN }
        );
        propeller_group.add(&propeller);
        let mut blade = factory.mesh(
            three::Geometry::new_box(1.0, 100.0, 20.0),
            three::Material::MeshLambert{ color: COLOR_BROWN_DARK }
        );
        blade.transform_mut().disp = three::Vector::new(8.0, 0.0, 0.0);
        propeller_group.add(&blade);

        AirPlane {
            group,
            _cockpit: cockpit,
            _engine: engine,
            _tail: tail,
            _wing: wing,
            propeller_group,
            _propeller: propeller,
            _blade: blade,
        }
    }

    pub fn update(&mut self, dt: f32, target: (f32, f32)) {
        self.propeller_group.transform_mut().rotate(0.3 * dt, 0.0, 0.0);
        self.group.transform_mut().disp =
            three::Vector::new(0.0 + target.0 * 100.0, 100.0 + target.1 * 75.0, 0.0);
    }
}
