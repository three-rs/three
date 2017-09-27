//! Source for for GLSL shaders used by the renderer.

use data;
use util;

use std::{io, ops, str};
use std::borrow::Borrow;
use std::path::Path;

/// Source code for a single GLSL shader.
#[derive(Clone, Debug)]
pub struct Source(pub(crate) String);

impl ops::Deref for Source {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        self.0.as_bytes()
    }
}

impl Source {
    fn preprocess<P: AsRef<Path>>(
        root: P,
        code: &str,
    ) -> io::Result<String> {
        let root = root.as_ref();
        let mut new_code = String::new();
        for line in code.lines() {
            if line.starts_with("#include") {
                for dep_name in line.split(' ').skip(1) {
                    match dep_name {
                        "locals" | "lights" | "globals" => {
                            let path = format!("data/shaders/{}.glsl", dep_name);
                            let content = &data::FILES.get(&path).unwrap();
                            new_code += str::from_utf8(content.borrow()).unwrap();
                        }
                        relative_path => {
                            let path = root.join(relative_path);
                            let content = util::read_file_to_string(&path)?;
                            let include = Self::preprocess(root, &content)?;
                            new_code += &include;
                        }
                    }
                }
            } else {
                new_code.push_str(&line);
                new_code.push('\n');
            }
        }
        Ok(new_code)
    }

    /// Load the named shader from the default set of shaders.
    pub fn default(
        name: &str,
        suffix: &str,
    ) -> io::Result<Self> {
        let path = format!("data/shaders/{}_{}.glsl", name, suffix);
        let unprocessed = data::FILES.get(&path).unwrap();
        let processed = Self::preprocess("", str::from_utf8(unprocessed.borrow()).unwrap())?;
        Ok(Source(processed))
    }

    /// Load the named shader from the given directory path.
    pub fn user<P: AsRef<Path>>(
        root: P,
        name: &str,
        suffix: &str,
    ) -> io::Result<Self> {
        let base_name = format!("{}_{}.glsl", name, suffix);
        let path = root.as_ref().join(&base_name);
        let unprocessed = util::read_file_to_string(Path::new(&path))?;
        let processed = Self::preprocess(root, &unprocessed)?;
        Ok(Source(processed))
    }
}

macro_rules! decl_shaders {
    { $(($pso:ident, $doc:ident, $ty:ident),)* } => {
        $( decl_shaders!($pso, $doc, $ty); )*

        /// The set of shaders needed by the `three` renderer.
        #[derive(Clone, Debug, Default)]
        pub struct Set {
            $(
                #[allow(missing_docs)]
                pub $pso: $ty,
            )*
        }
    };

    ($pso:ident, $doc:ident, $ty:ident) => {
        #[allow(missing_docs)]
        #[derive(Clone, Debug)]
        pub struct $ty {
            /// Vertex shader code.
            pub(crate) vs: Source,

            /// Pixel/fragment shader code.
            pub(crate) ps: Source,
        }

        impl $ty {
            /// Loads user shader code.
            pub fn user<P: AsRef<Path>>(root: P) -> io::Result<Self> {
                Ok(Self {
                    vs: Source::user(&root, stringify!($pso), "vs")?,
                    ps: Source::user(&root, stringify!($pso), "ps")?,
                })
            }
        }

        impl Default for $ty {
            fn default() -> Self {
                Self {
                    vs: Source::default(stringify!($pso), "vs").unwrap(),
                    ps: Source::default(stringify!($pso), "ps").unwrap(),
                }
            }
        }
    };
}

decl_shaders! {
    (basic, basic, Basic),
    (gouraud, Gouraud, Gouraud),
    (pbr, PBR, Pbr),
    (phong, Phong, Phong),
    (quad, quad, Quad),
    (shadow, shadow, Shadow),
    (skybox, skybox, Skybox),
    (sprite, sprite, Sprite),
}
