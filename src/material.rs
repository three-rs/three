//! Material parameters for mesh rendering.

use render::BasicPipelineState;
use scene::Color;
use texture::Texture;

pub use self::basic::Basic;

/// Basic material parameters.
pub mod basic {
    use super::*;

    /// Parameters for a basic mesh material.
    #[derive(Clone, Debug, Default)]
    pub struct Basic {
        /// Solid color applied in the absense of `map`.
        ///
        /// Default: `0x000000` (black).
        pub color: Color,

        /// Texture applied using the mesh texture co-ordiantes.
        ///
        /// Default: `None`.
        pub map: Option<Texture<[f32; 4]>>,

        /// Specifies which pipeline should be used to render the mesh.
        ///
        /// Default: `Solid`.
        pub pipeline: Pipeline,
    }

    /// Specifies which pipeline should be used to render the mesh.
    #[derive(Clone, Debug)]
    pub enum Pipeline {
        /// Renders the mesh as a solid.
        Solid,

        /// Renders the mesh as a wireframe.
        Wireframe,

        /// Renders the mesh with a custom pipeline state object.
        Custom(BasicPipelineState),
    }

    impl Default for Pipeline {
        fn default() -> Self {
            Pipeline::Solid
        }
    }
}

/// Parameters for a Lamberian diffusion reflection model.
#[derive(Clone, Debug, Default)]
pub struct Lambert {
    /// Solid color applied in the absense of `map`.
    ///
    /// Default: `0x000000` (black).
    pub color: Color,

    /// Specifies whether lighting should be constant over faces.
    ///
    /// Default: `false` (lighting is interpolated across faces).
    pub flat: bool,
}

/// Parameters for a PBR (physically based rendering) lighting model.
#[derive(Clone, Debug)]
pub struct Pbr {
    /// Solid base color applied in the absense of `base_color_map`.
    ///
    /// Default: `0xFFFFFF` (white).
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
    /// Default: `0x000000` (black).
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
            base_color_factor: 0xFFFFFF,
            base_color_alpha: 1.0,
            metallic_factor: 1.0,
            roughness_factor: 1.0,
            occlusion_strength: 1.0,
            emissive_factor: 0x000000,
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
#[derive(Clone, Debug, Default)]
pub struct Phong {
    /// Solid color applied in the absense of `map`.
    ///
    /// Default: `0x000000` (black).
    pub color: Color,

    /// Determines the sharpness of specular highlights.
    ///
    /// Higher values result in sharper highlights to produce a glossy effect.
    ///
    /// Default: `30.0`.
    pub glossiness: f32,
}

/// Texture for a 2D sprite.
#[derive(Clone, Debug)]
pub struct Sprite {
    /// The texture the apply to the sprite.
    pub map: Texture<[f32; 4]>,
}

/// Specifies the appearance of a [`Mesh`](struct.Mesh.html).
#[derive(Clone, Debug)]
pub struct Material(pub(crate) Params);

/// Internal material parameters.
#[derive(Clone, Debug)]
pub(crate) enum Params {
    Basic(Basic),
    Lambert(Lambert),
    Phong(Phong),
    Pbr(Pbr),
    Sprite(Sprite),
}

impl From<Basic> for Material {
    fn from(params: Basic) -> Material {
        Material(Params::Basic(params))
    }
}

impl From<Lambert> for Material {
    fn from(params: Lambert) -> Material {
        Material(Params::Lambert(params))
    }
}

impl From<Phong> for Material {
    fn from(params: Phong) -> Material {
        Material(Params::Phong(params))
    }
}

impl From<Pbr> for Material {
    fn from(params: Pbr) -> Material {
        Material(Params::Pbr(params))
    }
}

impl From<Sprite> for Material {
    fn from(params: Sprite) -> Material {
        Material(Params::Sprite(params))
    }
}
