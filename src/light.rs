//! Contains different types of light sources.

use gfx;
use std::ops;

use camera::Orthographic;
use hub::Operation;
use object::Object;
use render::{BackendResources, ShadowFormat};

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
    pub(crate) fn to_target(&self) -> gfx::handle::DepthStencilView<BackendResources, ShadowFormat> {
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
    pub(crate) object: Object,
}
three_object_internal!(Ambient::object);

impl Ambient {
    #[doc(hidden)]
    pub fn new(object: Object) -> Self {
        Ambient { object }
    }
}

/// The light source that illuminates all objects equally from a given direction,
/// like an area light of infinite size and infinite distance from the scene;
/// there is shading, but cannot be any distance falloff.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Directional {
    pub(crate) object: Object,
    pub(crate) shadow: Option<ShadowMap>,
}
three_object_internal!(Directional::object);

impl Directional {
    #[doc(hidden)]
    pub fn new(object: Object) -> Self {
        Directional {
            object,
            shadow: None,
        }
    }

    /// Returns `true` if it has [`ShadowMap`](struct.ShadowMap.html), `false` otherwise.
    pub fn has_shadow(&self) -> bool {
        self.shadow.is_some()
    }

    /// Adds shadow map for this light source.
    pub fn set_shadow(
        &mut self,
        map: ShadowMap,
        extent_y: f32,
        range: ops::Range<f32>,
    ) {
        let sp = ShadowProjection::Orthographic(Orthographic {
            center: [0.0; 2].into(),
            extent_y,
            range,
        });
        self.shadow = Some(map.clone());
        self.object.send(Operation::SetShadow(map, sp));
    }
}

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
    pub(crate) object: Object,
}
three_object_internal!(Hemisphere::object);

impl Hemisphere {
    #[doc(hidden)]
    pub fn new(object: Object) -> Self {
        Hemisphere { object }
    }
}

/// Light originates from a single point, and spreads outward in all directions.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Point {
    pub(crate) object: Object,
}
three_object_internal!(Point::object);

impl Point {
    #[doc(hidden)]
    pub fn new(object: Object) -> Self {
        Point { object }
    }
}
