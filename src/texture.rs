use gfx::handle as h;
use render::BackendResources;
use std::path::Path;
use util;

use mint;

pub use gfx::texture::{FilterMethod, WrapMode};

/// The sampling properties for a `Texture`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Sampler(pub h::Sampler<BackendResources>);

/// An image applied (mapped) to the surface of a shape or polygon.
#[derive(Derivative)]
#[derivative(Clone, Debug, PartialEq, Eq(bound = "T: PartialEq"), Hash(bound = ""))]
pub struct Texture<T> {
    view: h::ShaderResourceView<BackendResources, T>,
    sampler: h::Sampler<BackendResources>,
    total_size: [u32; 2],
    #[derivative(Hash(hash_with = "util::hash_f32_slice"))]
    tex0: [f32; 2],
    #[derivative(Hash(hash_with = "util::hash_f32_slice"))]
    tex1: [f32; 2],
}

impl<T> Texture<T> {
    pub(crate) fn new(
        view: h::ShaderResourceView<BackendResources, T>,
        sampler: h::Sampler<BackendResources>,
        total_size: [u32; 2],
    ) -> Self {
        Texture {
            view,
            sampler,
            total_size,
            tex0: [0.0; 2],
            tex1: [
                total_size[0] as f32,
                total_size[1] as f32,
            ],
        }
    }

    pub(crate) fn to_param(
        &self,
    ) -> (
        h::ShaderResourceView<BackendResources, T>,
        h::Sampler<BackendResources>,
    ) {
        (self.view.clone(), self.sampler.clone())
    }

    /// See [`Sprite::set_texel_range`](struct.Sprite.html#method.set_texel_range).
    pub fn set_texel_range(
        &mut self,
        base: mint::Point2<i16>,
        size: mint::Vector2<u16>,
    ) {
        self.tex0 = [
            base.x as f32,
            self.total_size[1] as f32 - base.y as f32 - size.y as f32,
        ];
        self.tex1 = [
            base.x as f32 + size.x as f32,
            self.total_size[1] as f32 - base.y as f32,
        ];
    }

    /// Returns normalized UV rectangle (x0, y0, x1, y1) of the current texel range.
    pub fn uv_range(&self) -> [f32; 4] {
        [
            self.tex0[0] / self.total_size[0] as f32,
            self.tex0[1] / self.total_size[1] as f32,
            self.tex1[0] / self.total_size[0] as f32,
            self.tex1[1] / self.total_size[1] as f32,
        ]
    }
}

/// Represents paths to cube map texture, useful for loading
/// [`CubeMap`](struct.CubeMap.html).
#[derive(Clone, Debug)]
pub struct CubeMapPath<P: AsRef<Path>> {
    /// "Front" image. `Z+`.
    pub front: P,
    /// "Back" image. `Z-`.
    pub back: P,
    /// "Left" image. `X-`.
    pub left: P,
    /// "Right" image. `X+`.
    pub right: P,
    /// "Up" image. `Y+`.
    pub up: P,
    /// "Down" image. `Y-`.
    pub down: P,
}

impl<P: AsRef<Path>> CubeMapPath<P> {
    pub(crate) fn as_array(&self) -> [&P; 6] {
        [
            &self.right,
            &self.left,
            &self.up,
            &self.down,
            &self.front,
            &self.back,
        ]
    }
}

/// Cubemap is six textures useful for
/// [`Cubemapping`](https://en.wikipedia.org/wiki/Cube_mapping).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CubeMap<T> {
    view: h::ShaderResourceView<BackendResources, T>,
    sampler: h::Sampler<BackendResources>,
}

impl<T> CubeMap<T> {
    pub(crate) fn new(
        view: h::ShaderResourceView<BackendResources, T>,
        sampler: h::Sampler<BackendResources>,
    ) -> Self {
        CubeMap { view, sampler }
    }

    pub(crate) fn to_param(
        &self,
    ) -> (
        h::ShaderResourceView<BackendResources, T>,
        h::Sampler<BackendResources>,
    ) {
        (self.view.clone(), self.sampler.clone())
    }
}
