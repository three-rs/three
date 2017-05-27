/*
    Example code is copied verbatim from:
    https://tympanus.net/codrops/2016/04/26/the-aviator-animating-basic-3d-scene-threejs/
*/

extern crate cgmath;
extern crate rand;
extern crate three;

use std::f32::consts::PI;
use cgmath::prelude::*;

struct Cloud {
    group: three::Group,
    meshes: Vec<three::Mesh>,
}

impl Cloud {
    fn new<R: rand::Rng>(rng: &mut R, factory: &mut three::Factory) -> Self {
        let mut cloud = Cloud {
            group: factory.group(),
            meshes: Vec::new()
        };
        let geo = three::Geometry::new_box(20.0, 20.0, 20.0);
        let material = three::Material::MeshBasic{ color: 0xFFFFFF };
        for i in 0 .. rng.gen_range(3, 6) {
            let mut m = factory.mesh(geo.clone(), material.clone());
            let rot: three::Orientation = rng.gen();
            *m.transform_mut() = three::Transform {
                scale: rng.gen_range(0.1, 1.0),
                rot: rot.normalize(),
                disp: cgmath::vec3(i as f32 * 15.0, rng.next_f32() * 10.0, rng.next_f32() * 10.0),
            };
            cloud.group.add(&m);
            cloud.meshes.push(m);
        }
        cloud
    }
}

struct Sky {
    group: three::Group,
    clouds: Vec<Cloud>,
}

impl Sky {
    fn new<R: rand::Rng>(rng: &mut R, factory: &mut three::Factory) -> Self {
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
            *c.group.transform_mut() = three::Transform {
                scale: rng.gen_range(1.0, 3.0),
                rot: three::Orientation::from_angle_z(angle + cgmath::Rad::turn_div_4()),
                disp: cgmath::vec3(angle.cos() * dist,
                                   angle.sin() * dist,
                                   rng.gen_range(-800.0, -400.0)),
            };
            sky.group.add(&c.group);
            sky.clouds.push(c);
        }
        sky
    }
}

struct AirPlane {
    group: three::Group,
    _cockpit: three::Mesh,
    _engine: three::Mesh,
    _tail: three::Mesh,
    _wing: three::Mesh,
    propeller_group: three::Group,
    _propeller: three::Mesh,
    _blade: three::Mesh,
}

impl AirPlane {
    fn new(factory: &mut three::Factory) -> Self {
        let mut group = factory.group();

        let cockpit = {
            let mut geo = three::Geometry::new_box(80.0, 50.0, 50.0);
            geo.vertices[3] += cgmath::vec3(0.0, -10.0, 20.0);
            geo.vertices[2] += cgmath::vec3(0.0, -10.0,  -20.0);
            geo.vertices[1] += cgmath::vec3(0.0, 30.0, 20.0);
            geo.vertices[0] += cgmath::vec3(0.0, 30.0, -20.0);
            factory.mesh(geo, three::Material::MeshBasic{ color: 0xFF0000 })
        };
        group.add(&cockpit);
        let mut engine = factory.mesh(
            three::Geometry::new_box(20.0, 50.0, 50.0),
            three::Material::MeshBasic{ color: 0xFFFFFF }
        );
        engine.transform_mut().disp.x = 40.0;
        group.add(&engine);
        let mut tail = factory.mesh(
            three::Geometry::new_box(15.0, 20.0, 5.0),
            three::Material::MeshBasic{ color: 0xFF0000 }
        );
        tail.transform_mut().disp = cgmath::vec3(-35.0, 25.0, 0.0);
        group.add(&tail);
        let wing = factory.mesh(
            three::Geometry::new_box(40.0, 8.0, 150.0),
            three::Material::MeshBasic{ color: 0xFF0000 }
        );
        group.add(&wing);

        let mut propeller_group = factory.group();
        propeller_group.transform_mut().disp = cgmath::vec3(50.0, 0.0, 0.0);
        group.add(&propeller_group);
        let propeller = factory.mesh(
            three::Geometry::new_box(20.0, 10.0, 10.0),
            three::Material::MeshBasic{ color: 0xa52a2a }
        );
        propeller_group.add(&propeller);
        let mut blade = factory.mesh(
            three::Geometry::new_box(1.0, 100.0, 20.0),
            three::Material::MeshBasic{ color: 0x23190f }
        );
        blade.transform_mut().disp = cgmath::vec3(8.0, 0.0, 0.0);
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

    fn update(&mut self, dt: f32, target: (f32, f32)) {
        let mut pt = self.propeller_group.transform_mut();
        pt.rot = pt.rot * three::Orientation::from_angle_x(cgmath::Rad(0.3 * dt));
        self.group.transform_mut().disp =
            cgmath::vec3(0.0 + target.0 * 100.0, 100.0 + target.1 * 75.0, 0.0);
    }
}


fn main() {
    let mut rng = rand::thread_rng();
    let mut cam = three::PerspectiveCamera::new(60.0, 0.0, 1.0, 1000.0);
    cam.position = three::Position::new(0.0, 100.0, 200.0);
    let mut win = three::Window::new("Three-rs box mesh drawing example", cam);

    //TODO: win.scene.fog = Some(three::Fog::new(...));
    //TODO: create lights
    //TODO: Phong materials
    //TODO: cast/receive shadows

    let mut sea = {
        let geo = three::Geometry::new_cylinder(600.0, 600.0, 800.0, 40);
        let material = three::Material::MeshBasic{ color: 0x0000FF };
        win.factory.mesh(geo, material)
    };
    *sea.transform_mut() = three::Transform {
        scale: 1.0,
        rot: three::Orientation::from_angle_x(-cgmath::Rad::turn_div_4()),
        disp: cgmath::vec3(0.0, -600.0, 0.0),
    };
    win.scene.add(&sea);

    let mut sky = Sky::new(&mut rng, &mut win.factory);
    sky.group.transform_mut().disp.y = -600.0;
    win.scene.add(&sky.group);

    let mut airplane = AirPlane::new(&mut win.factory);
    *airplane.group.transform_mut() = three::Transform {
        scale: 0.25,
        rot: three::Orientation::one(),
        disp: cgmath::vec3(0.0, 100.0, 0.0),
    };
    win.scene.add(&airplane.group);

    while let Some(events) = win.update() {
        // assume the original velocities are given for 60fps
        let dt = events.time_delta * 60.0;

        airplane.update(dt, events.mouse_pos);

        if let (mut t, 0) = (sea.transform_mut(), 0) {
            t.rot = three::Orientation::from_angle_z(cgmath::Rad(0.005 * dt)) * t.rot;
        }
        if let (mut t, 0) = (sky.group.transform_mut(), 0) {
            t.rot = three::Orientation::from_angle_z(cgmath::Rad(0.01 * dt)) * t.rot;
        }

        win.render();
    }
}
