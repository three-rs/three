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
pub mod material;
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

pub use controls::{KEY_SPACE, KEY_ESCAPE, MOUSE_LEFT, MOUSE_RIGHT, AXIS_DOWN_UP, AXIS_LEFT_RIGHT};
pub use controls::{Button, Input, Timer, KeyAxis};
pub use factory::Factory;
pub use geometry::Geometry;
pub use render::{ColorFormat, DepthFormat, Renderer, ShadowType, DebugQuadHandle};
pub use scene::{Background, Color, Scene};
pub use material::Material;
pub use mesh::{Mesh, DynamicMesh};
pub use node::{NodePointer, NodeTransform, NodeInfo};
pub use object::{Object, Group};
pub use sprite::Sprite;
pub use text::{Align, Text, Layout, Font};
pub use texture::{Texture, Sampler, WrapMode, FilterMethod};
#[cfg(feature = "opengl")]
pub use window::Window;
#[cfg(feature = "opengl")]
pub use glutin::VirtualKeyCode as Key;

use audio::Source;
use light::{Ambient, Directional, Hemisphere, Point};
three_object_wrapper!(Group, Mesh, DynamicMesh, Source, Sprite, Text);
three_object_wrapper!(Ambient, Hemisphere, Point, Directional);
