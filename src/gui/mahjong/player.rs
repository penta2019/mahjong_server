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
    pub fn new(param: &mut StageParam, parent: Entity, seat: Seat) -> Self {
        let e_player = param
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
            hand: GuiHand::new(param, e_player, seat),
            discard: GuiDiscard::new(param, e_player, seat),
            meld: GuiMeld::new(param, e_player, seat),
        }
    }

    pub fn init_hand(&mut self, param: &mut StageParam, tiles: &[Tile]) {
        self.hand.init(param, tiles);
        self.hand.align(param);
    }

    pub fn deal_tile(&mut self, param: &mut StageParam, tile: Tile) {
        self.hand.deal_tile(param, tile);
    }

    pub fn discard_tile(
        &mut self,
        param: &mut StageParam,
        tile: Tile,
        is_drawn: bool,
        is_riichi: bool,
    ) {
        if is_riichi {
            self.discard.set_riichi();
        }
        let gui_tile = self.hand.take_tile(param, tile, is_drawn);
        self.discard.push_tile(param, gui_tile);
        self.hand.align(param);
    }
}
