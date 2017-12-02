/// Implements conversion traits on a type wrapping a `three` type. Useful for when you wrap a
/// `three` type with your own struct. Allows you to use that struct in place of any [`Object`].
///
/// Used internally only.
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
macro_rules! three_object {
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

        impl ::std::ops::Deref for $name {
            type Target = $crate::Object;
            fn deref(&self) -> &Self::Target {
                &self.$field
            }
        }

        impl ::std::ops::DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.$field
            }
        }
    };
}
