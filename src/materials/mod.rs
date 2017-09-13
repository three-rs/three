use render::BasicPipelineState;
use scene::Color;
use texture::Texture;
pub use builder;

/// Material is the enhancement of Texture that is used to setup appearance of [`Mesh`](struct.Mesh.html).
#[allow(missing_docs)]
#[derive(Clone, Debug)]
pub enum Material {
    /// Basic wireframe with specific `Color`.
    LineBasic { color: Color },
    /// Basic material with color, optional `Texture` and optional wireframe mode.
    MeshBasic {
        color: Color,
        map: Option<Texture<[f32; 4]>>,
        wireframe: bool,
    },
    /// Lambertian diffuse reflection. This technique causes all closed polygons
    /// (such as a triangle within a 3D mesh) to reflect light equally in all
    /// directions when rendered.
    MeshLambert { color: Color, flat: bool },
    /// Material that uses Phong reflection model.
    MeshPhong { color: Color, glossiness: f32 },
    /// Physically-based rendering material.
    MeshPbr {
        base_color_factor: [f32; 4],
        metallic_roughness: [f32; 2],
        occlusion_strength: f32,
        emissive_factor: [f32; 3],
        normal_scale: f32,
        base_color_map: Option<Texture<[f32; 4]>>,
        normal_map: Option<Texture<[f32; 4]>>,
        emissive_map: Option<Texture<[f32; 4]>>,
        metallic_roughness_map: Option<Texture<[f32; 4]>>,
        occlusion_map: Option<Texture<[f32; 4]>>,
    },
    /// 2D Sprite.
    Sprite { map: Texture<[f32; 4]> },
    /// Custom material.
    CustomBasicPipeline {
        color: Color,
        map: Option<Texture<[f32; 4]>>,
        pipeline: BasicPipelineState,
    },
}

impl Material {

    pub fn line_basic(color:Color) -> Material{
        Material::LineBasic{color}
    }

    pub fn mesh_basic() -> MeshBasicBuilder{
        MeshBasicBuilder::new()
    }

    pub fn mesh_lambert() -> MeshLambertBuilder{
        MeshLambertBuilder::new()
    }

    pub fn mesh_phong() -> MeshPhongBuilder{
        MeshPhongBuilder::new()
    }

    pub fn mesh_pbr() -> MeshPbrBuilder{
        MeshPbrBuilder::new()
     }
}