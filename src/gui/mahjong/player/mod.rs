mod action;
mod action_menu;
mod action_setting_menu;
mod operation;

use bevy::input::ButtonState;

use super::*;

pub use self::action_menu::ActionButton;

pub enum HandMode {
    Camera,
    Close,
    Open,
}

#[derive(Debug, PartialEq, Eq)]
enum TargetState {
    Released,
    Pressed,
    Dragging,
}

#[derive(Debug)]
pub struct GuiPlayer {
    // Event用
    entity: Entity,
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
