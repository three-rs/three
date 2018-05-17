//! Mesh skinning.

use mint;
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

/// A matrix defining how bind mesh nodes to a bone.
pub type InverseBindMatrix = mint::ColumnMatrix4<f32>;
