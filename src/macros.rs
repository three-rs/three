/// Implements conversion traits on a type wrapping a `three` type. Useful for when you wrap a
/// `three` type with your own struct. Allows you to use that struct in place of any [`Object`].
///
/// # Examples
///
/// Creating a wrapper around a named field.
///
/// ```rust,no_run
/// #[macro_use]
/// extern crate three;
///
/// three_object!(MyStruct::mesh);
/// struct MyStruct {
///     mesh: three::Mesh,
/// }
/// # fn main() {}
/// ```
///
/// [`Object`]: object/struct.Object.html
#[macro_export]
macro_rules! three_object {
    ($name:ident::$field:ident) => {
        impl AsRef<$crate::Object> for $name {
            fn as_ref(&self) -> &$crate::Object {
                self.$field.as_ref()
            }
        }

        impl AsMut<$crate::Object> for $name {
            fn as_mut(&mut self) -> &mut $crate::Object {
                self.$field.as_mut()
            }
        }

        impl $name {
            /// Invisible objects are not rendered by cameras.
            #[allow(unused)]
            pub fn set_visible(
                &mut self,
                visible: bool,
            ) {
                self.$field.set_visible(visible)
            }

            /// Rotates object in the specific direction of `target`.
            #[allow(unused)]
            pub fn look_at<E, T>(
                &mut self,
                eye: E,
                target: T,
                up: Option<$crate::Vector3<f32>>,
            ) where
                E: Into<$crate::Point3<f32>>,
                T: Into<$crate::Point3<f32>>,
            {
                self.$field.look_at(eye, target, up)
            }

            /// Set both position, orientation and scale.
            #[allow(unused)]
            pub fn set_transform<P, Q>(
                &mut self,
                pos: P,
                rot: Q,
                scale: f32,
            ) where
                P: Into<$crate::Point3<f32>>,
                Q: Into<$crate::Quaternion<f32>>,
            {
                self.$field.set_transform(pos, rot, scale)
            }

            /// Add new [`Object`](struct.Object.html) to the group.
            #[allow(unused)]
            pub fn set_parent<P: AsRef<$crate::Object>>(
                &mut self,
                parent: &P,
            ) {
                self.$field.set_parent(parent)
            }

            /// Set position.
            #[allow(unused)]
            pub fn set_position<P>(
                &mut self,
                pos: P,
            ) where
                P: Into<$crate::Point3<f32>>,
            {
                self.$field.set_position(pos)
            }

            /// Set orientation.
            #[allow(unused)]
            pub fn set_orientation<Q>(
                &mut self,
                rot: Q,
            ) where
                Q: Into<$crate::Quaternion<f32>>,
            {
                self.$field.set_orientation(rot)
            }

            /// Set scale.
            #[allow(unused)]
            pub fn set_scale(
                &mut self,
                scale: f32,
            ) {
                self.$field.set_scale(scale)
            }

            /// Get actual information about itself from the `scene`.
            /// # Panics
            /// Panics if `scene` doesn't have this `Object`.
            #[allow(unused)]
            pub fn sync(
                &mut self,
                scene: &$crate::Scene,
            ) -> $crate::Node {
                self.$field.sync(scene)
            }
        }
    };
}

macro_rules! three_object_internal {
    ($name:ident::$field:ident) => {
        impl AsRef<$crate::Object> for $name {
            fn as_ref(&self) -> &$crate::Object {
                &self.$field
            }
        }

        impl AsMut<$crate::Object> for $name {
            fn as_mut(&mut self) -> &mut $crate::Object {
                &mut self.$field
            }
        }

        impl $name {
            /// Invisible objects are not rendered by cameras.
            pub fn set_visible(
                &mut self,
                visible: bool,
            ) {
                self.$field.set_visible(visible)
            }

            /// Rotates object in the specific direction of `target`.
            pub fn look_at<E, T>(
                &mut self,
                eye: E,
                target: T,
                up: Option<$crate::Vector3<f32>>,
            ) where
                E: Into<$crate::Point3<f32>>,
                T: Into<$crate::Point3<f32>>,
            {
                self.$field.look_at(eye, target, up)
            }

            /// Set both position, orientation and scale.
            pub fn set_transform<P, Q>(
                &mut self,
                pos: P,
                rot: Q,
                scale: f32,
            ) where
                P: Into<$crate::Point3<f32>>,
                Q: Into<$crate::Quaternion<f32>>,
            {
                self.$field.set_transform(pos, rot, scale)
            }

            /// Add new [`Object`](struct.Object.html) to the group.
            pub fn set_parent<P: AsRef<$crate::Object>>(
                &mut self,
                parent: &P,
            ) {
                self.$field.set_parent(parent)
            }

            /// Set position.
            pub fn set_position<P>(
                &mut self,
                pos: P,
            ) where
                P: Into<$crate::Point3<f32>>,
            {
                self.$field.set_position(pos)
            }

            /// Set orientation.
            pub fn set_orientation<Q>(
                &mut self,
                rot: Q,
            ) where
                Q: Into<$crate::Quaternion<f32>>,
            {
                self.$field.set_orientation(rot)
            }

            /// Set scale.
            pub fn set_scale(
                &mut self,
                scale: f32,
            ) {
                self.$field.set_scale(scale)
            }

            /// Get actual information about itself from the `scene`.
            /// # Panics
            /// Panics if `scene` doesn't have this `Object`.
            pub fn sync(
                &mut self,
                scene: &$crate::Scene,
            ) -> $crate::Node {
                self.$field.sync(scene)
            }
        }
    };
}
