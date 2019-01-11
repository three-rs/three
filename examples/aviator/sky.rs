use std::f32::consts::PI;

use cgmath;
use cgmath::prelude::*;
use rand::Rng;
use three::{self, Object};

use COLOR_WHITE;


pub struct Sky {
    pub group: three::Group,
}

impl Sky {
    fn make_cloud<R: Rng>(
        rng: &mut R,
        factory: &mut three::Factory,
    ) -> three::Group {
        let group = factory.group();
        let geo = three::Geometry::cuboid(20.0, 20.0, 20.0);
        let material = three::material::Lambert {
            color: COLOR_WHITE,
            flat: true,
        };
        let template = factory.mesh(geo, material.clone());
        for i in 0i32 .. rng.gen_range(3, 6) {
            let m = factory.mesh_instance(&template);
            let rot = cgmath::Quaternion::<f32>::new(rng.gen(), rng.gen(), rng.gen(), rng.gen());
            let q = rot.normalize();
            m.set_transform(
                [
                    i as f32 * 15.0,
                    rng.gen::<f32>() * 10.0,
                    rng.gen::<f32>() * 10.0,
                ],
                q,
                rng.gen_range(0.1, 1.0),
            );
            group.add(&m);
        }
        group
    }

    pub fn new<R: Rng>(
        rng: &mut R,
        factory: &mut three::Factory,
    ) -> Self {
        let group = factory.group();
        let num = 20i32;
        let step_angle = PI * 2.0 / num as f32;
        for i in 0 .. num {
            let cloud = Self::make_cloud(rng, factory);
            let angle = cgmath::Rad(i as f32 * step_angle);
            let dist = rng.gen_range(750.0, 950.0);
            let pos = [
                angle.cos() * dist,
                angle.sin() * dist,
                rng.gen_range(-800.0, -400.0),
            ];
            let q = cgmath::Quaternion::from_angle_z(angle + cgmath::Rad::turn_div_4());
            cloud.set_transform(pos, q, rng.gen_range(1.0, 3.0));
            group.add(&cloud);
        }
        Sky { group }
    }
}
