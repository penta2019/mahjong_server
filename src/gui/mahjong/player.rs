use super::{discard::GuiDiscard, hand::GuiHand, meld::GuiMeld, prelude::*};

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

impl GuiPlayer {
    pub fn new() -> Self {
        let cmd = &mut param().cmd;

        let entity = cmd.spawn(Name::new("Player")).id();

        let hand = GuiHand::new();
        cmd.entity(hand.entity())
            .insert((ChildOf(entity), TF_CLOSE_HAND));

        let discard = GuiDiscard::new();
        cmd.entity(discard.entity()).insert((
            ChildOf(entity),
            Transform {
                translation: Vec3::new(-0.05, GuiTile::DEPTH / 2.0, 0.074),
                rotation: Quat::from_rotation_x(-FRAC_PI_2),
                scale: Vec3::ONE,
            },
        ));

        let meld = GuiMeld::new();
        cmd.entity(meld.entity()).insert((
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
        param().cmd.entity(self.hand.entity()).insert(tf);
    }

    pub fn init_hand(&mut self, m_tiles: &[Tile]) {
        self.hand.init(m_tiles);
        self.hand.align();
    }

    pub fn deal_tile(&mut self, tile: GuiTile) {
        self.hand.deal_tile(tile);
    }

    pub fn discard_tile(&mut self, m_tile: Tile, is_drawn: bool, is_riichi: bool) {
        if is_riichi {
            self.discard.set_riichi();
        }
        let tile = self.take_tile_from_hand(m_tile, is_drawn);
        self.discard.push_tile(tile);
        self.hand.align();
    }

    pub fn confirm_discard_tile(&mut self) {
        self.discard.confirm_last_tile();
    }

    pub fn meld(&mut self, m_tiles: &[Tile], meld_tile: Option<GuiTile>, meld_offset: usize) {
        let tiles_from_hand: Vec<GuiTile> = m_tiles
            .iter()
            .map(|t| self.take_tile_from_hand(*t, false))
            .collect();
        self.meld.meld(tiles_from_hand, meld_tile, meld_offset);
        self.hand.align();
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

impl HasEntity for GuiPlayer {
    fn entity(&self) -> Entity {
        self.entity
    }
}
