use hub::Operation;
use mint;
use object;

/// Two-dimensional bitmap that is integrated into a larger scene.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Sprite {
    pub(crate) object: object::Base,
}
three_object!(Sprite::object);
derive_DowncastObject!(Sprite => object::ObjectType::Sprite);

impl Sprite {
    pub(crate) fn new(object: object::Base) -> Self {
        Sprite { object }
    }

    /// Set area of the texture to render. It can be used in sequential animations.
    pub fn set_texel_range<P, S>(&mut self, base: P, size: S)
    where
        P: Into<mint::Point2<i16>>,
        S: Into<mint::Vector2<u16>>,
    {
        let msg = Operation::SetTexelRange(base.into(), size.into());
        let _ = self.object.tx.send((self.object.node.downgrade(), msg));
    }
}
