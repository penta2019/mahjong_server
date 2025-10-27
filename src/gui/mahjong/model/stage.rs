use bevy::color::palettes::basic::GREEN;

use super::{
    super::{
        action::{ActionControl, ActionParam},
        prelude::*,
        setting::{Setting, SettingParam, SettingProps},
    },
    player::{GuiPlayer, HandMode},
    stage_info::StageInfo,
    wall::Wall,
};
use crate::gui::camera::CameraMove;

pub const CAMERA_POS: Vec3 = Vec3::new(0.0, 0.9, 0.8);
pub const CAMERA_LOOK_AT: Vec3 = Vec3::new(0.0, -0.04, 0.0);

#[derive(Resource, Debug)]
pub struct GuiStage {
    // stage Entity
    // 殆どのEntityはこのEntityの子孫なので,これをdespawn()すればほぼ消える
    entity: Entity,
    // 中央情報パネル
    info: StageInfo,
    // 牌山
    wall: Wall,
    // 各プレイヤー (手牌, 河, 副露)
    players: Vec<GuiPlayer>,
    // 副露に使用する最後に河に切られた牌
    last_tile: Option<(Seat, ActionType, Tile)>,
    // プレイ可能な場合のプレイヤーの座席
    player_seat: Option<Seat>,
    // カメラを配置するプレイヤーの座席
    camera_seat: Seat,
    // カメラ座席の操作
    action_control: ActionControl,
    // 卓の設定
    setting: Setting,
    // カメラ座席以外のプレイヤーの手牌の表示フラグ
    show_hand: bool,
}
crate::impl_has_entity!(GuiStage);

impl GuiStage {
    pub fn new() -> Self {
        let p = param();

        let entity = p
            .cmd
            .spawn((
                Name::new("Stage".to_string()),
                Mesh3d(p.meshes.add(Plane3d::default().mesh().size(0.65, 0.65))),
                MeshMaterial3d(p.materials.add(Color::from(GREEN))),
                Transform::from_xyz(0.0, 0.0, 0.0),
            ))
            .id();

        // Light
        // 斜め4方向から照射 (牌はこれらのライトを無視してシェーダで独自に行う)
        for i in 0..4 {
            p.cmd.spawn((
                DirectionalLight {
                    illuminance: 1_000.0,
                    shadows_enabled: false,
                    ..default()
                },
                ChildOf(entity),
                Transform::from_translation(
                    Quat::from_rotation_y(FRAC_PI_2 * i as f32) * Vec3::ONE,
                )
                .looking_at(Vec3::new(0.0, 0.1, 0.0), Vec3::new(0.0, 1.0, 0.0)),
            ));
        }

        let info = StageInfo::new();
        info.insert((ChildOf(entity), Transform::from_xyz(0.0, 0.001, 0.0)));

        let mut wall = Wall::new();
        wall.insert((ChildOf(entity), Transform::IDENTITY));
        wall.set_show(true);

        let mut players = vec![];
        for seat in 0..SEAT {
            let player = GuiPlayer::new();
            player.insert((
                ChildOf(entity),
                Transform::from_rotation(Quat::from_rotation_y(
                    std::f32::consts::FRAC_PI_2 * seat as f32,
                )),
            ));
            players.push(player);
        }

        let action_control = ActionControl::new();

        let setting = Setting::new();

        Self {
            entity,
            info,
            wall,
            players,
            last_tile: None,
            player_seat: None,
            camera_seat: 0,
            action_control,
            setting,
            show_hand: true,
        }
    }

    pub fn destroy(self) {
        self.despawn();
        self.info.destroy();
        self.action_control.destroy();
        self.setting.destroy();
    }

    pub fn set_player(&mut self, seat: Seat) {
        assert!(seat < SEAT);
        self.player_seat = Some(seat);
    }

    pub fn get_setting_props(&self) -> &SettingProps {
        self.setting.get_props()
    }

    pub fn set_setting_props(&mut self, props: SettingProps) {
        self.setting.set_props(props);
        self.apply_props();
    }

    pub fn handle_event(&mut self, event: &MjEvent) {
        self.action_control
            .handle_event(&mut self.players[self.camera_seat], event);
        match event {
            MjEvent::Begin(_ev) => {}
            MjEvent::New(ev) => self.event_new(ev),
            MjEvent::Deal(ev) => self.event_deal(ev),
            MjEvent::Discard(ev) => self.event_discard(ev),
            MjEvent::Meld(ev) => self.event_meld(ev),
            MjEvent::Nukidora(ev) => self.event_nukidora(ev),
            MjEvent::Dora(ev) => self.event_dora(ev),
            MjEvent::Win(ev) => self.event_win(ev),
            MjEvent::Draw(ev) => self.event_draw(ev),
            MjEvent::End(_ev) => {}
        }
    }

    pub fn handle_actions(&mut self, actions: PossibleActions) {
        self.action_control
            .handle_actions(&mut self.players[self.camera_seat], actions);
    }

    pub fn handle_action_events(
        &mut self,
        action_param: &mut ActionParam,
    ) -> Option<SelectedAction> {
        if let Some(seat) = self.player_seat
            && seat == self.camera_seat
        {
            self.action_control.set_visibility(true);
            self.action_control
                .handle_gui_events(action_param, &mut self.players[self.camera_seat])
        } else {
            self.action_control.set_visibility(false);
            None
        }
    }

    pub fn handle_setting_events(&mut self, setting_param: &mut SettingParam) {
        if self.setting.handle_gui_events(setting_param) {
            self.apply_props();
        }
    }

    fn apply_props(&mut self) {
        let props = self.setting.get_props();
        self.wall.set_show(props.show_wall);
        self.show_hand = props.show_hand;
        let seat = props.camera_seat; // propsの借用回避
        self.set_camera_seat(seat);
    }

    fn set_camera_seat(&mut self, seat: Seat) {
        assert!(seat < SEAT);
        self.camera_seat = seat;

        let mut props = self.setting.get_props().clone();
        props.camera_seat = seat;
        self.setting.set_props(props);

        self.info.set_camera_seat(seat);

        let pos = Quat::from_rotation_y(FRAC_PI_2 * seat as f32) * CAMERA_POS;
        param().camera.write(CameraMove::look(pos, CAMERA_LOOK_AT));

        for (s, player) in self.players.iter_mut().enumerate() {
            if s == seat {
                player.set_hand_mode(HandMode::Camera);
            } else if self.show_hand {
                player.set_hand_mode(HandMode::Open);
            } else {
                player.set_hand_mode(HandMode::Close);
            }
        }
    }

    fn event_new(&mut self, event: &EventNew) {
        self.info.init(event);
        self.info.set_camera_seat(self.camera_seat);

        self.wall.init(event);
        for dora in &event.doras {
            self.wall.add_dora(*dora);
        }

        for seat in 0..SEAT {
            self.players[seat].init_hand(&event.hands[seat]);
        }
    }

    fn event_deal(&mut self, event: &EventDeal) {
        if let Some((seat, ActionType::Discard, _)) = self.last_tile.take() {
            self.players[seat].confirm_discard_tile();
        }

        let mut tile = self.wall.take_tile(event.is_replacement);
        if tile.tile() != Z8 && event.tile != Z8 {
            assert!(tile.tile() == event.tile);
        }
        if tile.tile() == Z8 && event.tile != Z8 {
            tile.mutate(event.tile);
        }

        self.players[event.seat].deal_tile(tile);
    }

    fn event_discard(&mut self, event: &EventDiscard) {
        self.players[event.seat].discard_tile(event.tile, event.is_drawn, event.is_riichi);
        self.last_tile = Some((event.seat, ActionType::Discard, event.tile));
    }

    fn event_meld(&mut self, event: &EventMeld) {
        // 鳴いたプレイヤーから半時計回りに見た牌を捨てたプレイヤーの座席
        // 自身(0), 下家(1), 対面(2), 上家(3)
        let mut meld_offset = 0;

        // 他家が捨てた牌
        let meld_tile = match event.meld_type {
            MeldType::Chi | MeldType::Pon | MeldType::Minkan => {
                let target_seat = self.last_tile.unwrap().0;
                meld_offset = (target_seat + SEAT - event.seat) % SEAT;
                Some(self.players[target_seat].take_last_discard_tile())
            }
            _ => None,
        };

        self.players[event.seat].meld(&event.consumed, meld_tile, meld_offset);
    }

    fn event_nukidora(&mut self, _event: &EventNukidora) {}

    fn event_dora(&mut self, event: &EventDora) {
        self.wall.add_dora(event.tile);
    }

    fn event_win(&mut self, _event: &EventWin) {}

    fn event_draw(&mut self, _event: &EventDraw) {}
}
