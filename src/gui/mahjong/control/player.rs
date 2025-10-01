use bevy::input::ButtonState;

use crate::gui::mahjong::button::crate_action_type_button;

use super::*;

const COLOR_ACTIVE: LinearRgba = LinearRgba::rgb(0.0, 0.1, 0.0); // ハイライト (打牌可)
const COLOR_INACTIVE: LinearRgba = LinearRgba::rgb(0.1, 0.0, 0.0); // ハイライト (打牌不可)
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
    preferred_discard_tile: Option<Entity>,
    target_state: TargetState,
    possible_actions: Option<PossibleActions>,
    // アクションのタイプのみを表示するメインメニュー
    // 選択されたアクションタイプが
    // * 一つしかない場合 -> すぐに実行
    // * 複数存在する場合 -> サブメニューで候補一覧を表示
    action_main_menu: Option<Entity>,
    // 同じActionTypeが複数ある場合の候補一覧またはリーチ時のキャンセルボタンを表示するサブメニュー
    action_sub_menu: Option<Entity>,
    // リーチ宣言牌として可能な打牌一覧, リーチ時以外は空
    riichi_discard_tiles: Vec<Tile>,
}

impl GuiPlayer {
    pub fn new() -> Self {
        let commands = &mut param().commands;

        let entity = commands.spawn(Name::new("Player")).id();

        let tf_close_hand = Transform::from_xyz(-0.12, 0.0, 0.21);
        let hand = GuiHand::new();
        commands
            .entity(hand.entity())
            .insert((ChildOf(entity), tf_close_hand));

        let discard = GuiDiscard::new();
        commands.entity(discard.entity()).insert((
            ChildOf(entity),
            Transform {
                translation: Vec3::new(-0.05, GuiTile::DEPTH / 2.0, 0.074),
                rotation: Quat::from_rotation_x(-FRAC_PI_2),
                scale: Vec3::ONE,
            },
        ));

        let meld = GuiMeld::new();
        commands.entity(meld.entity()).insert((
            ChildOf(entity),
            Transform {
                translation: Vec3::new(0.25, GuiTile::DEPTH / 2.0, 0.22),
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
            preferred_discard_tile: None,
            target_state: TargetState::Released,
            possible_actions: None,
            action_main_menu: None,
            action_sub_menu: None,
            riichi_discard_tiles: vec![],
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
        self.set_target_tile(None); // 手牌から外れる前にハイライトを削除

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
        self.set_target_tile(None); // 手牌から外れる前にハイライトを削除

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
        let action0 = self.handle_hovered_tile();
        let action1 = self.handle_mouse_input();
        let action2 = self.handle_action_type_buttons();

        if let Some(action) = action0.or(action1).or(action2)
            && let Some(actions) = self.possible_actions.take()
        {
            self.clear_actions();
            Some(SelectedAction {
                id: actions.id,
                action,
            })
        } else {
            None
        }
    }

    pub fn handle_actions(&mut self, actions: PossibleActions) {
        let types = ordered_action_types(&actions.actions);
        if !types.is_empty() {
            self.action_main_menu = Some(create_action_menu(&types));
        }
        self.possible_actions = Some(actions);
        self.update_target_tile_color();
    }

    pub fn on_event(&mut self) {
        self.clear_actions();
    }

    fn handle_hovered_tile(&mut self) -> Option<Action> {
        for ev in param().hovered_tile.read() {
            self.set_target_tile(ev.tile_entity);
        }
        None
    }

    fn handle_mouse_input(&mut self) -> Option<Action> {
        let mut action = None;
        for ev in param().mouse_input.read() {
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
                        if self.action_sub_menu.is_some() {
                            self.cancel_sub_menu();
                        } else {
                            action = self.action_nop();
                        }
                    }
                    ButtonState::Released => {}
                },
                _ => {}
            }
        }
        action
    }

    fn handle_action_type_buttons(&mut self) -> Option<Action> {
        let param = param();
        let mut action = None;
        for (interaction, action_button, mut border_color) in &mut param.action_buttons {
            match *interaction {
                Interaction::Pressed => {
                    // サブメニューのキャンセルボタンの処理
                    if self.action_sub_menu.is_some()
                        && action_button.action_type == ActionType::Nop
                    {
                        self.cancel_sub_menu();
                        continue;
                    }

                    // メインメニューのボタン処理
                    if let Some(actions) = &self.possible_actions {
                        let mut actions: Vec<_> = actions
                            .actions
                            .iter()
                            .filter(|a| action_button.action_type == a.action_type)
                            .cloned()
                            .collect();
                        match actions.len() {
                            0 => {}
                            1 => {
                                let action0 = actions.remove(0);
                                if action0.action_type == ActionType::Riichi {
                                    self.riichi_discard_tiles = action0.tiles;
                                    if let Some(root) = self.action_main_menu {
                                        param.commands.entity(root).insert(Visibility::Hidden);
                                    }
                                    // リーチのキャンセルボタンを生成
                                    self.action_sub_menu = Some(create_action_sub_menu(&[]));
                                } else {
                                    action = Some(action0);
                                }
                            }
                            2.. => {
                                if let Some(root) = self.action_main_menu {
                                    param.commands.entity(root).insert(Visibility::Hidden);
                                }
                                self.action_sub_menu = Some(create_action_sub_menu(&actions));
                            }
                        }
                    }
                }
                Interaction::Hovered => {
                    border_color.0 = Color::WHITE;
                }
                Interaction::None => {
                    border_color.0 = Color::BLACK;
                }
            }
        }
        action
    }

    fn clear_actions(&mut self) {
        self.possible_actions = None;
        self.update_target_tile_color();
        self.riichi_discard_tiles = vec![];
        if let Some(root) = self.action_main_menu.take() {
            param().commands.entity(root).despawn();
        }
        if let Some(root) = self.action_sub_menu.take() {
            param().commands.entity(root).despawn();
        }
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

        // 手牌に存在する牌なら新しいtarget_tileに指定
        if let Some(e_tile) = tile
            && find_tile(self.hand.tiles(), e_tile).is_some()
        {
            self.target_tile = tile;
            self.update_target_tile_color();
        }
    }

    fn get_target_tile_if_discardable(&self) -> Option<(&GuiTile, IsDrawn)> {
        let e_tile = self.target_tile?;
        let tile = find_tile(self.hand.tiles(), e_tile)?;
        let is_drawn = self.hand.is_drawn_tile(tile);
        if self.riichi_discard_tiles.is_empty() {
            // 通常時
            let actions = self.possible_actions.as_ref()?;
            let discard = find_discard(&actions.actions)?;
            if !discard.tiles.contains(&tile.tile()) {
                return Some((tile, is_drawn));
            }
        } else if self.riichi_discard_tiles.contains(&tile.tile()) {
            // リーチ時
            return Some((tile, is_drawn));
        }

        None
    }

    fn update_target_tile_color(&mut self) {
        self.change_target_tile_color(if self.get_target_tile_if_discardable().is_some() {
            COLOR_ACTIVE
        } else {
            COLOR_INACTIVE
        });
    }

    fn change_target_tile_color(&self, color: LinearRgba) {
        if let Some(e_tile) = self.target_tile
            && let Some(tile) = find_tile(self.hand.tiles(), e_tile)
        {
            tile.set_emissive(color);
        }
    }

    fn cancel_sub_menu(&mut self) {
        if let Some(sub) = self.action_sub_menu.take() {
            let param = param();
            self.riichi_discard_tiles = vec![];
            param.commands.entity(sub).despawn();
            if let Some(main) = self.action_main_menu {
                param.commands.entity(main).insert(Visibility::Visible);
            }
        }
    }

    fn action_nop(&mut self) -> Option<Action> {
        if self.possible_actions.is_some() {
            return Some(Action::nop());
        }
        None
    }

    fn action_discard_tile(&mut self) -> Option<Action> {
        let (tile, is_drawn) = self.get_target_tile_if_discardable()?;
        let action = if self.riichi_discard_tiles.is_empty() {
            if is_drawn {
                Action::nop()
            } else {
                Action::discard(tile.tile())
            }
        } else {
            if is_drawn {
                Action::riichi_drawn()
            } else {
                Action::riichi(tile.tile())
            }
        };

        self.preferred_discard_tile = Some(tile.entity());
        Some(action)
    }
}

impl HasEntity for GuiPlayer {
    fn entity(&self) -> Entity {
        self.entity
    }
}

fn create_action_menu(action_types: &[ActionType]) -> Entity {
    let param = param();
    let root = param
        .commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            right: Val::Percent(20.0),
            bottom: Val::Percent(18.0),
            display: Display::Flex,
            flex_direction: FlexDirection::RowReverse,
            align_items: AlignItems::Center,
            ..default()
        })
        .id();

    for action_type in action_types {
        param
            .commands
            .spawn(crate_action_type_button(
                *action_type,
                &format!("{:?}", *action_type),
            ))
            .insert(ChildOf(root));
    }

    root
}

fn create_action_sub_menu(actions: &[Action]) -> Entity {
    let param = param();
    let root = param
        .commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            right: Val::Percent(20.0),
            bottom: Val::Percent(18.0),
            display: Display::Flex,
            flex_direction: FlexDirection::RowReverse,
            align_items: AlignItems::Center,
            ..default()
        })
        .id();

    param
        .commands
        .spawn(crate_action_type_button(ActionType::Nop, "Cancel"))
        .insert(ChildOf(root));

    // for action in actions {
    //     param
    //         .commands
    //         .spawn(crate_action_button(
    //             *action,
    //             &format!("{:?}", *action),
    //         ))
    //         .insert(ChildOf(root));
    // }

    root
}

fn ordered_action_types(actions: &[Action]) -> Vec<ActionType> {
    use ActionType::*;
    let mut types: Vec<_> = [
        Nop,
        // turn action
        // Discard,
        Tsumo,
        Riichi,
        Ankan,
        Kakan,
        Kyushukyuhai,
        Nukidora,
        // call action
        Ron,
        Chi,
        Pon,
        Minkan,
    ]
    .into_iter()
    .filter(|t| actions.iter().any(|a| a.action_type == *t))
    .collect();

    // 打牌可能な場合はNop(Skipボタン)は不要
    if find_discard(actions).is_some() {
        types.retain(|t| *t != ActionType::Nop);
    }

    types
}

fn find_discard(actions: &[Action]) -> Option<&Action> {
    actions
        .iter()
        .find(|a| a.action_type == ActionType::Discard)
}

fn find_tile(tiles: &[GuiTile], entity: Entity) -> Option<&GuiTile> {
    tiles.iter().find(|t| t.entity() == entity)
}
