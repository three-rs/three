use gfx;

use {LightObject};
use render::{BackendResources, Renderer};


pub struct ShadowMapViewer {
    view: gfx::handle::ShaderResourceView<BackendResources, f32>,
    pos: [u16; 2],
    size: [u16; 2],
}

impl ShadowMapViewer {
    pub fn new(light: &LightObject, pos: [u16; 2], size: [u16; 2]) -> Self {
        ShadowMapViewer {
            view: light.get_shadow().unwrap().to_resource(),
            pos,
            size,
        }
    }

    pub fn render(&self, renderer: &mut Renderer) {
        renderer.draw_quad(&self.view, 1, self.pos, self.size);
    }
}
