use gfx::handle as h;
use render::BackendResources;

use mint;

pub use gfx::texture::{FilterMethod, WrapMode};

/// The sampling properties for a `Texture`.
#[derive(Clone, Debug)]
pub struct Sampler(pub h::Sampler<BackendResources>);

/// An image applied (mapped) to the surface of a shape or polygon.
#[derive(Clone, Debug)]
pub struct Texture<T> {
    view: h::ShaderResourceView<BackendResources, T>,
    sampler: h::Sampler<BackendResources>,
    total_size: [u32; 2],
    tex0: [f32; 2],
    tex1: [f32; 2],
}

impl<T> Texture<T> {
    pub(crate) fn new(view: h::ShaderResourceView<BackendResources, T>,
               sampler: h::Sampler<BackendResources>,
               total_size: [u32; 2]) -> Self {
        Texture {
            view,
            sampler,
            total_size,
            tex0: [0.0; 2],
            tex1: [total_size[0] as f32, total_size[1] as f32],
        }
    }

    pub(crate) fn to_param(&self) -> (h::ShaderResourceView<BackendResources, T>, h::Sampler<BackendResources>) {
        (self.view.clone(), self.sampler.clone())
    }

    /// See [`Sprite::set_texel_range`](struct.Sprite.html#method.set_texel_range).
    pub fn set_texel_range(&mut self, base: mint::Point2<i16>, size: mint::Vector2<u16>) {
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
        [self.tex0[0] / self.total_size[0] as f32,
            self.tex0[1] / self.total_size[1] as f32,
            self.tex1[0] / self.total_size[0] as f32,
            self.tex1[1] / self.total_size[1] as f32]
    }
}
