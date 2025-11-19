use mahjong_core::control::string::tiles_from_string;

use super::{
    Rx, Tx,
    action::create_tile_set,
    dialog::{Dialog, OkButtonQuery},
    model::GuiStage,
    param::{MahjongParam, with_param},
    plugin::{InfoTexture, setup},
    prelude::*,
    tile_plugin::TilePlugin,
};
use crate::ui3d::{UI3D_LAYER, Ui3dTransform};

#[derive(Resource, Debug, Default)]
pub struct MahjongResource {
    stage: Stage,
    tile: Option<Entity>,
    gui_stage: Option<GuiStage>, // 初期化はwith_paramの内部から行う
    dialog: Option<Box<dyn Dialog>>,
}

pub struct MahjongPlugin {}

impl MahjongPlugin {
    #[allow(unused)]
    pub fn new(_tx: Tx, _rx: Rx) -> Self {
        Self {}
    }
}

impl Plugin for MahjongPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TilePlugin)
            .insert_resource(MahjongResource::default())
            .add_systems(Startup, (setup, test_setup).chain())
            .add_systems(Update, system);
    }
}

fn test_setup(mut param: MahjongParam, mut res: ResMut<MahjongResource>) {
    with_param(&mut param, || {
        res.gui_stage = Some(GuiStage::new());
        res.stage.players[0].score = 25000;
        res.stage.players[1].score = 25000;
        res.stage.players[2].score = 25000;
        res.stage.players[3].score = 25000;
        res.stage.players[0].name = "Player0".into();
        res.stage.players[1].name = "プレイヤー1".into();
        res.stage.players[2].name = "Pl2".into();
        res.stage.players[3].name = "4P".into();

        let camera_seat = 0;
        // res.dialog = Some(Box::new(super::dialog::DrawDialog::new(
        //     &res.stage,
        //     &create_draw_event(),
        //     camera_seat,
        // )));
        res.dialog = Some(Box::new(super::dialog::WinDialog::new(
            &res.stage,
            &create_win_event(),
            camera_seat,
        )));
        // res.dialog = Some(Box::new(super::dialog::EndDialog::new(&res.stage)));

        // let node0 = cmd()
        //     .spawn(Node {
        //         justify_self: JustifySelf::Stretch,
        //         align_self: AlignSelf::Stretch,
        //         ..default()
        //     })
        //     .id();
        // let node = cmd()
        //     .spawn((
        //         ChildOf(node0),
        //         Node {
        //             position_type: PositionType::Absolute,
        //             bottom: Val::Percent(25.0),
        //             right: Val::Percent(25.0),
        //             ..default()
        //         },
        //     ))
        //     .id();
        // let tile_set = create_tile_set(&vec![Tile(TM, 8), Tile(TM, 9)]);
        // res.tile = Some(tile_set);
        // cmd()
        //     .entity(tile_set)
        //     .insert(Ui3dTransform::new(node, Quat::IDENTITY, Vec3::splat(1.0)));
    });
}

fn system(
    mut param: MahjongParam,
    mut res: ResMut<MahjongResource>,
    mut ok_buttons: OkButtonQuery,
) {
    with_param(&mut param, || {
        if let Some(mut dialog) = res.dialog.take()
            && !dialog.handle_event(&mut ok_buttons)
        {
            res.dialog = Some(dialog);
        } else {
            if let Some(t) = res.tile.take() {
                println!("despawn: {t}");
                cmd().entity(t).insert(Transform::from_xyz(0.0, 0.0, 0.0));
            }
        }
    });
}

fn create_draw_event() -> EventDraw {
    EventDraw {
        draw_type: DrawType::Kouhaiheikyoku,
        delta_scores: [12000, -3000, -3000, -3000],
        nagashimangan_scores: [12000, 0, 0, 0],
        hands: [vec![], vec![], vec![], vec![]],
    }
}

fn create_win_event() -> EventWin {
    EventWin {
        // ドラ表示牌
        ura_doras: vec![], // 裏ドラ表示牌                            // プレイヤー名
        delta_scores: [12000, -3000, -3000, -3000], // scores + delta_scores = new_scores
        contexts: vec![
            WinContext {
                seat: 0,
                hand: tiles_from_string("m5").unwrap(),
                winning_tile: Tile(TM, 0),
                melds: vec![
                    Meld {
                        step: 0,
                        meld_type: MeldType::Minkan,
                        tiles: tiles_from_string("m1111").unwrap(),
                        froms: vec![0, 0, 0, 1],
                    },
                    Meld {
                        step: 0,
                        meld_type: MeldType::Kakan,
                        tiles: tiles_from_string("m4444").unwrap(),
                        froms: vec![0, 0, 3, 0],
                    },
                    Meld {
                        step: 0,
                        meld_type: MeldType::Minkan,
                        tiles: tiles_from_string("m2222").unwrap(),
                        froms: vec![0, 0, 0, 2],
                    },
                    Meld {
                        step: 0,
                        meld_type: MeldType::Minkan,
                        tiles: tiles_from_string("m3333").unwrap(),
                        froms: vec![0, 0, 0, 3],
                    },
                ],
                is_dealer: true,
                is_drawn: true,
                pao: None,
                delta_scores: [12000, -3000, -3000, -3000],
                score_context: ScoreContext {
                    yakus: vec![
                        Yaku {
                            name: "立直".into(),
                            fan: 1,
                        },
                        Yaku {
                            name: "平和".into(),
                            fan: 2,
                        },
                        Yaku {
                            name: "門前清自摸和".into(),
                            fan: 3,
                        },
                        Yaku {
                            name: "断么九".into(),
                            fan: 4,
                        },
                        Yaku {
                            name: "ドラ".into(),
                            fan: 1,
                        },
                        Yaku {
                            name: "立直".into(),
                            fan: 1,
                        },
                        Yaku {
                            name: "平和".into(),
                            fan: 1,
                        },
                    ],
                    fu: 30,
                    fan: 5,
                    yakuman: 0,
                    score: 12000,
                    points: (12000, 3000, 0),
                    title: "満貫".into(),
                },
            },
            WinContext {
                seat: 1,
                hand: tiles_from_string("z1112223334445").unwrap(),
                winning_tile: Tile(TZ, 5),
                melds: vec![],
                is_dealer: false,
                is_drawn: true,
                pao: None,
                delta_scores: [12000, -3000, -3000, -3000],
                score_context: ScoreContext {
                    yakus: vec![
                        Yaku {
                            name: "天和".into(),
                            fan: 1,
                        },
                        Yaku {
                            name: "四暗刻単騎".into(),
                            fan: 2,
                        },
                        Yaku {
                            name: "大四喜".into(),
                            fan: 2,
                        },
                        Yaku {
                            name: "字一色".into(),
                            fan: 1,
                        },
                    ],
                    fu: 20,
                    fan: 0,
                    yakuman: 6,
                    score: 32000 * 6,
                    points: (32000 * 6, 8000 * 6, 16000 * 6),
                    title: "六倍役満".into(),
                },
            },
        ],
    }
}
