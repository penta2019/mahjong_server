use std::{f32::consts::FRAC_PI_2, sync::Mutex};

use bevy::{
    color::palettes::basic::{BLACK, GREEN},
    ecs::system::SystemParam,
    prelude::*,
    // scene::SceneInstanceReady,
};

use super::tile::*;
use crate::{
    gui::util,
    listener::EventRx,
    model::{self, *},
};

#[derive(Resource, Debug)]
struct EventReceiver {
    recv: Mutex<EventRx>,
}

pub struct StagePlugin {
    event_rx: Mutex<Option<EventRx>>,
}

impl StagePlugin {
    pub fn new(event_rx: EventRx) -> Self {
        Self {
            event_rx: Mutex::new(Some(event_rx)),
        }
    }
}

impl Plugin for StagePlugin {
    fn build(&self, app: &mut App) {
        let event_rx = self.event_rx.lock().unwrap().take().unwrap();
        app.insert_resource(EventReceiver {
            recv: Mutex::new(event_rx),
        })
        .add_systems(Update, (read_event, animate_move).chain());
    }
}

fn reparent_tranform(
    child: Entity,
    new_parent: Entity,
    globals: &Query<&'static mut GlobalTransform>,
) -> Transform {
    let child_global = *globals.get(child).unwrap();
    let parent_global = *globals.get(new_parent).unwrap();
    let new_local = parent_global.affine().inverse() * child_global.affine();
    Transform::from_matrix(new_local.into())
}

// 等速移動アニメーション
#[derive(Component, Debug)]
pub struct MoveTo {
    // 移動アニメーションの目標(終了)位置
    target: Vec3,
    // アニメーションの残りフレーム数
    // フレームごとに値を1づつ下げていき, 1/frame_left * (target - 現在位置)つづ移動
    // frame_left == 1のときはtargetをそのまま現在位置にセットしてanimationを終了 (= MoveToを削除)
    frame_left: usize,
}

impl MoveTo {
    fn new(target: Vec3) -> Self {
        Self {
            target,
            frame_left: 60,
        }
    }
}

#[derive(Component, Debug)]
pub struct GuiStage;

#[derive(Component, Debug)]
pub struct GuiPlayer {
    seat: Seat,
}

#[derive(Component, Debug)]
pub struct GuiHand {
    seat: Seat,
    tiles: Vec<(Entity, Tile)>,
    drawn_tile: Option<(Entity, Tile)>,
}

impl GuiHand {
    fn tf_tile(&self, is_drawn: bool) -> Transform {
        Transform::from_xyz(
            GuiTile::WIDTH * self.tiles.len() as f32 + if is_drawn { 0.005 } else { 0. },
            GuiTile::HEIGHT / 2.,
            GuiTile::DEPTH / 2.,
        )
    }

    fn sort_hand(&mut self, commands: &mut Commands) {
        self.tiles.sort_by(|a, b| a.1.cmp(&b.1));
        for (i, (e_tile, _)) in self.tiles.iter().enumerate() {
            commands.entity(*e_tile).insert(MoveTo::new(Vec3::new(
                GuiTile::WIDTH * i as f32,
                GuiTile::HEIGHT / 2.,
                GuiTile::DEPTH / 2.,
            )));
        }
    }
}

#[derive(Component, Debug)]
pub struct GuiDiscard {
    seat: Seat,
    // 捨て牌のVec3は(0., 0., 0.)から開始し横(x)にTILE_WIDTHずつスライドしていく
    // 各行の捨て牌は6個までなのでpos_tiles.len() % 6 == 0のときは
    // xを0.にリセットしてyをTILE_HEIGHTだけマイナス方向にスライド
    // 例外的にリーチ宣言牌とその次の牌(行の先頭は例外)の場合は
    // xのスライドは(TILE_WIDHT + TILE_HEIGHT) / 2.になることに注意
    pos_tiles: Vec<Vec3>,
    riichi_index: Option<usize>,
}

impl GuiDiscard {
    const TILES_IN_ROW: usize = 6;
}

#[derive(Component, Debug)]
pub struct GuiMeld {
    seat: Seat,
}

#[derive(Component, Debug)]
pub struct GuiMeldItem {
    seat: Seat,
}

#[derive(SystemParam)]
struct StageParam<'w, 's> {
    commands: Commands<'w, 's>,
    meshes: ResMut<'w, Assets<Mesh>>,
    materials: ResMut<'w, Assets<StandardMaterial>>,
    asset_server: Res<'w, AssetServer>,
    globals: Query<'w, 's, &'static mut GlobalTransform>,
    stage: Query<'w, 's, (Entity, &'static mut GuiStage)>,
    players: Query<'w, 's, (Entity, &'static mut GuiPlayer)>,
    hands: Query<'w, 's, (Entity, &'static mut GuiHand)>,
    discards: Query<'w, 's, (Entity, &'static mut GuiDiscard)>,
    melds: Query<'w, 's, (Entity, &'static mut GuiMeld)>,
    tiles: Query<'w, 's, (Entity, &'static mut GuiTile)>,
    // for debug
    names: Query<'w, 's, &'static Name>,
    children: Query<'w, 's, &'static Children>,
}

// 関数ではStageParamの変数を個別に借用することができないためマクロを使用
macro_rules! get_mut {
    ($query:expr, $seat:expr) => {
        $query
            .iter_mut()
            .find(|(_, h)| h.seat == $seat)
            .expect(&format!("seat {} not found", $seat))
    };
}

fn animate_move(mut commands: Commands, q_move: Query<(Entity, &mut Transform, &mut MoveTo)>) {
    for (e, mut tf, mut move_to) in q_move {
        let diff_vec = move_to.target - tf.translation;
        tf.translation += 1.0 / move_to.frame_left as f32 * diff_vec;
        move_to.frame_left -= 1;
        if move_to.frame_left == 0 {
            commands.entity(e).remove::<MoveTo>();
        }
    }
}

fn read_event(param: StageParam, event_reader: ResMut<EventReceiver>) {
    if let Ok(ev) = event_reader.recv.lock().unwrap().try_recv() {
        handle_event(param, &ev);
    }
}

fn init_stage(mut param: StageParam) {
    let commands = &mut param.commands;
    for (e_stage, _) in &param.stage {
        commands.entity(e_stage).despawn();
    }

    // Light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            intensity: 10_000_000.,
            range: 100.0,
            shadow_depth_bias: 0.2,
            ..default()
        },
        Transform::from_xyz(8.0, 16.0, 8.0),
    ));

    // stage
    let stage = commands
        .spawn((
            Name::new(format!("Stage")),
            Transform::from_xyz(0., 0., 0.),
            Mesh3d(param.meshes.add(Plane3d::default().mesh().size(0.65, 0.65))),
            MeshMaterial3d(param.materials.add(Color::from(GREEN))),
            GuiStage,
        ))
        .id();

    // info
    commands.spawn((
        Name::new(format!("Info")),
        ChildOf(stage),
        Transform::from_xyz(0., 0.001, 0.),
        Mesh3d(param.meshes.add(Plane3d::default().mesh().size(0.12, 0.12))),
        MeshMaterial3d(param.materials.add(Color::from(BLACK))),
    ));

    for s in 0..SEAT {
        let player = commands
            .spawn((
                Name::new(format!("Player[{s}]")),
                ChildOf(stage),
                Transform {
                    rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_2 * s as f32),
                    ..Default::default()
                },
                GuiPlayer { seat: s },
            ))
            .id();

        commands.spawn((
            Name::new(format!("Hand[{s}]")),
            ChildOf(player),
            Transform::from_xyz(-0.12, 0., 0.21),
            GuiHand {
                seat: s,
                tiles: vec![],
                drawn_tile: None,
            },
        ));

        commands.spawn((
            Name::new(format!("Discard[{s}]")),
            ChildOf(player),
            Transform {
                translation: Vec3::new(-0.05, GuiTile::DEPTH / 2., 0.074),
                rotation: Quat::from_rotation_x(-FRAC_PI_2),
                scale: Vec3::ONE,
            },
            GuiDiscard {
                seat: s,
                pos_tiles: vec![],
                riichi_index: None,
            },
        ));

        commands.spawn((
            Name::new(format!("Meld[{s}]")),
            ChildOf(player),
            GuiMeld { seat: s },
        ));
    }
}

fn handle_event(param: StageParam, event: &model::Event) {
    // for (e_discard, _) in &param.discards {
    //     util::print_hierarchy(e_discard, &param.names, &param.children);
    // }

    match event {
        model::Event::Begin(ev) => event_begin(param, ev),
        model::Event::New(ev) => event_new(param, ev),
        model::Event::Deal(ev) => event_deal(param, ev),
        model::Event::Discard(ev) => event_discard(param, ev),
        model::Event::Meld(ev) => event_meld(param, ev),
        model::Event::Nukidora(ev) => event_nukidora(param, ev),
        model::Event::Dora(ev) => event_dora(param, ev),
        model::Event::Win(ev) => event_win(param, ev),
        model::Event::Draw(ev) => event_draw(param, ev),
        model::Event::End(ev) => event_end(param, ev),
    }
}

fn event_begin(param: StageParam, _event: &model::EventBegin) {
    init_stage(param);
}

fn event_new(mut param: StageParam, event: &model::EventNew) {
    let commands = &mut param.commands;
    for seat in 0..SEAT {
        let (e_hand, mut hand) = get_mut!(param.hands, seat);
        for tile in &event.hands[seat] {
            let e_tile = create_tile(commands, &param.asset_server, *tile);
            commands
                .entity(e_tile)
                .insert((ChildOf(e_hand), hand.tf_tile(false)));
            hand.tiles.push((e_tile, *tile));
            hand.sort_hand(commands);
        }
    }
}

fn event_deal(mut param: StageParam, event: &model::EventDeal) {
    let commands = &mut param.commands;
    let e_tile = create_tile(commands, &param.asset_server, event.tile);
    let (e_hand, mut hand) = get_mut!(&mut param.hands, event.seat);
    param
        .commands
        .entity(e_tile)
        .insert((ChildOf(e_hand), hand.tf_tile(true)));
    hand.drawn_tile = Some((e_tile, event.tile));
}

fn event_discard(mut param: StageParam, event: &model::EventDiscard) {
    let (_, mut hand) = get_mut!(&mut param.hands, event.seat);
    let (e_tile, _) = if event.is_drawn {
        hand.drawn_tile.take().unwrap()
    } else {
        if let Some(pos) = hand.tiles.iter().position(|&(_, tile)| tile == event.tile) {
            hand.tiles.remove(pos)
        } else {
            panic!("{} not found", event.tile);
        }
    };

    let (e_discard, mut discard) = get_mut!(&mut param.discards, event.seat);
    let i_tile = discard.pos_tiles.len();
    if event.is_riichi {
        discard.riichi_index = Some(i_tile);
    }

    let mut pos = if let Some(last) = discard.pos_tiles.last() {
        if i_tile % GuiDiscard::TILES_IN_ROW == 0 {
            let y = -GuiTile::HEIGHT * (i_tile / GuiDiscard::TILES_IN_ROW) as f32;
            Vec3::new(0., y, 0.)
        } else {
            last + Vec3::new(GuiTile::WIDTH, 0., 0.)
        }
    } else {
        Vec3::default()
    };
    let mut rot = Quat::IDENTITY;

    // event.is_riichi == trueの時以外に直前のリーチ宣言牌が鳴きで他家の副露に移動した場合なども含む
    if let Some(riichi_index) = discard.riichi_index {
        // リーチ宣言牌とその次の牌は位置が少しずれる
        if i_tile == riichi_index || i_tile == riichi_index + 1 {
            pos += Vec3::new((GuiTile::HEIGHT - GuiTile::WIDTH) / 2., 0., 0.);
        }
        // リーチ宣言牌を横に倒す
        if i_tile == riichi_index {
            rot = Quat::from_axis_angle(Vec3::Z, FRAC_PI_2);
        }
    }

    let mut tf = reparent_tranform(e_tile, e_discard, &param.globals);
    tf.rotation = rot;

    let commands = &mut param.commands;
    discard.pos_tiles.push(pos);
    // commands.entity(e_discard).add_child(e_tile);
    commands
        .entity(e_tile)
        .insert((ChildOf(e_discard), tf, MoveTo::new(pos)));

    // 手牌の並び替え
    if let Some(t) = hand.drawn_tile.take() {
        hand.tiles.push(t);
    }
    hand.sort_hand(commands);
}

fn event_meld(param: StageParam, event: &model::EventMeld) {}

fn event_nukidora(param: StageParam, event: &model::EventNukidora) {}

fn event_dora(param: StageParam, event: &model::EventDora) {}

fn event_win(param: StageParam, event: &model::EventWin) {}

fn event_draw(param: StageParam, event: &model::EventDraw) {}

fn event_end(param: StageParam, event: &model::EventEnd) {}
