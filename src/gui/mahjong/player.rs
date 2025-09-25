use super::*;

pub enum HandMode {
    Camera,
    Close,
    Open,
}

#[derive(Debug)]
pub struct GuiPlayer {
    entity: Entity,
    tf_close_hand: Transform,
    hand: GuiHand,
    discard: GuiDiscard,
    meld: GuiMeld,
    target_tile: Option<Entity>,
}

impl GuiPlayer {
    pub fn new() -> Self {
        let commands = &mut param().commands;

        let entity = commands.spawn(Name::new("Player")).id();

        let tf_close_hand = Transform::from_xyz(-0.12, 0., 0.21);
        let hand = GuiHand::new();
        commands
            .entity(hand.entity())
            .insert((ChildOf(entity), tf_close_hand));

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
            tf_close_hand,
            hand,
            discard,
            meld,
            target_tile: None,
        }
    }

    pub fn set_hand_mode(&mut self, mode: HandMode) {
        use super::stage::{CAMERA_LOOK_AT, CAMERA_POS};
        let tf = match mode {
            HandMode::Camera => {
                let tf_camera =
                    Transform::from_translation(CAMERA_POS).looking_at(CAMERA_LOOK_AT, Vec3::Y);
                let tf_close_hand = Transform {
                    translation: Vec3::new(-0.13, -0.13, -0.9),
                    rotation: Quat::from_rotation_x(10.0_f32.to_radians()),
                    scale: Vec3::ONE,
                };
                tf_camera * tf_close_hand
            }
            HandMode::Close => self.tf_close_hand,
            HandMode::Open => {
                let mut tf = self.tf_close_hand;
                tf.rotation = Quat::from_rotation_x(-FRAC_PI_2);
                tf
            }
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

    pub fn set_target_tile(&mut self, tile: Option<Entity>) {
        if tile == self.target_tile {
            return;
        }

        // 元々のtarget_tileを解除
        let param = param();
        if let Some(e_tile) = self.target_tile
            && let Ok(tile_tag) = param.tile_tags.get(e_tile)
        {
            tile_tag.set_highlight(&mut param.materials, false);
        }
        self.target_tile = None;

        // 新しいtarget_tileを指定
        if let Some(e_tile) = tile
            && let Ok(tile_tag) = param.tile_tags.get(e_tile)
            && self.hand.find_tile_from_entity(e_tile).is_some()
        {
            tile_tag.set_highlight(&mut param.materials, true);
            self.target_tile = tile;
        }
    }

    pub fn confirm_discard_tile(&mut self) {
        self.discard.confirm_last_tile();
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
