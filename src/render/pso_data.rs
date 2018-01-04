use color;
use gfx::handle as h;
use material::Material;
use render::{BackendResources, PbrParams};
use std::mem;
use texture::Texture;

type MapParam = (
    h::ShaderResourceView<BackendResources, [f32; 4]>,
    h::Sampler<BackendResources>,
);

bitflags! {
    struct PbrFlags: i32 {
        const BASE_COLOR_MAP         = 1 << 0;
        const NORMAL_MAP             = 1 << 1;
        const METALLIC_ROUGHNESS_MAP = 1 << 2;
        const EMISSIVE_MAP           = 1 << 3;
        const OCCLUSION_MAP          = 1 << 4;
    }
}

#[derive(Clone, Debug)]
pub(crate) struct PbrMaps {
    base_color: Option<Texture<[f32; 4]>>,
    normal: Option<Texture<[f32; 4]>>,
    emissive: Option<Texture<[f32; 4]>>,
    metallic_roughness: Option<Texture<[f32; 4]>>,
    occlusion: Option<Texture<[f32; 4]>>,
}

#[derive(Clone, Debug)]
pub(crate) struct PbrMapParams {
    pub(crate) base_color: MapParam,
    pub(crate) normal: MapParam,
    pub(crate) emissive: MapParam,
    pub(crate) metallic_roughness: MapParam,
    pub(crate) occlusion: MapParam,
}

impl PbrMaps {
    pub(crate) fn into_params(
        self,
        map_default: &Texture<[f32; 4]>,
    ) -> PbrMapParams {
        PbrMapParams {
            base_color: self.base_color.as_ref().unwrap_or(map_default).to_param(),
            normal: self.normal.as_ref().unwrap_or(map_default).to_param(),
            emissive: self.emissive.as_ref().unwrap_or(map_default).to_param(),
            metallic_roughness: self.metallic_roughness
                .as_ref()
                .unwrap_or(map_default)
                .to_param(),
            occlusion: self.occlusion.as_ref().unwrap_or(map_default).to_param(),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) enum PsoData {
    Pbr {
        params: PbrParams,
        maps: PbrMaps,
    },
    Basic {
        color: u32,
        param0: f32,
        map: Option<Texture<[f32; 4]>>,
    },
}

impl Material {
    pub(crate) fn to_pso_data(&self) -> PsoData {
        match *self {
            Material::Pbr(ref material) => {
                let mut pbr_flags = PbrFlags::empty();
                if material.base_color_map.is_some() {
                    pbr_flags.insert(PbrFlags::BASE_COLOR_MAP);
                }
                if material.normal_map.is_some() {
                    pbr_flags.insert(PbrFlags::NORMAL_MAP);
                }
                if material.metallic_roughness_map.is_some() {
                    pbr_flags.insert(PbrFlags::METALLIC_ROUGHNESS_MAP);
                }
                if material.emissive_map.is_some() {
                    pbr_flags.insert(PbrFlags::EMISSIVE_MAP);
                }
                if material.occlusion_map.is_some() {
                    pbr_flags.insert(PbrFlags::OCCLUSION_MAP);
                }
                let bcf = color::to_linear_rgb(material.base_color_factor);
                let emf = color::to_linear_rgb(material.emissive_factor);
                let pbr_params = PbrParams {
                    base_color_factor: [bcf[0], bcf[1], bcf[2], material.base_color_alpha],
                    camera: [0.0, 0.0, 1.0],
                    emissive_factor: [emf[0], emf[1], emf[2]],
                    metallic_roughness: [material.metallic_factor, material.roughness_factor],
                    normal_scale: material.normal_scale,
                    occlusion_strength: material.occlusion_strength,
                    pbr_flags: pbr_flags.bits(),
                    _padding0: unsafe { mem::uninitialized() },
                    _padding1: unsafe { mem::uninitialized() },
                };
                PsoData::Pbr {
                    maps: PbrMaps {
                        base_color: material.base_color_map.clone(),
                        normal: material.normal_map.clone(),
                        emissive: material.emissive_map.clone(),
                        metallic_roughness: material.metallic_roughness_map.clone(),
                        occlusion: material.occlusion_map.clone(),
                    },
                    params: pbr_params,
                }
            }
            Material::Basic(ref params) => PsoData::Basic {
                color: params.color,
                map: params.map.clone(),
                param0: 0.0,
            },
            Material::CustomBasic(ref params) => PsoData::Basic {
                color: params.color,
                map: params.map.clone(),
                param0: 0.0,
            },
            Material::Line(ref params) => PsoData::Basic {
                color: params.color,
                map: None,
                param0: 0.0,
            },
            Material::Wireframe(ref params) => PsoData::Basic {
                color: params.color,
                map: None,
                param0: 0.0,
            },
            Material::Lambert(ref params) => PsoData::Basic {
                color: params.color,
                map: None,
                param0: if params.flat { 0.0 } else { 1.0 },
            },
            Material::Phong(ref params) => PsoData::Basic {
                color: params.color,
                map: None,
                param0: params.glossiness,
            },
            Material::Sprite(ref params) => PsoData::Basic {
                color: !0,
                map: Some(params.map.clone()),
                param0: 0.0,
            },
        }
    }
}
