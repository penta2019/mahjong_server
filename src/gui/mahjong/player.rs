use super::*;

#[derive(Debug)]
pub struct GuiPlayer {
    entity: Entity,
    seat: Seat,
    hand: GuiHand,
    discard: GuiDiscard,
    meld: GuiMeld,
}

impl GuiPlayer {
    pub fn new(parent: Entity, seat: Seat) -> Self {
        let e_player = param()
            .commands
            .spawn((
                Name::new(format!("Player[{seat}]")),
                ChildOf(parent),
                Transform {
                    rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_2 * seat as f32),
                    ..Default::default()
                },
            ))
            .id();
        Self {
            entity: e_player,
            seat,
            hand: GuiHand::new(e_player, seat),
            discard: GuiDiscard::new(e_player, seat),
            meld: GuiMeld::new(e_player, seat),
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
