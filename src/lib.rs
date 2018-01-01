#![warn(missing_docs)]
//! Three.js inspired 3D engine.
//!
//! # Getting Started
//!
//! ## Creating a window
//!
//! Every `three` application begins with a [`Window`]. We create it as follows.
//!
//! ```rust,no_run
//! # extern crate three;
//! # fn main() {
//! let title = "Getting started with three-rs";
//! let mut window = three::Window::new(title);
//! # }
//! ```
//!
//! ## The four key structs
//!
//! Every [`Window`] comes equipped with four structures, namely [`Factory`],
//! [`Renderer`], [`Input`], and [`Scene`].
//!
//! * The [`Factory`] instantiates game objects such as [`Mesh`] and [`Camera`].
//! * The [`Input`] handles window events at a high-level.
//! * The [`Scene`] is the root node of the component-graph system.
//! * The [`Renderer`] renders the [`Scene`] from the view of a [`Camera`] object.
//!
//! ## Creating a basic mesh
//!
//! Renderable 3D objects are represented by the [`Mesh`] struct. A mesh is a
//! combination of [`Geometry`], describing the shape of the object, paired with a
//! [`Material`], describing the appearance of the object.
//!
//! ```rust,no_run
//! # extern crate three;
//! # fn main() {
//! # let title = "Getting started with three-rs";
//! # let mut window = three::Window::new(title);
//! let geometry = three::Geometry::with_vertices(vec![
//!     [-0.5, -0.5, -0.5].into(),
//!     [ 0.5, -0.5, -0.5].into(),
//!     [ 0.0,  0.5, -0.5].into(),
//! ]);
//! let material = three::material::Basic {
//!     color: 0xFFFF00,
//!     .. Default::default()
//! };
//! let mut mesh = window.factory.mesh(geometry, material);
//! # }
//! ```
//!
//! ## Managing the scene
//!
//! In order to be rendered by the [`Renderer`], meshes must be placed in the
//! [`Scene`] within the viewable region. Any marked with the [`Object`] trait
//! may be placed into the scene heirarchy, including user-defined structs.
//!
//! ```rust,no_run
//! # extern crate three;
//! # fn main() {
//! # let title = "Getting started with three-rs";
//! # let mut window = three::Window::new(title);
//! # let vertices = vec![
//! #     [-0.5, -0.5, -0.5].into(),
//! #     [ 0.5, -0.5, -0.5].into(),
//! #     [ 0.0,  0.5, -0.5].into(),
//! # ];
//! # let geometry = three::Geometry::with_vertices(vertices);
//! # let material = three::material::Basic {
//! #      color: 0xFFFF00,
//! #      .. Default::default()
//! # };
//! # let mut mesh = window.factory.mesh(geometry, material);
//! use three::Object;
//! mesh.set_parent(&window.scene);
//! # }
//! ```
//!
//! We can set the initial scene background using the `Scene::background`
//! field. The default background is solid black. Let's set the background to a
//! sky blue color instead.
//!
//! ```rust,no_run
//! # extern crate three;
//! # fn main() {
//! # let title = "Getting started with three-rs";
//! # let mut window = three::Window::new(title);
//! window.scene.background = three::Background::Color(0xC6F0FF);
//! # }
//! ```
//!
//! ## Creating the game loop
//!
//! All is left to do to render our triangle is to create a camera and to
//! write the main game loop.
//!
//! ```rust,no_run
//! # extern crate three;
//! # fn main() {
//! #     let title = "Getting started with three-rs";
//! #     let mut window = three::Window::new(title);
//! #     let vertices = vec![
//! #         [-0.5, -0.5, -0.5].into(),
//! #         [ 0.5, -0.5, -0.5].into(),
//! #         [ 0.0,  0.5, -0.5].into(),
//! #     ];
//! #     let geometry = three::Geometry::with_vertices(vertices);
//! #     let material = three::material::Basic {
//! #         color: 0xFFFF00,
//! #         .. Default::default()
//! #     };
//! #     let mut mesh = window.factory.mesh(geometry, material);
//! #     use three::Object;
//! #     mesh.set_parent(&window.scene);
//! let center = [0.0, 0.0];
//! let yextent = 1.0;
//! let zrange = -1.0 .. 1.0;
//! let camera = window.factory.orthographic_camera(center, yextent, zrange);
//! while window.update() {
//!     window.render(&camera);
//! }
//! # }
//! ```
//!
//! ## Putting it all together
//!
//! You should have the following code which renders a single yellow triangle
//! upon a sky blue background.
//!
//! ```rust,no_run
//! extern crate three;
//!
//! use three::Object;
//!
//! fn main() {
//!     let title = "Getting started with three-rs";
//!     let mut window = three::Window::new(title);
//!
//!     let vertices = vec![
//!         [-0.5, -0.5, -0.5].into(),
//!         [ 0.5, -0.5, -0.5].into(),
//!         [ 0.0,  0.5, -0.5].into(),
//!     ];
//!     let geometry = three::Geometry::with_vertices(vertices);
//!     let material = three::material::Basic {
//!         color: 0xFFFF00,
//!         .. Default::default()
//!     };
//!     let mut mesh = window.factory.mesh(geometry, material);
//!     mesh.set_parent(&window.scene);
//!
//!     let center = [0.0, 0.0];
//!     let yextent = 1.0;
//!     let zrange = -1.0 .. 1.0;
//!     let camera = window.factory.orthographic_camera(center, yextent, zrange);
//!
//!     while window.update() {
//!         window.render(&camera);
//!     }
//! }
//! ```
//!
//! # Highlighted features
//!
//! ## Scene management
//!
//! We use [`froggy`] to manage the scene heirarchy. `three` takes a slightly
//! different approach to regular scene graphs whereby child objects keep their
//! parents alive, opposed to parents keeping their children alive.
//!
//! At the heart of the scene heirarchy is the [`object::Base`] type, which
//! is a member of all `three` objects that are placeable in the scene.
//!
//! All three objects are marked by the [`Object`] trait which provides the
//! library with the [`object::Base`] type. Users may implement this trait in
//! order to add their own structs into the scene heirarchy.
//!
//! ```rust,no_run
//! #[macro_use]
//! extern crate three;
//!
//! use three::Object;
//!
//! struct MyObject {
//!     group: three::Group,
//! }
//!
//! impl AsRef<three::object::Base> for MyObject {
//!     fn as_ref(&self) -> &three::object::Base {
//!         self.group.as_ref()
//!     }
//! }
//!
//! impl AsMut<three::object::Base> for MyObject {
//!     fn as_mut(&mut self) -> &mut three::object::Base {
//!         self.group.as_mut()
//!     }
//! }
//!
//! impl Object for MyObject {}
//!
//! fn main() {
//! #    let mut window = three::Window::new("");
//!     // Initialization code omitted.
//!     let mut my_object = MyObject { group: window.factory.group() };
//!     my_object.set_parent(&window.scene);
//! }
//! ```
//!
//! ## Multiple graphics pipelines
//!
//! Graphics pipeline management is handled implicitly by `three`. The pipeline used
//! to render a [`Mesh`] is chosen by its [`Material`] properties and the available
//! vertex attributes from its [`Geometry`].
//!
//! The range of graphics pipelines available range from simple sprite rendering to
//! physically based rendering.
//!
//! ## 3D format loading
//!
//! ### glTF 2.0
//!
//! `three` comes equipped with support for rendering and animating glTF scenes.
//!
//! See [`Factory::load_gltf`] to get started.
//!
//! ### Wavefront OBJ
//!
//! For less complex meshes, `three` supports loading models in OBJ format.
//!
//! See [`Factory::load_obj`] for more information.
//!
//! ## Procedurally generated geometry
//!
//! The [`Geometry`] struct leverages the [`genmesh`] crate to provide procedurally
//! generated primtives such as cuboids, spheres, and cylinders. See the
//! documentation on the [`Geometry`] struct for more information.
//!
//! [`froggy`]: https://crates.io/crates/froggy
//! [`genmesh`]: https://crates.io/crates/genmesh
//!
//! [`Camera`]: camera/struct.Camera.html
//! [`Factory`]: factory/struct.Factory.html
//! [`Factory::load_gltf`]: factory/struct.Factory.html#method.load_gltf
//! [`Factory::load_obj`]: factory/struct.Factory.html#method.load_obj
//! [`Geometry`]: geometry/struct.Geometry.html
//! [`Input`]: input/struct.Input.html
//! [`Material`]: material/enum.Material.html
//! [`Mesh`]: mesh/struct.Mesh.html
//! [`object::Base`]: object/struct.Base.html
//! [`Object`]: object/trait.Object.html
//! [`Renderer`]: struct.Renderer.html
//! [`Scene`]: scene/struct.Scene.html
//! [`Window`]: window/struct.Window.html

extern crate arrayvec;
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
extern crate includedir;
#[macro_use]
extern crate itertools;
#[macro_use]
extern crate log;
extern crate mint;
extern crate obj;
extern crate phf;
#[macro_use]
extern crate quick_error;
extern crate rodio;
extern crate vec_map;

#[cfg(feature = "opengl")]
extern crate gfx_device_gl;
#[cfg(feature = "opengl")]
extern crate gfx_window_glutin;
#[cfg(feature = "opengl")]
extern crate glutin;

#[macro_use]
mod macros;

pub mod audio;
pub mod animation;
pub mod camera;
pub mod color;
pub mod controls;
pub mod custom;
mod data;
mod factory;
pub mod geometry;
mod group;
mod hub;
mod input;
pub mod light;
pub mod material;
mod mesh;
mod node;
pub mod object;
pub mod render;
pub mod scene;
pub mod skeleton;
mod sprite;
mod text;
mod texture;
mod util;
#[cfg(feature = "opengl")]
pub mod window;

#[doc(inline)]
pub use color::Color;

#[doc(inline)]
pub use controls::{AXIS_DOWN_UP, AXIS_LEFT_RIGHT, KEY_ESCAPE, KEY_SPACE, MOUSE_LEFT, MOUSE_RIGHT};

#[doc(inline)]
pub use controls::{Button, Input, Timer};

#[doc(inline)]
pub use factory::{Factory, Gltf};

#[doc(inline)]
pub use geometry::Geometry;

#[cfg(feature = "opengl")]
#[doc(inline)]
pub use glutin::VirtualKeyCode as Key;

#[doc(inline)]
pub use group::Group;

#[doc(inline)]
pub use material::Material;

#[doc(inline)]
pub use mesh::{DynamicMesh, Mesh};

#[doc(inline)]
pub use node::{Node, Transform};

#[doc(inline)]
pub use object::Object;

#[doc(inline)]
pub use render::Renderer;

#[doc(inline)]
pub use scene::{Background, Scene};

#[doc(inline)]
pub use sprite::Sprite;

#[doc(inline)]
pub use text::{Align, Font, Layout, Text};

#[doc(inline)]
pub use texture::{CubeMap, CubeMapPath, FilterMethod, Sampler, Texture, WrapMode};

#[cfg(feature = "opengl")]
#[doc(inline)]
pub use window::Window;
