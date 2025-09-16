use super::*;

#[derive(Debug)]
pub struct GuiPlayer {
    entity: Entity,
    tf_hand: Transform,
    hand: GuiHand,
    discard: GuiDiscard,
    meld: GuiMeld,
}

impl GuiPlayer {
    pub fn new() -> Self {
        let commands = &mut param().commands;

        let entity = commands.spawn(Name::new("Player")).id();

        let tf_hand = Transform::from_xyz(-0.12, 0., 0.21);
        let hand = GuiHand::new();
        commands
            .entity(hand.entity())
            .insert((ChildOf(entity), tf_hand));

        let discard = GuiDiscard::new();
        commands.entity(discard.entity()).insert((
            ChildOf(entity),
            Transform {
                translation: Vec3::new(-0.05, GuiTile::DEPTH / 2., 0.074),
                rotation: Quat::from_rotation_x(-FRAC_PI_2),
                scale: Vec3::ONE,
            },
        ));

        let meld = GuiMeld::new();
        commands.entity(meld.entity()).insert((
            ChildOf(entity),
            Transform {
                translation: Vec3::new(0.25, GuiTile::DEPTH / 2., 0.22),
                rotation: Quat::from_rotation_x(-FRAC_PI_2),
                scale: Vec3::ONE,
            },
        ));

        Self {
            entity,
            tf_hand,
            hand,
            discard,
            meld,
        }
    }

    pub fn set_player_mode(&mut self, flag: bool) {
        use super::stage::{CAMERA_LOOK_AT, CAMERA_POS};
        let tf = if flag {
            let tf_camera =
                Transform::from_translation(CAMERA_POS).looking_at(CAMERA_LOOK_AT, Vec3::Y);
            let tf_hand = Transform {
                translation: Vec3::new(-0.13, -0.13, -0.9),
                rotation: Quat::from_rotation_x(10.0_f32.to_radians()),
                scale: Vec3::ONE,
            };
            tf_camera * tf_hand
        } else {
            self.tf_hand
        };
        param().commands.entity(self.hand.entity()).insert(tf);
    }

    pub fn init_hand(&mut self, m_tiles: &[Tile]) {
        self.hand.init(m_tiles);
        self.hand.align();
    }

    pub fn deal_tile(&mut self, m_tile: Tile) {
        self.hand.deal_tile(m_tile);
    }

    pub fn discard_tile(&mut self, m_tile: Tile, is_drawn: bool, is_riichi: bool) {
        if is_riichi {
            self.discard.set_riichi();
        }
        let tile = self.hand.take_tile(m_tile, is_drawn);
        self.discard.push_tile(tile);
        self.hand.align();
    }

    pub fn meld(&mut self, m_tiles: &[Tile], meld_tile: Option<GuiTile>, meld_offset: usize) {
        let tiles_from_hand: Vec<GuiTile> = m_tiles
            .iter()
            .map(|t| self.hand.take_tile(*t, false))
            .collect();
        self.meld.meld(tiles_from_hand, meld_tile, meld_offset);
        self.hand.align();
    }

    pub fn take_last_discard_tile(&mut self) -> GuiTile {
        self.discard.take_last_tile()
    }
}

impl HasEntity for GuiPlayer {
    fn entity(&self) -> Entity {
        self.entity
    }
}
