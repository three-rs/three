//! Material parameters for mesh rendering.

use color;

use color::Color;
use render::BasicPipelineState;
use texture::Texture;

pub use self::basic::Basic;

use std::sync::atomic::{self, AtomicUsize};
static MATERIAL_ID: AtomicUsize = atomic::ATOMIC_USIZE_INIT;

/// Basic material API.
pub mod basic {
    use super::*;

    /// Parameters for a basic solid mesh material.
    ///
    /// Renders triangle meshes with a solid color or texture.
    #[derive(Clone, Debug, PartialEq)]
    pub struct Basic {
        /// Solid color applied in the absense of `map`.
        ///
        /// Default: `WHITE`.
        pub color: Color,

        /// Texture applied using the mesh texture co-ordinates.
        ///
        /// Default: `None`.
        pub map: Option<Texture<[f32; 4]>>,
    }

    impl Default for Basic {
        fn default() -> Self {
            Self {
                color: color::WHITE,
                map: None,
            }
        }
    }

    /// Parameters for a basic solid mesh material with a custom pipeline.
    ///
    /// Renders triangle meshes with a custom pipeline with a basic material as
    /// its input.
    #[derive(Clone, Debug, PartialEq)]
    pub struct Custom {
        /// Solid color applied in the absense of `map`.
        ///
        /// Default: `WHITE`.
        pub color: Color,

        /// Texture applied using the mesh texture co-ordinates.
        ///
        /// Default: `None`.
        pub map: Option<Texture<[f32; 4]>>,

        /// The custom pipeline state object to be applied to the mesh.
        pub pipeline: BasicPipelineState,
    }
}

/// Parameters for a Lamberian diffusion reflection model.
///
/// Renders triangle meshes with the Gouraud illumination model.
#[derive(Clone, Debug, PartialEq)]
pub struct Lambert {
    /// Solid color applied in the absense of `map`.
    ///
    /// Default: `WHITE`.
    pub color: Color,

    /// Specifies whether lighting should be constant over faces.
    ///
    /// Default: `false` (lighting is interpolated across faces).
    pub flat: bool,
}

impl Default for Lambert {
    fn default() -> Self {
        Self {
            color: color::WHITE,
            flat: false,
        }
    }
}

/// Parameters for a line material.
///
/// Renders line strip meshes with a solid color and unit width.
#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    /// Solid line color.
    ///
    /// Default: `0xFFFFFF` (white).
    pub color: Color,
}

impl Default for Line {
    fn default() -> Self {
        Self {
            color: color::WHITE,
        }
    }
}

/// Parameters for a PBR (physically based rendering) lighting model.
///
/// Renders triangle meshes with a PBR (physically-based rendering)
/// illumination model
#[derive(Clone, Debug, PartialEq)]
pub struct Pbr {
    /// Solid base color applied in the absense of `base_color_map`.
    ///
    /// Default: `WHITE`.
    pub base_color_factor: Color,

    /// Base color alpha factor applied in the absense of `base_color_map`.
    ///
    /// Default: `1.0` (opaque).
    pub base_color_alpha: f32,

    /// Metallic factor in the range [0.0, 1.0].
    ///
    /// Default: `1.0`.
    pub metallic_factor: f32,

    /// Roughness factor in the range [0.0, 1.0].
    ///
    /// * A value of 1.0 means the material is completely rough.
    /// * A value of 0.0 means the material is completely smooth.
    ///
    /// Default: `1.0`.
    pub roughness_factor: f32,

    /// Scalar multiplier in the range [0.0, 1.0] that controls the amount of
    /// occlusion applied in the presense of `occlusion_map`.
    ///
    /// Default: `1.0`.
    pub occlusion_strength: f32,

    /// Solid emissive color applied in the absense of `emissive_map`.
    ///
    /// Default: `BLACK`.
    pub emissive_factor: Color,

    /// Scalar multiplier applied to each normal vector of the `normal_map`.
    ///
    /// This value is ignored in the absense of `normal_map`.
    ///
    /// Default: `1.0`.
    pub normal_scale: f32,

    /// Base color texture.
    ///
    /// Default: `None`.
    pub base_color_map: Option<Texture<[f32; 4]>>,

    /// Normal texture.
    ///
    /// Default: `None`.
    pub normal_map: Option<Texture<[f32; 4]>>,

    /// Emissive texture.
    ///
    /// Default: `None`.
    pub emissive_map: Option<Texture<[f32; 4]>>,

    /// Metallic-roughness texture.
    ///
    /// Default: `None`.
    pub metallic_roughness_map: Option<Texture<[f32; 4]>>,

    /// Occlusion texture.
    ///
    /// Default: `None`.
    pub occlusion_map: Option<Texture<[f32; 4]>>,
}

impl Default for Pbr {
    fn default() -> Self {
        Self {
            base_color_factor: color::WHITE,
            base_color_alpha: 1.0,
            metallic_factor: 1.0,
            roughness_factor: 1.0,
            occlusion_strength: 1.0,
            emissive_factor: color::BLACK,
            normal_scale: 1.0,
            base_color_map: None,
            normal_map: None,
            emissive_map: None,
            metallic_roughness_map: None,
            occlusion_map: None,
        }
    }
}

/// Parameters for a Phong reflection model.
///
/// Renders triangle meshes with the Phong illumination model.
#[derive(Clone, Debug, PartialEq)]
pub struct Phong {
    /// Solid color applied in the absense of `map`.
    ///
    /// Default: `WHITE`.
    pub color: Color,

    /// Determines the sharpness of specular highlights.
    ///
    /// Higher values result in sharper highlights to produce a glossy effect.
    ///
    /// Default: `30.0`.
    pub glossiness: f32,
}

impl Default for Phong {
    fn default() -> Self {
        Self {
            color: color::WHITE,
            glossiness: 30.0,
        }
    }
}

/// Texture for a 2D sprite.
///
/// Renders [`Sprite`] objects with the given texture.
///
/// [`Sprite`]: ../sprite/struct.Sprite.html
#[derive(Clone, Debug, PartialEq)]
pub struct Sprite {
    /// The texture the apply to the sprite.
    pub map: Texture<[f32; 4]>,
}

/// Parameters for mesh wireframe rasterization.
///
/// Renders the edges of a triangle mesh with a solid color.
#[derive(Clone, Debug, PartialEq)]
pub struct Wireframe {
    /// Solid color applied to each wireframe edge.
    ///
    /// Default: `WHITE`.
    pub color: Color,
}

/// Specifies the appearance of a [`Mesh`](struct.Mesh.html).
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum MaterialType {
    Basic(Basic),
    CustomBasic(basic::Custom),
    Line(Line),
    Lambert(Lambert),
    Phong(Phong),
    Pbr(Pbr),
    Sprite(Sprite),
    Wireframe(Wireframe),
}

/// Specifies the appearance of a [`Mesh`](struct.Mesh.html).
#[derive(Debug, Clone, PartialEq)]
pub struct Material {
    pub(crate) mat_type: MaterialType,
    pub(crate) id: usize,
}

macro_rules! from_for_material {
    ($($name:ident),*) => {
        $(
            impl From<$name> for Material {
                fn from(params: $name) -> Material {
                    Material {
                        mat_type: MaterialType::$name(params),
                        id: MATERIAL_ID.fetch_add(1, atomic::Ordering::SeqCst),
                    }
                }
            }
        )*
    };
}

impl From<basic::Custom> for Material {
    fn from(params: basic::Custom) -> Material {
        Material {
            mat_type: MaterialType::CustomBasic(params),
            id: MATERIAL_ID.fetch_add(1, atomic::Ordering::SeqCst),
        }
    }
}

from_for_material!(Basic, Line, Wireframe, Phong, Pbr, Lambert, Sprite);
