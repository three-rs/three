use cgmath::{Quaternion, Rad, Rotation3};
use mint;
use three::{self, Object};

use {COLOR_BROWN, COLOR_BROWN_DARK, COLOR_RED, COLOR_WHITE};

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
        let group = factory.group();

        let cockpit = {
            let mut geo = three::Geometry::cuboid(80.0, 50.0, 50.0);
            for v in geo.base.vertices.iter_mut() {
                if v.x < 0.0 {
                    v.z += if v.y > 0.0 { -20.0 } else { 20.0 };
                    v.y += if v.y > 0.0 { -10.0 } else { 30.0 };
                }
            }
            factory.mesh(geo, three::material::Lambert { color: COLOR_RED, flat: false })
        };
        group.add(&cockpit);

        let engine = factory.mesh(three::Geometry::cuboid(20.0, 50.0, 50.0), three::material::Lambert { color: COLOR_WHITE, flat: false });
        engine.set_position([40.0, 0.0, 0.0]);
        group.add(&engine);

        let tail = factory.mesh(three::Geometry::cuboid(15.0, 20.0, 5.0), three::material::Lambert { color: COLOR_RED, flat: false });
        tail.set_position([-35.0, 25.0, 0.0]);
        group.add(&tail);

        let wing = factory.mesh(three::Geometry::cuboid(40.0, 8.0, 150.0), three::material::Lambert { color: COLOR_RED, flat: false });
        group.add(&wing);

        let propeller_group = factory.group();
        propeller_group.set_position([50.0, 0.0, 0.0]);
        group.add(&propeller_group);
        let propeller = factory.mesh(three::Geometry::cuboid(20.0, 10.0, 10.0), three::material::Lambert { color: COLOR_BROWN, flat: false });
        propeller_group.add(&propeller);
        let blade = factory.mesh(three::Geometry::cuboid(1.0, 100.0, 20.0), three::material::Lambert { color: COLOR_BROWN_DARK, flat: false });
        blade.set_position([8.0, 0.0, 0.0]);
        propeller_group.add(&blade);

        AirPlane { group, _cockpit: cockpit, _engine: engine, _tail: tail, _wing: wing, propeller_group, _propeller: propeller, _blade: blade }
    }

    pub fn update(
        &mut self,
        time: f32,
        target: mint::Point2<f32>,
    ) {
        let q = Quaternion::from_angle_x(Rad(0.3 * time));
        self.propeller_group.set_orientation(q);
        self.group.set_position([0.0 + target.x * 100.0, 100.0 + target.y * 75.0, 0.0]);
    }
}
