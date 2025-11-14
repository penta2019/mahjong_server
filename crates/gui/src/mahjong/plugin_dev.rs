use super::{
    Rx, Tx,
    dialog::{Dialog, OkButtonQuery},
    model::GuiStage,
    param::{MahjongParam, with_param},
    plugin::{InfoTexture, UI3D_LAYER, setup},
    prelude::*,
    tile_plugin::TilePlugin,
};

#[derive(Resource, Debug, Default)]
pub struct MahjongResource {
    stage: Stage,
    tile: Option<GuiTile>,
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
        // res.dialog = Some(Box::new(super::dialog::WinDialog::new(
        //     &res.stage,
        //     &create_win_event(),
        //     camera_seat,
        // )));
        res.dialog = Some(Box::new(super::dialog::EndDialog::new(&res.stage)));

        // GuiTile::new(Tile(TM, 9)).insert(PropagateRenderLayer::new(&RenderLayers::layer(1)));
        // GuiTile::new(Tile(TM, 9));
        GuiTile::with_layer(Tile(TM, 9), &UI3D_LAYER).insert(Transform::from_xyz(0.0, 0.0, 0.0));
        GuiTile::with_layer(Tile(TM, 9), &UI3D_LAYER).insert(Transform::from_xyz(-0.021, 0.0, 0.0));
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
                hand: vec![],
                winning_tile: Tile(TM, 1),
                melds: vec![],
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
                hand: vec![],
                winning_tile: Tile(TM, 1),
                melds: vec![],
                is_dealer: false,
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
                            name: "門前清自摸和".into(),
                            fan: 1,
                        },
                        Yaku {
                            name: "平和".into(),
                            fan: 1,
                        },
                    ],
                    fu: 20,
                    fan: 3,
                    yakuman: 0,
                    score: 5200,
                    points: (5200, 1300, 2600),
                    title: "".into(),
                },
            },
        ],
    }
}
