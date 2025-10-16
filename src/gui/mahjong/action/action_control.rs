use std::collections::HashMap;

use bevy::input::ButtonState;

use super::{
    super::{
        hand::{GuiHand, IsDrawn},
        player::GuiPlayer,
        prelude::*,
    },
    BUTTON_ACTIVE, BUTTON_INACTIVE, GameButton,
    action_menu::{create_main_action_menu, create_sub_action_menu},
    auto_menu::create_auto_menu,
};
use crate::{gui::mahjong::action::AutoButton, model::ActionType};

use ActionType::*;

#[derive(Debug, PartialEq, Eq)]
enum TargetState {
    Released,
    Pressed,
    Dragging,
}

// このマクロで&mutを複数作成する場合には借用チェッカー正しく機能しないので注意して使用すること
macro_rules! hand {
    ($s:expr) => {
        unsafe { (**$s.player.as_ref().unwrap()).hand() }
    };
}

#[derive(Debug)]
pub struct ActionControl {
    // イベント処理のpub関数が呼ばれた際にSomeをセットして抜ける際にNoneに戻す
    player: Option<*mut GuiPlayer>,

    // auto
    auto_reset_request: bool,
    auto_menu: Entity,
    auto_flags: HashMap<AutoButton, bool>,

    // action
    target_tile: Option<Entity>,
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

unsafe impl Send for ActionControl {} // *mut GuiPlayer用
unsafe impl Sync for ActionControl {} // *mut GuiPlayer用

impl ActionControl {
    pub fn new() -> Self {
        Self {
            player: None,
            auto_reset_request: false,
            auto_menu: create_auto_menu(),
            auto_flags: HashMap::new(),
            target_tile: None,
            target_state: TargetState::Released,
            possible_actions: None,
            action_main_menu: None,
            action_sub_menu: None,
            riichi_discard_tiles: vec![],
        }
    }

    pub fn destroy(self) {
        for e in [
            Some(self.auto_menu),
            self.action_main_menu,
            self.action_sub_menu,
        ]
        .into_iter()
        .flatten()
        {
            param().cmd.entity(e).despawn();
        }
    }

    pub fn handle_event(&mut self, player: &mut GuiPlayer, event: &MjEvent) {
        self.player = Some(player as *mut GuiPlayer);

        self.clear_actions();

        // Newが届いた時点ではstageの生成がQueryに反映されていないため
        // 次のイベントでフラグのリセット処理を実行
        // TODO: この方法は同じUpdate内でeventを行う場合正しく動作しない
        if self.auto_reset_request {
            self.auto_reset_request = false;
            self.set_auto_flag(AutoButton::Discard, false);
            self.set_auto_flag(AutoButton::Sort, true);
            self.set_auto_flag(AutoButton::Win, false);
            self.set_auto_flag(AutoButton::Skip, false);
        }

        if let MjEvent::New(_) = event {
            self.auto_reset_request = true
        }

        self.player = None;
    }

    pub fn handle_actions(&mut self, player: &mut GuiPlayer, actions: PossibleActions) {
        self.player = Some(player as *mut GuiPlayer);

        let types = ordered_action_types(&actions.actions);
        if !types.is_empty() {
            self.action_main_menu = Some(create_main_action_menu(&types));
        }
        self.possible_actions = Some(actions);
        self.update_target_tile_color();

        self.player = None;
    }

    pub fn handle_gui_events(&mut self, player: &mut GuiPlayer) -> Option<SelectedAction> {
        self.player = Some(player as *mut GuiPlayer);

        let selected_act = [
            self.handle_hovered_tile(),
            self.handle_mouse_input(),
            self.handle_action_buttons(),
            self.handle_auto_action(),
        ]
        .into_iter()
        .fold(None, |a, b| a.or(b));

        let selected_act = if let Some(act) = selected_act
            && let Some(acts) = self.possible_actions.take()
        {
            if is_valid_action(&acts.actions, &act, hand!(self)) {
                self.clear_actions();
                Some(SelectedAction {
                    id: acts.id,
                    action: act,
                })
            } else {
                error!("`{act:?}` is not valid action");
                None
            }
        } else {
            None
        };

        self.player = None;

        selected_act
    }

    fn handle_hovered_tile(&mut self) -> Option<Action> {
        for ev in param().hovered_tile.read() {
            if self.target_state == TargetState::Released {
                self.set_target_tile(ev.entity);
            } else {
                if self.target_tile == ev.entity {
                    continue;
                } else {
                    // 牌を選択して左クリックを押し込んだ状態の場合並び替えを実行
                    self.target_state = TargetState::Dragging;
                    if let Some(target_tile) = self.target_tile
                        && let Some(new_target_tile) = ev.entity
                        && find_tile(hand!(self).tiles(), new_target_tile).is_some()
                    {
                        if self.auto_flags.get(&AutoButton::Sort) == Some(&true) {
                            self.set_auto_flag(AutoButton::Sort, false);
                        }
                        hand!(self).move_tile(target_tile, new_target_tile);
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
        let p = param();
        let mut act = None;
        for (entity, interaction) in &mut p.button_interaction {
            let Ok((button, mut border, mut background)) = p.game_buttons.get_mut(entity) else {
                continue;
            };

            match *interaction {
                Interaction::Pressed => match &*button {
                    GameButton::Main(ty) => act = self.on_main_pressed(*ty),
                    GameButton::Sub(act0) => act = Some(act0.clone()),
                    GameButton::Auto(auto) => {
                        let flag = self.auto_flags.entry(*auto).or_insert(false);
                        *flag ^= true;
                        *background = if *flag {
                            BUTTON_ACTIVE.into()
                        } else {
                            BUTTON_INACTIVE.into()
                        };

                        if auto == &AutoButton::Sort {
                            hand!(self).set_sort(*flag)
                        }
                    }
                },
                Interaction::Hovered => {
                    border.set_all(Color::WHITE);
                }
                Interaction::None => {
                    border.set_all(Color::BLACK);
                }
            }
        }
        act
    }

    fn handle_auto_action(&mut self) -> Option<Action> {
        if let Some(acts) = &self.possible_actions {
            let acts = &acts.actions;

            if self.auto_flags.get(&AutoButton::Discard) == Some(&true)
                && has_action(acts, Discard)
                && has_action(acts, Nop)
                && !has_action(acts, Tsumo)
            {
                return Some(Action::nop());
            }

            if self.auto_flags.get(&AutoButton::Win) == Some(&true) {
                if has_action(acts, Tsumo) {
                    return Some(Action::tsumo());
                }
                if has_action(acts, Ron) {
                    return Some(Action::ron());
                }
            }

            if self.auto_flags.get(&AutoButton::Skip) == Some(&true)
                && !has_action(acts, Discard)
                && has_action(acts, Nop)
                && !has_action(acts, Ron)
            {
                return Some(Action::nop());
            }
        }
        None
    }

    fn set_auto_flag(&mut self, target: AutoButton, flag: bool) {
        for (button, _, mut background) in &mut param().game_buttons {
            let GameButton::Auto(auto_button) = *button else {
                continue;
            };

            if target == auto_button {
                self.auto_flags.insert(auto_button, flag);
                *background = if flag {
                    BUTTON_ACTIVE.into()
                } else {
                    BUTTON_INACTIVE.into()
                };
            }
        }
    }

    fn on_main_pressed(&mut self, ty: ActionType) -> Option<Action> {
        // サブメニューのキャンセルボタンの処理
        if self.action_sub_menu.is_some() && ty == Nop {
            self.cancel_sub_menu();
            return None;
        }

        let p = param();
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
                    if act0.ty == Riichi {
                        self.set_riichi(act0.tiles);
                        if let Some(menu) = self.action_main_menu {
                            p.cmd.entity(menu).insert(Visibility::Hidden);
                        }
                        // リーチのキャンセルボタンを生成
                        self.action_sub_menu = Some(create_sub_action_menu(&[]));
                    } else {
                        act = Some(act0);
                    }
                }
                2.. => {
                    if let Some(menu) = self.action_main_menu {
                        p.cmd.entity(menu).insert(Visibility::Hidden);
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
            param().cmd.entity(menu).despawn();
        }
        if let Some(menu) = self.action_sub_menu.take() {
            param().cmd.entity(menu).despawn();
        }
    }

    fn set_target_tile(&mut self, tile: Option<Entity>) {
        // 牌が変化していない場合何もしない
        if tile == self.target_tile {
            return;
        }

        // 元々のtarget_tileを解除
        if !self.is_riichi() {
            self.change_target_tile_color(GuiTile::NORMAL);
        } else if let Some(e_tile) = self.target_tile
            && let Some(tile) = find_tile(hand!(self).tiles(), e_tile)
            && self.riichi_discard_tiles.contains(&tile.tile())
        {
            self.change_target_tile_color(GuiTile::NORMAL);
        }

        self.target_tile = None;
        self.target_state = TargetState::Released;

        // 手牌に存在する牌なら新しいtarget_tileに指定
        if let Some(e_tile) = tile
            && find_tile(hand!(self).tiles(), e_tile).is_some()
        {
            self.target_tile = tile;
            self.update_target_tile_color();
        }
    }

    fn get_target_tile_if_discardable(&self) -> Option<(&GuiTile, IsDrawn)> {
        let e_tile = self.target_tile?;
        let tile = find_tile(hand!(self).tiles(), e_tile)?;
        let is_drawn = hand!(self).is_drawn_tile(tile);
        if !self.is_riichi() {
            // 通常時
            let actions = self.possible_actions.as_ref()?;
            let discard = &actions.actions.iter().find(|a| a.ty == Discard)?;
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
            GuiTile::ACTIVE
        } else {
            GuiTile::INACTIVE
        });
    }

    fn change_target_tile_color(&self, color: LinearRgba) {
        if let Some(e_tile) = self.target_tile
            && let Some(tile) = find_tile(hand!(self).tiles(), e_tile)
        {
            tile.blend(color);
        }
    }

    fn cancel_sub_menu(&mut self) {
        if let Some(sub) = self.action_sub_menu.take() {
            let p = param();
            self.set_riichi(vec![]);
            p.cmd.entity(sub).despawn();
            if let Some(main) = self.action_main_menu {
                p.cmd.entity(main).insert(Visibility::Visible);
            }
        }
    }

    fn set_riichi(&mut self, m_tiles: Vec<Tile>) {
        // リーチをキャンセルした場合に色を戻す
        if self.is_riichi() {
            for tile in hand!(self).tiles() {
                tile.blend(GuiTile::NORMAL);
            }
        }

        self.riichi_discard_tiles = m_tiles;

        // リーチで宣言牌になれない牌をハイライトする
        if self.is_riichi() {
            for tile in hand!(self).tiles() {
                if !self.riichi_discard_tiles.contains(&tile.tile()) {
                    tile.blend(GuiTile::INACTIVE);
                }
            }
        }
    }

    fn is_riichi(&self) -> bool {
        !self.riichi_discard_tiles.is_empty()
    }

    fn action_nop(&mut self) -> Option<Action> {
        if let Some(acts) = &self.possible_actions
            && has_action(&acts.actions, Nop)
        {
            Some(Action::nop())
        } else {
            None
        }
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

        hand!(self).set_preferred_tile(tile.entity());
        Some(act)
    }
}

fn ordered_action_types(actions: &[Action]) -> Vec<ActionType> {
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
    if has_action(actions, Discard) {
        types.retain(|t| *t != Nop);
    }

    types
}

fn has_action(actions: &[Action], ty: ActionType) -> bool {
    actions.iter().any(|a| a.ty == ty)
}

fn find_tile(tiles: &mut [GuiTile], entity: Entity) -> Option<&mut GuiTile> {
    tiles.iter_mut().find(|t| t.entity() == entity)
}

fn is_valid_action(actions: &[Action], action: &Action, hand: &mut GuiHand) -> bool {
    match action.ty {
        Discard => {
            if let Some(discard) = actions.iter().find(|a| a.ty == Discard)
                && let Some(tile) = action.tiles.first()
                && !discard.tiles.contains(tile)
                && hand.tiles().iter().any(|t| t.tile() == *tile)
            {
                true
            } else {
                false
            }
        }
        Riichi => {
            if let Some(riichi) = actions.iter().find(|a| a.ty == Riichi) {
                if let Some(riichi_tile) = action.tiles.first()
                    && riichi.tiles.contains(riichi_tile)
                {
                    // ツモ切り以外のリーチ
                    true
                } else if action.tiles.is_empty()
                    && let Some(tile) = hand.drawn_tile()
                    && riichi.tiles.contains(&tile.tile())
                {
                    // ツモ切りリーチ
                    true
                } else {
                    false
                }
            } else {
                false
            }
        }
        _ => actions.contains(action),
    }
}
