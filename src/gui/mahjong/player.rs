use super::*;

#[derive(Debug)]
pub struct GuiPlayer {
    entity: Entity,
    hand: GuiHand,
    discard: GuiDiscard,
    meld: GuiMeld,
}

impl GuiPlayer {
    pub fn new() -> Self {
        let entity = param().commands.spawn(Name::new("Player")).id();

        let commands = &mut param().commands;

        let hand = GuiHand::new();
        commands
            .entity(hand.entity())
            .insert((ChildOf(entity), Transform::from_xyz(-0.12, 0., 0.21)));

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
                translation: Vec3::new(0., 0., 0.074),
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

    pub fn init_hand(&mut self, tiles: &[Tile]) {
        self.hand.init(tiles);
        self.hand.align();
    }

    pub fn deal_tile(&mut self, tile: Tile) {
        self.hand.deal_tile(tile);
    }

    pub fn discard_tile(&mut self, tile: Tile, is_drawn: bool, is_riichi: bool) {
        if is_riichi {
            self.discard.set_riichi();
        }
        let gui_tile = self.hand.take_tile(tile, is_drawn);
        self.discard.push_tile(gui_tile);
        self.hand.align();
    }
}

impl HasEntity for GuiPlayer {
    fn entity(&self) -> Entity {
        self.entity
    }
}
