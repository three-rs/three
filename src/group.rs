use object::Base;

/// Groups are used to combine several other objects or groups to work with
/// them as with a single entity.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Group {
    pub(crate) object: Base,
}
three_object!(Group::object);

impl Group {
    pub(crate) fn new(object: Base) -> Self {
        Group { object }
    }
}
