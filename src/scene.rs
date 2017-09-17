use hub::{HubPtr, Message, Operation};
use node::NodePointer;
use std::sync::mpsc;
use texture::Texture;

/// Color represented by 4-bytes hex number.
pub type Color = u32;

pub type SceneId = usize;

/// Background type.
#[derive(Clone, Debug)]
pub enum Background {
    /// Basic solid color background.
    Color(Color),
    /// Texture background, covers the whole screen.
    // TODO: different wrap modes?
    Texture(Texture<[f32; 4]>),
    //TODO: cubemap
}

/// Game scene contains game objects and can be rendered by [`Camera`](struct.Camera.html).
pub struct Scene {
    pub(crate) unique_id: SceneId,
    pub(crate) node: NodePointer,
    pub(crate) tx: mpsc::Sender<Message>,
    pub(crate) hub: HubPtr,
    /// See [`Background`](struct.Background.html).
    pub background: Background,
}

impl Scene {
    /// Add new [`Object`](struct.Object.html) to the scene.
    pub fn add<P: AsRef<NodePointer>>(
        &mut self,
        child: &P,
    ) {
        let msg = Operation::SetParent(self.node.clone());
        let _ = self.tx.send((child.as_ref().downgrade(), msg));
    }
}

/// This is a module that contains the standard web colors in a format the three
/// expects.
#[allow(missing_docs)]
pub mod web_colors {
    use super::Color;

    pub const WHTIE: Color = 0xff_ff_ff;
    pub const SILVER: Color = 0xc0_c0_c0;
    pub const GRAY: Color = 0x80_80_80;
    pub const BLACK: Color = 0x00_00_00;
    pub const RED: Color = 0xff_00_00;
    pub const MAROON: Color = 0x80_00_00;
    pub const YELLOW: Color = 0xff_ff_00;
    pub const OLIVE: Color = 0x80_80_00;
    pub const LIME: Color = 0x00_ff_00;
    pub const GREEN: Color = 0x00_80_00;
    pub const AQUA: Color = 0x00_ff_ff;
    pub const TEAL: Color = 0x00_80_80;
    pub const BLUE: Color = 0x00_00_ff;
    pub const NAVY: Color = 0x00_00_80;
    pub const FUCHSIA: Color = 0xff_00_ff;
    pub const PURPLE: Color = 0x80_00_80;
}

/// This contains a list of crayon colors from a major crayon vendor.
#[allow(missing_docs)]
pub mod crayon_colors {
    use super::Color;

    pub const RED: Color = 0xED0A3F;
    pub const MAROON: Color = 0xC32148;
    pub const SCARLET: Color = 0xFD0E35;
    pub const BRICK_RED: Color = 0xC62D42;
    pub const ENGLISH_VERMILION: Color = 0xCC474B;
    pub const MADDER_LAKE: Color = 0xCC3336;
    pub const PERMANENT_GERANIUM_LAKE: Color = 0xE12C2C;
    pub const MAXIMUM_RED: Color = 0xD92121;
    pub const INDIAN_RED: Color = 0xB94E48;
    pub const ORANGE_RED: Color = 0xFF5349;
    pub const SUNSET_ORANGE: Color = 0xFE4C40;
    pub const BITTERSWEET: Color = 0xFE6F5E;
    pub const DARK_VENETIAN_RED: Color = 0xB33B24;
    pub const VENETIAN_RED: Color = 0xCC553D;
    pub const LIGHT_VENETIAN_RED: Color = 0xE6735C;
    pub const VIVID_TANGERINE: Color = 0xFF9980;
    pub const MIDDLE_RED: Color = 0xE58E73;
    pub const BURNT_ORANGE: Color = 0xFF7F49;
    pub const RED_ORANGE: Color = 0xFF681F;
    pub const ORANGE: Color = 0xFF8833;
    pub const MACARONI_AND_CHEESE: Color = 0xFFB97B;
    pub const MIDDLE_YELLOW_RED: Color = 0xECB176;
    pub const MANGO_TANGO: Color = 0xE77200;
    pub const YELLOW_ORANGE: Color = 0xFFAE42;
    pub const MAXIMUM_YELLOW_RED: Color = 0xF2BA49;
    pub const BANANA_MANIA: Color = 0xFBE7B2;
    pub const MAIZE: Color = 0xF2C649;
    pub const ORANGE_YELLOW: Color = 0xF8D568;
    pub const GOLDENROD: Color = 0xFCD667;
    pub const DANDELION: Color = 0xFED85D;
    pub const YELLOW: Color = 0xFBE870;
    pub const GREEN_YELLOW: Color = 0xF1E788;
    pub const MIDDLE_YELLOW: Color = 0xFFEB00;
    pub const OLIVE_GREEN: Color = 0xB5B35C;
    pub const SPRING_GREEN: Color = 0xECEBBD;
    pub const MAXIMUM_YELLOW: Color = 0xFAFA37;
    pub const CANARY: Color = 0xFFFF99;
    pub const LEMON_YELLOW: Color = 0xFFFF9F;
    pub const MAXIMUM_GREEN_YELLOW: Color = 0xD9E650;
    pub const MIDDLE_GREEN_YELLOW: Color = 0xACBF60;
    pub const INCHWORM: Color = 0xAFE313;
    pub const LIGHT_CHROME_GREEN: Color = 0xBEE64B;
    pub const YELLOW_GREEN: Color = 0xC5E17A;
    pub const MAXIMUM_GREEN: Color = 0x5E8C31;
    pub const ASPARAGUS: Color = 0x7BA05B;
    pub const GRANNY_SMITH_APPLE: Color = 0x9DE093;
    pub const FERN: Color = 0x63B76C;
    pub const MIDDLE_GREEN: Color = 0x4D8C57;
    pub const GREEN: Color = 0x3AA655;
    pub const MEDIUM_CHROME_GREEN: Color = 0x6CA67C;
    pub const FOREST_GREEN: Color = 0x5FA777;
    pub const SEA_GREEN: Color = 0x93DFB8;
    pub const SHAMROCK: Color = 0x33CC99;
    pub const MOUNTAIN_MEADOW: Color = 0x1AB385;
    pub const JUNGLE_GREEN: Color = 0x29AB87;
    pub const CARIBBEAN_GREEN: Color = 0x00CC99;
    pub const TROPICAL_RAIN_FOREST: Color = 0x00755E;
    pub const MIDDLE_BLUE_GREEN: Color = 0x8DD9CC;
    pub const PINE_GREEN: Color = 0x01786F;
    pub const MAXIMUM_BLUE_GREEN: Color = 0x30BFBF;
    pub const ROBINS_EGG_BLUE: Color = 0x00CCCC;
    pub const TEAL_BLUE: Color = 0x008080;
    pub const LIGHT_BLUE: Color = 0x8FD8D8;
    pub const AQUAMARINE: Color = 0x95E0E8;
    pub const TURQUOISE_BLUE: Color = 0x6CDAE7;
    pub const OUTER_SPACE: Color = 0x2D383A;
    pub const SKY_BLUE: Color = 0x76D7EA;
    pub const MIDDLE_BLUE: Color = 0x7ED4E6;
    pub const BLUE_GREEN: Color = 0x0095B7;
    pub const PACIFIC_BLUE: Color = 0x009DC4;
    pub const CERULEAN: Color = 0x02A4D3;
    pub const MAXIMUM_BLUE: Color = 0x47ABCC;
    pub const CORNFLOWER: Color = 0x93CCEA;
    pub const GREEN_BLUE: Color = 0x2887C8;
    pub const MIDNIGHT_BLUE: Color = 0x00468C;
    pub const NAVY_BLUE: Color = 0x0066CC;
    pub const DENIM: Color = 0x1560BD;
    pub const BLUE: Color = 0x0066FF;
    pub const CADET_BLUE: Color = 0xA9B2C3;
    pub const PERIWINKLE: Color = 0xC3CDE6;
    pub const BLUETIFUL: Color = 0x3C69E7;
    pub const WILD_BLUE_YONDER: Color = 0x7A89B8;
    pub const INDIGO: Color = 0x4F69C6;
    pub const MANATEE: Color = 0x8D90A1;
    pub const COBALT_BLUE: Color = 0x8C90C8;
    pub const CELESTIAL_BLUE: Color = 0x7070CC;
    pub const BLUE_BELL: Color = 0x9999CC;
    pub const MAXIMUM_BLUE_PURPLE: Color = 0xACACE6;
    pub const VIOLET_BLUE: Color = 0x766EC8;
    pub const BLUE_VIOLET: Color = 0x6456B7;
    pub const ULTRAMARINE_BLUE: Color = 0x3F26BF;
    pub const MIDDLE_BLUE_PURPLE: Color = 0x8B72BE;
    pub const PURPLE_HEART: Color = 0x652DC1;
    pub const ROYAL_PURPLE: Color = 0x6B3FA0;
    pub const VIOLET: Color = 0x8359A3;
    pub const MEDIUM_VIOLET: Color = 0x8F47B3;
    pub const WISTERIA: Color = 0xC9A0DC;
    pub const VIVID_VIOLET: Color = 0x803790;
    pub const MAXIMUM_PURPLE: Color = 0x733380;
    pub const PURPLE_MOUNTAINS_MAJESTY: Color = 0xD6AEDD;
    pub const FUCHSIA: Color = 0xC154C1;
    pub const PINK_FLAMINGO: Color = 0xFC74FD;
    pub const BRILLIANT_ROSE: Color = 0xE667CE;
    pub const ORCHID: Color = 0xE29CD2;
    pub const PLUM: Color = 0x8E3179;
    pub const MEDIUM_ROSE: Color = 0xD96CBE;
    pub const THISTLE: Color = 0xEBB0D7;
    pub const MULBERRY: Color = 0xC8509B;
    pub const RED_VIOLET: Color = 0xBB3385;
    pub const MIDDLE_PURPLE: Color = 0xD982B5;
    pub const MAXIMUM_RED_PURPLE: Color = 0xA63A79;
    pub const JAZZBERRY_JAM: Color = 0xA50B5E;
    pub const EGGPLANT: Color = 0x614051;
    pub const MAGENTA: Color = 0xF653A6;
    pub const CERISE: Color = 0xDA3287;
    pub const WILD_STRAWBERRY: Color = 0xFF3399;
    pub const LAVENDER: Color = 0xFBAED2;
    pub const COTTON_CANDY: Color = 0xFFB7D5;
    pub const CARNATION_PINK: Color = 0xFFA6C9;
    pub const VIOLET_RED: Color = 0xF7468A;
    pub const RAZZMATAZZ: Color = 0xE30B5C;
    pub const PIG_PINK: Color = 0xFDD7E4;
    pub const CARMINE: Color = 0xE62E6B;
    pub const BLUSH: Color = 0xDB5079;
    pub const TICKLE_ME_PINK: Color = 0xFC80A5;
    pub const MAUVELOUS: Color = 0xF091A9;
    pub const SALMON: Color = 0xFF91A4;
    pub const MIDDLE_RED_PURPLE: Color = 0xA55353;
    pub const MAHOGANY: Color = 0xCA3435;
    pub const MELON: Color = 0xFEBAAD;
    pub const PINK_SHERBERT: Color = 0xF7A38E;
    pub const BURNT_SIENNA: Color = 0xE97451;
    pub const BROWN: Color = 0xAF593E;
    pub const SEPIA: Color = 0x9E5B40;
    pub const FUZZY_WUZZY: Color = 0x87421F;
    pub const BEAVER: Color = 0x926F5B;
    pub const TUMBLEWEED: Color = 0xDEA681;
    pub const RAW_SIENNA: Color = 0xD27D46;
    pub const VAN_DYKE_BROWN: Color = 0x664228;
    pub const TAN: Color = 0xD99A6C;
    pub const DESERT_SAND: Color = 0xEDC9AF;
    pub const PEACH: Color = 0xFFCBA4;
    pub const BURNT_UMBER: Color = 0x805533;
    pub const APRICOT: Color = 0xFDD5B1;
    pub const ALMOND: Color = 0xEED9C4;
    pub const RAW_UMBER: Color = 0x665233;
    pub const SHADOW: Color = 0x837050;
    pub const TIMBERWOLF: Color = 0xD9D6CF;
    pub const GOLD: Color = 0xE6BE8A;
    pub const SILVER: Color = 0xC9C0BB;
    pub const COPPER: Color = 0xDA8A67;
    pub const ANTIQUE_BRASS: Color = 0xC88A65;
    pub const BLACK: Color = 0x000000;
    pub const CHARCOAL_GRAY: Color = 0x736A62;
    pub const GRAY: Color = 0x8B8680;
    pub const BLUE_GRAY: Color = 0xC8C8CD;
    pub const WHITE: Color = 0xFFFFFF;
}
