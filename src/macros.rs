/// Implements conversion traits on a type wrapping a `three` type. Useful for when you wrap a
/// `three` type with your own struct. Allows you to use that struct in place of any [`Object`].
///
/// Implements the following traits:
///
/// * `Deref<Target=Object>`
/// * `DerefMut<Object>`
///
/// # Examples
///
/// Creating a wrapper around a named field.
///
/// ```rust
/// #[macro_use]
/// extern crate three;
///
/// three_object_wrapper!(MyStruct::mesh);
/// struct MyStruct {
///     mesh: three::Mesh,
/// }
/// # fn main() {}
/// ```
///
/// If the field parameter is omitted then the field name defaults to `object`.
///
/// ```rust
/// #[macro_use]
/// extern crate three;
///
/// // Equivalent to `three_object_wrapper!(MyStruct::object);`
/// three_object_wrapper!(MyStruct);
/// struct MyStruct {
///     object: three::Mesh,
/// }
/// # fn main() {}
/// ```
///
/// [`Object`]: object/struct.Object.html
#[macro_export]
macro_rules! three_object_wrapper {
    ($($name:ident),*) => {
        three_object_wrapper!($($name::object),*);
    };
    ($($name:ident::$field:ident),*) => {
        $(
            impl AsRef<$crate::Object> for $name {
                fn as_ref(&self) -> &$crate::Object {
                    &self.$field
                }
            }

            impl ::std::ops::Deref for $name {
                type Target = $crate::Object;
                fn deref(&self) -> &$crate::Object {
                    &self.$field
                }
            }

            impl ::std::ops::DerefMut for $name {
                fn deref_mut(&mut self) -> &mut $crate::Object {
                    &mut self.$field
                }
            }
        )*
    };
}
