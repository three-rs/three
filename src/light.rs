//! Contains different types of light sources.

use gfx;
use object::{Base, Object, ObjectType};
use std::ops;

use camera::Orthographic;
use color::Color;
use hub::{self, Operation, SubLight, SubNode};
use render::{BackendResources, ShadowFormat};
use scene::SyncGuard;

#[derive(Debug)]
pub(crate) enum LightOperation {
    Color(Color),
    Intensity(f32),
}

/// Marks light sources and implements their common methods.
pub trait Light: Object {
    /// Change light color.
    fn set_color(&self, color: Color) {
        let msg = Operation::SetLight(LightOperation::Color(color));
        let _ = self.as_ref().tx.send((self.as_ref().node.downgrade(), msg));
    }

    /// Change light intensity.
    fn set_intensity(&self, intensity: f32) {
        let msg = Operation::SetLight(LightOperation::Intensity(intensity));
        let _ = self.as_ref().tx.send((self.as_ref().node.downgrade(), msg));
    }
}

impl Light for Ambient {}
impl Light for Directional {}
impl Light for Hemisphere {}
impl Light for Point {}

/// `ShadowMap` is used to render shadows from [`PointLight`](struct.PointLight.html)
/// and [`DirectionalLight`](struct.DirectionalLight.html).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ShadowMap {
    pub(crate) resource: gfx::handle::ShaderResourceView<BackendResources, f32>,
    pub(crate) target: gfx::handle::DepthStencilView<BackendResources, ShadowFormat>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ShadowProjection {
    Orthographic(Orthographic),
}

impl ShadowMap {
    pub(crate) fn to_target(
        &self,
    ) -> gfx::handle::DepthStencilView<BackendResources, ShadowFormat> {
        self.target.clone()
    }

    pub(crate) fn to_resource(&self) -> gfx::handle::ShaderResourceView<BackendResources, f32> {
        self.resource.clone()
    }
}

/// Omni-directional, fixed-intensity and fixed-color light source that affects
/// all objects in the scene equally.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Ambient {
    pub(crate) object: Base,
}

impl Ambient {
    pub(crate) fn new(object: Base) -> Self {
        Ambient { object }
    }
}

impl AsRef<Base> for Ambient {
    fn as_ref(&self) -> &Base {
        &self.object
    }
}

impl Object for Ambient {
    type Data = LightData;

    fn resolve_data(&self, sync_guard: &SyncGuard) -> Self::Data {
        match &sync_guard.hub[self].sub_node {
            SubNode::Light(ref light_data) => light_data.into(),
            sub_node @ _ => panic!("`Ambient` had a bad sub node type: {:?}", sub_node),
        }
    }
}

derive_DowncastObject!(Ambient => ObjectType::AmbientLight);

/// The light source that illuminates all objects equally from a given direction,
/// like an area light of infinite size and infinite distance from the scene;
/// there is shading, but cannot be any distance falloff.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Directional {
    pub(crate) object: Base,
}

impl Directional {
    pub(crate) fn new(object: Base) -> Self {
        Directional { object }
    }

    /// Adds or updates the shadow map for this light source.
    pub fn set_shadow(&mut self, map: ShadowMap, extent_y: f32, range: ops::Range<f32>) {
        let sp = ShadowProjection::Orthographic(Orthographic {
            center: [0.0; 2].into(),
            extent_y,
            range,
        });
        let msg = Operation::SetShadow(map, sp);
        let _ = self.object.tx.send((self.object.node.downgrade(), msg));
    }
}

impl AsRef<Base> for Directional {
    fn as_ref(&self) -> &Base {
        &self.object
    }
}

impl Object for Directional {
    type Data = LightData;

    fn resolve_data(&self, sync_guard: &SyncGuard) -> Self::Data {
        match &sync_guard.hub[self].sub_node {
            SubNode::Light(ref light_data) => light_data.into(),
            sub_node @ _ => panic!("`Directional` had a bad sub node type: {:?}", sub_node),
        }
    }
}

derive_DowncastObject!(Directional => ObjectType::DirectionalLight);

/// `HemisphereLight` uses two different colors in opposite to
/// [`Ambient`](struct.Ambient.html).
///
/// The color of each fragment is determined by direction of normal. If the
/// normal points in the direction of the upper hemisphere, the fragment has
/// color of the "sky". If the direction of the normal is opposite, then fragment
/// takes color of the "ground". In other cases, color is determined as
/// interpolation between colors of upper and lower hemispheres, depending on
/// how much the normal is oriented to the upper and the lower hemisphere.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Hemisphere {
    pub(crate) object: Base,
}

impl Hemisphere {
    pub(crate) fn new(object: Base) -> Self {
        Hemisphere { object }
    }
}

impl AsRef<Base> for Hemisphere {
    fn as_ref(&self) -> &Base {
        &self.object
    }
}

impl Object for Hemisphere {
    type Data = HemisphereLightData;

    fn resolve_data(&self, sync_guard: &SyncGuard) -> Self::Data {
        match &sync_guard.hub[self].sub_node {
            SubNode::Light(ref light_data) => light_data.into(),
            sub_node @ _ => panic!("`Hemisphere` had a bad sub node type: {:?}", sub_node),
        }
    }
}

derive_DowncastObject!(Hemisphere => ObjectType::HemisphereLight);

/// Light originates from a single point, and spreads outward in all directions.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Point {
    pub(crate) object: Base,
}

impl Point {
    pub(crate) fn new(object: Base) -> Self {
        Point { object }
    }
}

impl AsRef<Base> for Point {
    fn as_ref(&self) -> &Base {
        &self.object
    }
}

impl Object for Point {
    type Data = LightData;

    fn resolve_data(&self, sync_guard: &SyncGuard) -> Self::Data {
        match &sync_guard.hub[self].sub_node {
            SubNode::Light(ref light_data) => light_data.into(),
            sub_node @ _ => panic!("`Point` had a bad sub node type: {:?}", sub_node),
        }
    }
}

derive_DowncastObject!(Point => ObjectType::PointLight);

/// Internal data for [`Ambient`], [`Directional`], and [`Point`] lights.
///
/// [`Ambient`]: ./struct.Ambient.html
/// [`Directional`]: ./struct.Directional.html
/// [`Point`]: ./struct.Point.html
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LightData {
    /// The color of the light.
    pub color: Color,

    /// The intensity of the light.
    pub intensity: f32,
}

impl<'a> From<&'a hub::LightData> for LightData {
    fn from(from: &'a hub::LightData) -> Self {
        LightData {
            color: from.color,
            intensity: from.intensity,
        }
    }
}

/// Internal data for [`Hemisphere`] lights.
///
/// [`Hemisphere`]: ./struct.Hemisphere.html
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HemisphereLightData {
    /// The ground color of the light.
    pub ground_color: Color,

    /// The sky color of the light.
    pub sky_color: Color,

    /// The intensity of the light.
    pub intensity: f32,
}

impl<'a> From<&'a hub::LightData> for HemisphereLightData {
    fn from(from: &'a hub::LightData) -> Self {
        let ground_color = match from.sub_light {
            SubLight::Hemisphere { ground } => ground,
            _ => panic!("Bad sub-light for `Hemisphere`: {:?}", from.sub_light),
        };
        HemisphereLightData {
            sky_color: from.color,
            ground_color,
            intensity: from.intensity,
        }
    }
}
