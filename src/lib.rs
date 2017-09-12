#![warn(missing_docs)]
//! Three.js inspired 3D engine in Rust.

#[macro_use]
extern crate bitflags;
extern crate cgmath;
extern crate froggy;
extern crate genmesh;
#[macro_use]
extern crate gfx;
extern crate gfx_glyph;
extern crate gltf;
extern crate gltf_importer;
extern crate gltf_utils;
extern crate image;
#[macro_use]
extern crate itertools;
#[macro_use]
extern crate log;
extern crate mint;
extern crate obj;
extern crate rodio;
extern crate vec_map;
// OpenGL

#[cfg(feature = "opengl")]
extern crate gfx_device_gl;
#[cfg(feature = "opengl")]
extern crate gfx_window_glutin;
#[cfg(feature = "opengl")]
extern crate glutin;

#[macro_use]
mod macros;
pub mod audio;
pub mod camera;
pub mod controls;
pub mod custom;
mod factory;
pub mod geometry;
mod hub;
mod input;
pub mod light;
mod material;
mod mesh;
mod node;
mod object;
mod render;
mod scene;
mod sprite;
mod text;
mod texture;
#[cfg(feature = "opengl")]
pub mod window;

pub use controls::{AXIS_DOWN_UP, AXIS_LEFT_RIGHT, KEY_ESCAPE, KEY_SPACE, MOUSE_LEFT, MOUSE_RIGHT};
pub use controls::{Button, Input, KeyAxis, Timer};
pub use factory::Factory;
pub use geometry::Geometry;
#[cfg(feature = "opengl")]
pub use glutin::VirtualKeyCode as Key;
pub use material::Material;
pub use mesh::{DynamicMesh, Mesh};
pub use node::{NodeInfo, NodePointer, NodeTransform};
pub use object::{Group, Object};
pub use render::{ColorFormat, DebugQuadHandle, DepthFormat, Renderer, ShadowType};
pub use scene::{Background, Color, Scene};
pub use sprite::Sprite;
pub use text::{Align, Font, Layout, Text};
pub use texture::{CubeMap, CubeMapPath, FilterMethod, Sampler, Texture, WrapMode};
#[cfg(feature = "opengl")]
pub use window::Window;

use audio::Source;
use light::{Ambient, Directional, Hemisphere, Point};
three_object_wrapper!(Group, Mesh, DynamicMesh, Source, Sprite, Text);
three_object_wrapper!(Ambient, Hemisphere, Point, Directional);
