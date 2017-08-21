use std::f32::consts::PI;

use cgmath;
use cgmath::prelude::*;
use rand::Rng;
use three;

use {COLOR_WHITE};


struct Cloud {
    group: three::Group,
    meshes: Vec<three::Mesh>,
}

impl Cloud {
    fn new<R: Rng>(rng: &mut R, factory: &mut three::Factory) -> Self {
        let mut cloud = Cloud {
            group: factory.group(),
            meshes: Vec::new()
        };
        let geo = three::Geometry::new_box(20.0, 20.0, 20.0);
        let material = three::Material::MeshLambert{ color: COLOR_WHITE, flat: true };
        let template = factory.mesh(geo, material.clone());
        for i in 0 .. rng.gen_range(3, 6) {
            let mut m = factory.mesh_instance(&template, material.clone());
            let rot: cgmath::Quaternion<f32> = rng.gen();
            let q = rot.normalize();
            m.set_transform([i as f32 * 15.0, rng.next_f32() * 10.0, rng.next_f32() * 10.0],
                            q,
                            rng.gen_range(0.1, 1.0));
            cloud.group.add(&m);
            cloud.meshes.push(m);
        }
        cloud
    }
}

pub struct Sky {
    pub group: three::Group,
    clouds: Vec<Cloud>,
}

impl Sky {
    pub fn new<R: Rng>(rng: &mut R, factory: &mut three::Factory) -> Self {
        let mut sky = Sky {
            group: factory.group(),
            clouds: Vec::new(),
        };
        let num = 20i32;
        let step_angle = PI * 2.0 / num as f32;
        for i in 0 .. num {
            let mut c = Cloud::new(rng, factory);
            let angle = cgmath::Rad(i as f32 * step_angle);
            let dist = rng.gen_range(750.0, 950.0);
            let pos = [angle.cos() * dist,
                       angle.sin() * dist,
                       rng.gen_range(-800.0, -400.0)];
            let q = cgmath::Quaternion::from_angle_z(angle + cgmath::Rad::turn_div_4());
            c.group.set_transform(pos,
                                  q,
                                  rng.gen_range(1.0, 3.0));
            sky.group.add(&c.group);
            sky.clouds.push(c);
        }
        sky
    }
}
