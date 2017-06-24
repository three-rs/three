/// Implements conversion traits on a type wrapping a three-rs type. Useful for when you wrap a
/// three-rs type with your own struct. Allows you to use that struct in place of any three-rs
/// object.
///
/// Implements the following traits:
///
/// * `AsRef<NodePointer>`
/// * `Deref<Target=Object>`
/// * `DerefMut<Object>`
///
/// ```rust,ignore
/// // Implements conversion traits for MyStruct using field `internal_three_type`
/// three_object_wrapper!(MyStruct::internal_three_type);
/// // If field is omitted, it defaults to `object`
/// three_object_wrapper!(MyStruct); // equivalent to three_object_wrapper!(MyStruct::object);
/// ```
#[macro_export]
macro_rules! three_object_wrapper {
    ($($name:ident),*) => {
        three_object_wrapper!($($name::object),*);
    };
    ($($name:ident::$field:ident),*) => {
        $(
            impl AsRef<$crate::NodePointer> for $name {
                fn as_ref(&self) -> &$crate::NodePointer {
                    self.$field.as_ref()
                }
            }

            impl ::std::ops::Deref for $name {
                type Target = Object;
                fn deref(&self) -> &Object {
                    &self.$field
                }
            }

            impl ::std::ops::DerefMut for $name {
                fn deref_mut(&mut self) -> &mut Object {
                    &mut self.$field
                }
            }
        )*
    };
}
