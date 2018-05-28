//! Mesh skinning.

use mint;
use object::{self, ObjectType};

/// Contains array of bones.
#[derive(Clone, Debug)]
pub struct Skeleton {
    pub(crate) object: object::Base,
}
three_object!(Skeleton::object);
derive_DowncastObject!(Skeleton => ObjectType::Skeleton);

/// A single bone that forms one component of a [`Skeleton`].
///
/// [`Skeleton`]: struct.Skeleton.html
#[derive(Clone, Debug)]
pub struct Bone {
    pub(crate) object: object::Base,
}
three_object!(Bone::object);
derive_DowncastObject!(Bone => ObjectType::Bone);

/// A matrix defining how bind mesh nodes to a bone.
pub type InverseBindMatrix = mint::ColumnMatrix4<f32>;
