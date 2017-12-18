use arrayvec::ArrayVec;
use hub::Operation;
use mesh::{MAX_TARGETS, Weight};
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

    /// Applies morph target weights to the direct mesh descendents of the
    /// group.
    pub fn set_weights(
        &mut self,
        weights: ArrayVec<[Weight; MAX_TARGETS]>,
    ) {
        let msg = Operation::SetWeights(weights);
        let _ = self.object.tx.send((self.object.node.downgrade(), msg));
    }
}
