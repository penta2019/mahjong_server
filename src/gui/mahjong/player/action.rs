use super::{
    action_menu::{create_main_action_menu, create_sub_action_menu},
    *,
};
use crate::model::ActionType;

const COLOR_ACTIVE: LinearRgba = LinearRgba::rgb(0.0, 0.1, 0.0); // ハイライト (打牌可)
const COLOR_INACTIVE: LinearRgba = LinearRgba::rgb(0.1, 0.0, 0.0); // ハイライト (打牌不可)

impl GuiPlayer {
    pub fn on_event(&mut self) {
        self.clear_actions();
    }

    pub fn handle_actions(&mut self, actions: PossibleActions) {
        let types = ordered_action_types(&actions.actions);
        if !types.is_empty() {
            self.action_main_menu = Some(create_main_action_menu(&types));
        }
        self.possible_actions = Some(actions);
        self.update_target_tile_color();
    }

    pub fn handle_gui_events(&mut self) -> Option<SelectedAction> {
        let selected_act = [
            self.handle_hovered_tile(),
            self.handle_mouse_input(),
            self.handle_action_buttons(),
        ]
        .into_iter()
        .fold(None, |a, b| a.or(b));

        if let Some(act) = selected_act
            && let Some(acts) = self.possible_actions.take()
        {
            self.clear_actions();
            Some(SelectedAction {
                id: acts.id,
                action: act,
            })
        } else {
            None
        }
    }

    fn handle_hovered_tile(&mut self) -> Option<Action> {
        for ev in param().hovered_tile.read() {
            if self.target_state == TargetState::Released {
                self.set_target_tile(ev.tile_entity);
            } else {
                if self.target_tile == ev.tile_entity {
                    continue;
                } else {
                    // 牌を選択して左クリックを押し込んだ状態の場合並び替えを実行
                    self.target_state = TargetState::Dragging;
                    if let Some(target_tile) = self.target_tile
                        && let Some(new_target_tile) = ev.tile_entity
                        && find_tile(self.hand.tiles(), new_target_tile).is_some()
                    {
                        self.hand.move_tile(target_tile, new_target_tile);
                    }
                }
            }
        }
        None
    }

    fn handle_mouse_input(&mut self) -> Option<Action> {
        let mut act = None;
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
                            act = self.action_discard_tile();
                        }
                        self.target_state = TargetState::Released;
                    }
                },
                MouseButton::Right => match ev.state {
                    ButtonState::Pressed => {
                        if self.action_sub_menu.is_some() {
                            self.cancel_sub_menu();
                        } else {
                            act = self.action_nop();
                        }
                    }
                    ButtonState::Released => {}
                },
                _ => {}
            }
        }
        act
    }

    fn handle_action_buttons(&mut self) -> Option<Action> {
        let mut act = None;
        for (interaction, button, mut border_color) in &mut param().action_menu_buttons {
            match *interaction {
                Interaction::Pressed => match &*button {
                    ActionButton::Main(ty) => act = self.on_main_pressed(*ty),
                    ActionButton::Sub(act0) => act = Some(act0.clone()),
                },
                Interaction::Hovered => {
                    border_color.set_all(Color::WHITE);
                }
                Interaction::None => {
                    border_color.set_all(Color::BLACK);
                }
            }
        }
        act
    }

    fn on_main_pressed(&mut self, ty: ActionType) -> Option<Action> {
        // サブメニューのキャンセルボタンの処理
        if self.action_sub_menu.is_some() && ty == ActionType::Nop {
            self.cancel_sub_menu();
            return None;
        }

        let param = param();
        let mut act = None;
        // メインメニューのボタン処理
        if let Some(possible_acts) = &self.possible_actions {
            let mut acts: Vec<_> = possible_acts
                .actions
                .iter()
                .filter(|a| ty == a.ty)
                .cloned()
                .collect();
            match acts.len() {
                0 => {}
                1 => {
                    let act0 = acts.remove(0);
                    if act0.ty == ActionType::Riichi {
                        self.set_riichi(act0.tiles);
                        if let Some(menu) = self.action_main_menu {
                            param.commands.entity(menu).insert(Visibility::Hidden);
                        }
                        // リーチのキャンセルボタンを生成
                        self.action_sub_menu = Some(create_sub_action_menu(&[]));
                    } else {
                        act = Some(act0);
                    }
                }
                2.. => {
                    if let Some(menu) = self.action_main_menu {
                        param.commands.entity(menu).insert(Visibility::Hidden);
                    }
                    self.action_sub_menu = Some(create_sub_action_menu(&acts));
                }
            }
        }
        act
    }

    fn clear_actions(&mut self) {
        self.possible_actions = None;
        self.update_target_tile_color();
        self.set_riichi(vec![]);
        if let Some(menu) = self.action_main_menu.take() {
            param().commands.entity(menu).despawn();
        }
        if let Some(menu) = self.action_sub_menu.take() {
            param().commands.entity(menu).despawn();
        }
    }

    fn set_target_tile(&mut self, tile: Option<Entity>) {
        // 牌が変化していない場合何もしない
        if tile == self.target_tile {
            return;
        }

        // 元々のtarget_tileを解除
        if !self.is_riichi() {
            self.change_target_tile_color(LinearRgba::BLACK);
        } else if let Some(e_tile) = self.target_tile
            && let Some(tile) = find_tile(self.hand.tiles(), e_tile)
            && self.riichi_discard_tiles.contains(&tile.tile())
        {
            self.change_target_tile_color(LinearRgba::BLACK);
        }

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
        if !self.is_riichi() {
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
            self.set_riichi(vec![]);
            param.commands.entity(sub).despawn();
            if let Some(main) = self.action_main_menu {
                param.commands.entity(main).insert(Visibility::Visible);
            }
        }
    }

    fn set_riichi(&mut self, m_tiles: Vec<Tile>) {
        // リーチをキャンセルした場合に色を戻す
        if self.is_riichi() {
            for tile in self.hand.tiles() {
                tile.set_emissive(LinearRgba::BLACK);
            }
        }

        self.riichi_discard_tiles = m_tiles;

        // リーチで宣言牌になれない牌をハイライトする
        if self.is_riichi() {
            for tile in self.hand.tiles() {
                if !self.riichi_discard_tiles.contains(&tile.tile()) {
                    tile.set_emissive(COLOR_INACTIVE);
                }
            }
        }
    }

    fn is_riichi(&self) -> bool {
        !self.riichi_discard_tiles.is_empty()
    }

    fn action_nop(&mut self) -> Option<Action> {
        if self.possible_actions.is_some() {
            return Some(Action::nop());
        }
        None
    }

    fn action_discard_tile(&mut self) -> Option<Action> {
        let (tile, is_drawn) = self.get_target_tile_if_discardable()?;
        let act = if !self.is_riichi() {
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
        Some(act)
    }
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
    .filter(|t| actions.iter().any(|a| a.ty == *t))
    .collect();

    // 打牌可能な場合はNop(Skipボタン)は不要
    if find_discard(actions).is_some() {
        types.retain(|t| *t != ActionType::Nop);
    }

    types
}

fn find_discard(actions: &[Action]) -> Option<&Action> {
    actions.iter().find(|a| a.ty == ActionType::Discard)
}

fn find_tile(tiles: &[GuiTile], entity: Entity) -> Option<&GuiTile> {
    tiles.iter().find(|t| t.entity() == entity)
}
