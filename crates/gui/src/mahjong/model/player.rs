use super::{super::prelude::*, discard::GuiDiscard, hand::GuiHand, meld::GuiMeld};

const TF_CLOSE_HAND: Transform = Transform::from_xyz(-0.12, 0.0, 0.235);

pub enum HandMode {
    Camera,
    Close,
    Open,
}

#[derive(Debug)]
pub struct GuiPlayer {
    entity: Entity,
    hand: GuiHand,
    discard: GuiDiscard,
    meld: GuiMeld,
}
crate::impl_has_entity!(GuiPlayer);

impl GuiPlayer {
    pub fn new() -> Self {
        let entity = cmd().spawn(Name::new("Player")).id();

        let hand = GuiHand::new();
        hand.insert((ChildOf(entity), TF_CLOSE_HAND));

        let discard = GuiDiscard::new();
        discard.insert((
            ChildOf(entity),
            Transform {
                translation: Vec3::new(-GuiTile::WIDTH * 2.5, GuiTile::DEPTH / 2.0, 0.074),
                rotation: Quat::from_rotation_x(-FRAC_PI_2),
                scale: Vec3::ONE,
            },
        ));

        let meld = GuiMeld::new();
        meld.insert((
            ChildOf(entity),
            Transform {
                translation: Vec3::new(0.25, GuiTile::DEPTH / 2.0, 0.23),
                rotation: Quat::from_rotation_x(-FRAC_PI_2),
                scale: Vec3::ONE,
            },
        ));

        Self {
            entity,
            hand,
            discard,
            meld,
        }
    }

    pub fn set_hand_mode(&mut self, mode: HandMode) {
        let tf = match mode {
            HandMode::Camera => {
                use super::stage::{CAMERA_LOOK_AT, CAMERA_POS};
                let tf_camera =
                    Transform::from_translation(CAMERA_POS).looking_at(CAMERA_LOOK_AT, Vec3::Y);
                let tf_camera_hand = Transform {
                    translation: Vec3::new(-0.13, -0.132, -0.9),
                    rotation: Quat::from_rotation_x(10.0_f32.to_radians()),
                    scale: Vec3::ONE,
                };
                tf_camera * tf_camera_hand
            }
            HandMode::Close => TF_CLOSE_HAND,
            HandMode::Open => TF_CLOSE_HAND.with_rotation(Quat::from_rotation_x(-FRAC_PI_2)),
        };
        self.hand.insert(tf);
    }

    pub fn init_hand(&mut self, tiles: Vec<GuiTile>) {
        self.hand.init(tiles);
        self.hand.align(true);
    }

    pub fn deal_tile(&mut self, tile: GuiTile) {
        self.hand.deal_tile(tile, true);
    }

    pub fn discard_tile(&mut self, m_tile: Tile, is_drawn: bool, is_riichi: bool) {
        if is_riichi {
            self.discard.set_riichi();
        }
        let tile = self.take_tile_from_hand(m_tile, is_drawn);
        self.discard.push_tile(tile);
        self.hand.align(true);
    }

    pub fn confirm_discard_tile(&mut self) {
        self.discard.confirm_last_tile();
    }

    pub fn meld(&mut self, self_tiles: &[Tile], meld_tile: Option<(GuiTile, usize)>) {
        let self_tiles: Vec<GuiTile> = self_tiles
            .iter()
            .map(|t| self.take_tile_from_hand(*t, false))
            .collect();
        self.meld.meld(self_tiles, meld_tile, true);
        self.hand.align(true);
    }

    pub fn take_last_discard_tile(&mut self) -> GuiTile {
        self.discard.take_last_tile()
    }

    pub fn hand(&mut self) -> &mut GuiHand {
        &mut self.hand
    }

    fn take_tile_from_hand(&mut self, m_tile: Tile, is_drawn: bool) -> GuiTile {
        let mut tile = self.hand.take_tile(m_tile, is_drawn);
        tile.blend(GuiTile::NORMAL);
        tile
    }
}
