use bevy::color::palettes::basic::{BLACK, GREEN};

use super::*;
use crate::gui::camera::CameraMove;

pub const CAMERA_POS: Vec3 = Vec3::new(0.0, 0.8, 0.8);
pub const CAMERA_LOOK_AT: Vec3 = Vec3::new(0.0, -0.02, 0.0);

#[derive(Resource, Debug)]
pub struct GuiStage {
    entity: Entity,
    players: Vec<GuiPlayer>,
    last_tile: Option<(Seat, ActionType, Tile)>,
    camera_seat: Seat,
    action_control: ActionControl,
    show_hand: bool,
}

impl GuiStage {
    pub fn new() -> Self {
        let param = param();
        let commands = &mut param.commands;
        let meshes = &mut param.meshes;
        let materials = &mut param.materials;

        // stage
        let entity = commands
            .spawn((
                Name::new("Stage".to_string()),
                Transform::from_xyz(0.0, 0.0, 0.0),
                Mesh3d(meshes.add(Plane3d::default().mesh().size(0.65, 0.65))),
                MeshMaterial3d(materials.add(Color::from(GREEN))),
            ))
            .id();

        // info
        // 局全体の情報 (得点, リー棒などのプレイヤーごとの情報はGuiPlayerに置く)
        commands.spawn((
            Name::new("Info".to_string()),
            ChildOf(entity),
            Transform::from_xyz(0.0, 0.001, 0.0),
            Mesh3d(meshes.add(Plane3d::default().mesh().size(0.12, 0.12))),
            MeshMaterial3d(materials.add(Color::from(BLACK))),
        ));

        // Light
        // 斜め4方向から照射
        for i in 0..4 {
            commands.spawn((
                ChildOf(entity),
                DirectionalLight {
                    illuminance: 1_000.0,
                    shadows_enabled: false,
                    ..default()
                },
                Transform::from_translation(
                    Quat::from_rotation_y(FRAC_PI_2 * i as f32) * Vec3::ONE,
                )
                .looking_at(Vec3::new(0.0, 0.1, 0.0), Vec3::new(0.0, 1.0, 0.0)),
            ));
        }

        let mut players = vec![];
        for seat in 0..SEAT {
            let player = GuiPlayer::new();
            commands.entity(player.entity()).insert((
                ChildOf(entity),
                Transform {
                    rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_2 * seat as f32),
                    ..Default::default()
                },
            ));
            players.push(player);
        }

        let action_control = ActionControl::new();

        Self {
            entity,
            players,
            last_tile: None,
            camera_seat: 0,
            action_control,
            show_hand: false,
        }
    }

    pub fn destroy(self) {
        self.action_control.destroy();
        param().commands.entity(self.entity).despawn();
    }

    pub fn set_player(&mut self, seat: Seat) {
        self.camera_seat = seat;
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

    pub fn handle_event(&mut self, event: &MjEvent) {
        self.action_control
            .on_event(&mut self.players[self.camera_seat]);
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

    pub fn handle_gui_events(&mut self) -> Option<SelectedAction> {
        self.action_control
            .handle_gui_events(&mut self.players[self.camera_seat])
    }

    fn event_new(&mut self, event: &EventNew) {
        for seat in 0..SEAT {
            self.players[seat].init_hand(&event.hands[seat]);
        }
    }

    fn event_deal(&mut self, event: &EventDeal) {
        if let Some((seat, ActionType::Discard, _)) = self.last_tile {
            self.players[seat].confirm_discard_tile();
        }
        self.last_tile = None;
        self.players[event.seat].deal_tile(event.tile);
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

    fn event_dora(&mut self, _event: &EventDora) {}

    fn event_win(&mut self, _event: &EventWin) {}

    fn event_draw(&mut self, _event: &EventDraw) {}
}

impl HasEntity for GuiStage {
    fn entity(&self) -> Entity {
        self.entity
    }
}
