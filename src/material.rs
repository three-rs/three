use render::BasicPipelineState;
use scene::Color;
use texture::Texture;

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




#[derive(Debug)]
pub struct LineBasicBuilder {
    color: u32
}

impl LineBasicBuilder {
    pub fn new() -> LineBasicBuilder {
        LineBasicBuilder{
            color: 0
        }
    }
    pub fn color(&mut self, color: u32) -> &mut LineBasicBuilder{
        self.color = color;
        self
    }

    pub fn build(&self) -> Material {
        Material::LineBasic{color: self.color}
    }

}

#[derive(Debug)]

pub struct MeshBasicBuilder {
    color: u32,
    map: Option<Texture<[f32;4]>>,
    wireframe: bool,
}


impl MeshBasicBuilder {
    pub fn new() -> MeshBasicBuilder{
        MeshBasicBuilder{
            color: 0,
            map: None,
            wireframe: false
        }
    }

    pub fn color(&mut self, color: u32) -> &mut MeshBasicBuilder{
        self.color = color;
        self
    }

    pub fn map(&mut self, map: Texture<[f32; 4]>) -> &mut MeshBasicBuilder{
        self.map = Some(map);
        self
    }

    pub fn wireframe(&mut self, wireframe: bool) -> &mut MeshBasicBuilder{
        self.wireframe = wireframe;
        self
    }

    pub fn build(&self) -> Material {
        Material::MeshBasic{
            color: self.color,
            map: self.map.clone(),
            wireframe: self.wireframe
        }
    }
}

#[derive(Debug)]
pub struct MeshLambertBuilder {
    color: u32,
    flat: bool,
}

impl  MeshLambertBuilder {
    pub fn new() -> MeshLambertBuilder {
        MeshLambertBuilder{
            color: 0,
            flat: false
        } 
    }

    pub fn color(&mut self, color: u32) -> &mut MeshLambertBuilder{
        self.color = color;
        self
    }

    pub fn flat(&mut self, flat: bool) -> &mut MeshLambertBuilder{
        self.flat = flat;
        self
    }

    pub fn build(&self) -> Material{
        Material::MeshLambert{
            color: self.color,
            flat: self.flat
        }
    }   

}

#[derive(Debug)]
pub struct MeshPhongBuilder {
    color: u32,
    glossiness: f32,
}

impl MeshPhongBuilder {
    pub fn new() -> MeshPhongBuilder{
        MeshPhongBuilder{
            color:0,
            glossiness: 0.0
        }
    }

    pub fn color(&mut self, color: u32) -> &mut MeshPhongBuilder{
        self.color = color;
        self
    }    

    pub fn glossiness(&mut self, glossiness: f32) -> &mut MeshPhongBuilder{
        self.glossiness = glossiness;
        self
    }

    pub fn build(&self) -> Material{
        Material::MeshPhong{
            color: self.color,
            glossiness: self.glossiness
        }
    }
}

#[derive(Debug)]
pub struct MeshPbrBuilder {
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
}

impl MeshPbrBuilder{
     pub fn new() -> MeshPbrBuilder {
         MeshPbrBuilder{
            base_color_factor: [0.0;4],
            metallic_roughness: [0.0; 2],
            occlusion_strength: 0.0,
            emissive_factor: [0.0;3],
            normal_scale: 0.0,
            base_color_map: None,
            normal_map: None,
            emissive_map: None,
            metallic_roughness_map: None,
            occlusion_map: None
         }
     }

     pub fn base_color_factor(&mut self, base_color_factor: [f32; 4]) -> &mut MeshPbrBuilder{
        self.base_color_factor = base_color_factor;
        self
     }

     pub fn metallic_roughness(&mut self, metallic_roughness: [f32;2]) -> &mut MeshPbrBuilder{
        self.metallic_roughness = metallic_roughness;
        self
     }  

     pub fn occlusion_strength(&mut self, occlusion_strength: f32) -> &mut MeshPbrBuilder{
        self.occlusion_strength = occlusion_strength;
        self
     }

     pub fn emissive_factor(&mut self, emissive_factor: [f32;3]) -> &mut MeshPbrBuilder{
        self.emissive_factor = emissive_factor;
        self
     }

     pub fn normal_scale(&mut self, normal_scale: f32) -> &mut MeshPbrBuilder{
        self.normal_scale = normal_scale;
        self
     }

     pub fn base_color_map(&mut self, base_color_map: Texture<[f32;4]>) -> &mut MeshPbrBuilder{
        self.base_color_map = Some(base_color_map);
        self
     }

     pub fn normal_map(&mut self, normal_map: Texture<[f32;4]>) -> &mut MeshPbrBuilder{
        self.normal_map = Some(normal_map);
        self
     }

     pub fn emissive_map(&mut self, emissive_map: Texture<[f32;4]>) -> &mut MeshPbrBuilder{
        self.emissive_map = Some(emissive_map);
        self
     }

     pub fn metallic_roughness_map(&mut self, metallic_roughness_map: Texture<[f32;4]>) -> &mut MeshPbrBuilder{
        self.metallic_roughness_map = Some(metallic_roughness_map);
        self
     }

     pub fn occlusion_map(&mut self, occlusion_map: Texture<[f32;4]>) -> &mut MeshPbrBuilder{
        self.occlusion_map = Some(occlusion_map);
        self
     }

     pub fn build(&self) -> Material {
       Material:: MeshPbr{
        base_color_factor: self.base_color_factor,
        metallic_roughness: self.metallic_roughness,
        occlusion_strength: self.occlusion_strength,
        emissive_factor: self.emissive_factor,
        normal_scale: self.normal_scale,
        base_color_map: self.base_color_map.clone(),
        normal_map: self.normal_map.clone(),
        emissive_map: self.emissive_map.clone(),
        metallic_roughness_map: self.metallic_roughness_map.clone(),
        occlusion_map: self.occlusion_map.clone()
       }
     }
     
}

