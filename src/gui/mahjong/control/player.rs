use bevy::input::ButtonState;

use super::*;

const COLOR_ACTIVE: LinearRgba = LinearRgba::new(0., 0.1, 0., 0.); // ハイライト (打牌可)
const COLOR_INACTIVE: LinearRgba = LinearRgba::new(0.1, 0., 0., 0.); // ハイライト (打牌不可)
const COLOR_NORMAL: LinearRgba = LinearRgba::BLACK; // ハイライトなし

pub enum HandMode {
    Camera,
    Close,
    Open,
}

#[derive(Debug, PartialEq, Eq)]
enum TargetState {
    Released,
    Pressed,
    // Dragging,
}

#[derive(Debug)]
pub struct GuiPlayer {
    // Event用
    entity: Entity,
    tf_close_hand: Transform,
    hand: GuiHand,
    discard: GuiDiscard,
    meld: GuiMeld,
    // Action用
    target_tile: Option<Entity>,
    target_state: TargetState,
    can_discard: bool,
    possible_actions: Option<PossibleActions>,
    preferred_discard_tile: Option<Entity>,
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
            target_state: TargetState::Released,
            can_discard: false,
            possible_actions: None,
            preferred_discard_tile: None,
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
        self.set_target_tile(None);

        if is_riichi {
            self.discard.set_riichi();
        }
        let tile = self
            .hand
            .take_tile(m_tile, is_drawn, self.preferred_discard_tile);
        self.preferred_discard_tile = None;
        self.discard.push_tile(tile);
        self.hand.align();
    }

    pub fn confirm_discard_tile(&mut self) {
        self.discard.confirm_last_tile();
    }

    pub fn meld(&mut self, m_tiles: &[Tile], meld_tile: Option<GuiTile>, meld_offset: usize) {
        self.set_target_tile(None);

        let tiles_from_hand: Vec<GuiTile> = m_tiles
            .iter()
            .map(|t| self.hand.take_tile(*t, false, None))
            .collect();
        self.meld.meld(tiles_from_hand, meld_tile, meld_offset);
        self.hand.align();
    }

    pub fn take_last_discard_tile(&mut self) -> GuiTile {
        self.discard.take_last_tile()
    }

    pub fn handle_gui_events(&mut self) -> Option<SelectedAction> {
        let param = param();
        let mut action = None;

        for ev in param.hovered_tile.read() {
            self.set_target_tile(ev.tile_entity);
        }

        for ev in param.mouse_input.read() {
            match ev.button {
                MouseButton::Left => match ev.state {
                    ButtonState::Pressed => {
                        if self.target_tile.is_some() {
                            self.target_state = TargetState::Pressed;
                        }
                    }
                    ButtonState::Released => {
                        if self.target_state == TargetState::Pressed {
                            action = self.action_discard_tile();
                        }
                        self.target_state = TargetState::Released;
                    }
                },
                MouseButton::Right => match ev.state {
                    ButtonState::Pressed => {
                        action = self.action_nop();
                    }
                    ButtonState::Released => {}
                },
                _ => {}
            }
        }

        // TODO: 打牌以外は一旦今はスキップ
        if let Some(actions) = &self.possible_actions
            && actions
                .actions
                .iter()
                .all(|a| a.action_type != ActionType::Discard)
        {
            action = Some(SelectedAction {
                id: actions.id,
                action: Action::nop(),
            });
        }

        if action.is_some() {
            self.possible_actions = None;
        }

        action
    }

    pub fn handle_actions(&mut self, possible_actions: PossibleActions) {
        self.set_can_discard(true);
        self.possible_actions = Some(possible_actions);
    }

    pub fn on_event(&mut self) {
        self.set_can_discard(false);
        self.possible_actions = None;
    }

    fn set_can_discard(&mut self, flag: bool) {
        self.can_discard = flag;
        self.change_target_tile_color(if self.can_discard {
            COLOR_ACTIVE
        } else {
            COLOR_INACTIVE
        });
    }

    fn set_target_tile(&mut self, tile: Option<Entity>) {
        // 牌が変化していない場合何もしない
        if tile == self.target_tile {
            return;
        }

        // 元々のtarget_tileを解除
        self.change_target_tile_color(COLOR_NORMAL);
        self.target_tile = None;
        self.target_state = TargetState::Released;

        // 新しいtarget_tileを指定
        if let Some(e_tile) = tile
            && self.hand.find_tile_from_entity(e_tile).is_some()
        {
            self.target_tile = tile;
            self.change_target_tile_color(if self.can_discard {
                COLOR_ACTIVE
            } else {
                COLOR_INACTIVE
            });
        }
    }

    fn change_target_tile_color(&self, color: LinearRgba) {
        if let Some(e_tile) = self.target_tile
            && let Some((tile, _)) = self.hand.find_tile_from_entity(e_tile)
        {
            tile.set_emissive(color);
        }
    }

    fn action_nop(&mut self) -> Option<SelectedAction> {
        if let Some(actions) = &self.possible_actions {
            return Some(SelectedAction {
                id: actions.id,
                action: Action::nop(),
            });
        }
        None
    }

    fn action_discard_tile(&mut self) -> Option<SelectedAction> {
        if let Some(e_tile) = self.target_tile
            && let Some(actions) = &self.possible_actions
        {
            for act in &actions.actions {
                if act.action_type == ActionType::Discard
                    && let Some((tile, is_drawn)) = self.hand.find_tile_from_entity(e_tile)
                {
                    self.preferred_discard_tile = Some(tile.entity());
                    return Some(SelectedAction {
                        id: actions.id,
                        action: if is_drawn {
                            Action::nop()
                        } else {
                            Action::discard(tile.tile())
                        },
                    });
                }
            }
        }
        None
    }
}

impl HasEntity for GuiPlayer {
    fn entity(&self) -> Entity {
        self.entity
    }
}
