/// Implements the following traits:
///
/// * `AsRef<object::Base>`
/// * `AsMut<object::Base>`
/// * `Object`
///
/// # Examples
///
/// Creating a wrapper around a named field.
///
/// ```rust
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
/// If the field parameter is omitted then the field name defaults to `object`.
///
/// ```rust
/// #[macro_use]
/// extern crate three;
///
/// // Equivalent to `three_object!(MyStruct::object);`
/// three_object!(MyStruct);
/// struct MyStruct {
///     object: three::Mesh,
/// }
/// # fn main() {}
/// ```
///
/// [`object::Base`]: object/struct.Base.html
macro_rules! three_object {
    ($name:ident::$field:ident) => {
        impl AsRef<$crate::object::Base> for $name {
            fn as_ref(&self) -> &$crate::object::Base {
                &self.$field
            }
        }

        impl $crate::Object for $name {
            type Data = ();

            fn resolve_data(&self, _: &mut $crate::scene::SyncGuard) -> Self::Data {}
        }
    };
}
