//! Mesh skinning.

use object;

/// Contains array of bones.
#[derive(Clone, Debug)]
pub struct Skeleton {
    pub(crate) object: object::Base,
}
three_object!(Skeleton::object);

/// A single bone that forms one component of a [`Skeleton`].
///
/// [`Skeleton`]: struct.Skeleton.html
#[derive(Clone, Debug)]
pub struct Bone {
    pub(crate) object: object::Base,
}
three_object!(Bone::object);
