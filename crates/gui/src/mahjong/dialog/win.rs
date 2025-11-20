use mahjong_core::control::common::{calc_seat_offset, calc_seat_wind, get_names, get_scores};

use super::{
    super::model::{GuiHand, GuiMeld},
    *,
};
use crate::ui3d::Ui3dTransform;

#[derive(Debug)]
pub struct WinDialog {
    entity: Entity,
    ui3d: Entity,
    hand: GuiHand,
    meld: GuiMeld,
    round: usize,
    dealer: Seat,
    honba: usize,
    names: [String; SEAT],
    scores: [Score; SEAT],
    doras: Vec<Tile>,
    event: EventWin,
    camera_seat: Seat,
    next_win_index: usize,
    is_score_result: bool,
}

impl WinDialog {
    pub fn new(stage: &Stage, event: &EventWin, camera_seat: Seat) -> Self {
        let mut obj = Self {
            entity: cmd().spawn_empty().id(),
            ui3d: cmd().spawn_empty().id(),
            hand: GuiHand::new(),
            meld: GuiMeld::new(),
            round: stage.round,
            dealer: stage.dealer,
            honba: stage.honba,
            names: get_names(stage),
            scores: get_scores(stage),
            doras: stage.doras.clone(),
            event: event.clone(),
            camera_seat,
            next_win_index: 0,
            is_score_result: false,
        };
        obj.show_next_win();
        obj
    }

    fn show_next_win(&mut self) -> bool {
        let cmd = cmd();

        if self.next_win_index >= self.event.contexts.len() {
            return false;
        }
        let ctx = &self.event.contexts[self.next_win_index];
        self.next_win_index += 1;

        let entity = cmd.spawn(create_dialog()).insert(Transform::default()).id();

        // プレイヤー (風,名前)
        let wind = wind_to_char_jp(calc_seat_wind(self.dealer, ctx.seat));
        let name = &self.names[ctx.seat];
        let win_type = if ctx.is_drawn { "ツモ" } else { "ロン" };
        cmd.spawn((
            ChildOf(entity),
            Node {
                margin: UiRect::vertical(Val::Px(8.0)),
                ..default()
            },
            children![create_text(format!("{wind}家 {name} {win_type}"), 32.0)],
        ));

        // 手牌,副露,ドラ,裏ドラ表示領域
        let ui3d_area = cmd
            .spawn((
                ChildOf(entity),
                Node {
                    height: Val::Px(40.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(1.0, 0.0, 0.0, 1.0)),
            ))
            .id();
        let ui3d = cmd
            .spawn(Ui3dTransform::new(
                ui3d_area,
                Quat::from_rotation_y(5.0_f32.to_radians())
                    * Quat::from_rotation_x(10.0_f32.to_radians()),
                Vec3::ONE,
            ))
            .id();
        self.ui3d = ui3d;

        // 手牌
        let hand = &mut self.hand;
        hand.init(ctx.hand.iter().map(|t| GuiTile::ui(*t)).collect());
        hand.align(false);
        hand.deal_tile(GuiTile::ui(ctx.winning_tile), false);

        // 副露
        let meld = &mut self.meld;
        for m_meld in &ctx.melds {
            let (self_tiles, meld_tile) = parse_meld(ctx.seat, m_meld);
            let mut self_tiles: Vec<GuiTile> = self_tiles.iter().map(|t| GuiTile::ui(*t)).collect();
            let meld_tile = meld_tile.map(|(t, i)| (GuiTile::ui(t), i));
            match m_meld.meld_type {
                MeldType::Kakan => {
                    let kanan_tile = self_tiles.pop().unwrap();
                    meld.meld(self_tiles, meld_tile, false); // ポン
                    meld.meld(vec![kanan_tile], None, false); // 加槓
                }
                _ => meld.meld(self_tiles, meld_tile, false),
            }
        }

        // ui3dに追加
        let mut x_offset = -(hand.width() + meld.width()) * 0.5; // 中央を基準に左端を計算
        x_offset += GuiTile::WIDTH * 0.5; // 位置微調整
        hand.insert((
            ChildOf(ui3d),
            Transform::from_xyz(x_offset, -GuiTile::HEIGHT * 0.5, 0.0),
        ));
        x_offset += hand.width() + meld.width() + GuiTile::WIDTH * 0.25;
        meld.insert((ChildOf(ui3d), Transform::from_xyz(x_offset, 0.0, 0.0)));

        // ドラ,裏ドラ 表示領域
        let dora_area = cmd
            .spawn((
                ChildOf(entity),
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(28.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                // BackgroundColor(Color::srgba(0.0, 1.0, 0.0, 1.0)),
            ))
            .id();

        // ドラ表示牌
        cmd.spawn((
            ChildOf(dora_area),
            Node {
                width: Val::Percent(10.0),
                justify_content: JustifyContent::End,
                ..default()
            },
            children![create_text("ドラ".into(), 16.0)],
        ));
        let dora_view = cmd
            .spawn((
                ChildOf(dora_area),
                Node {
                    width: Val::Percent(25.0),
                    ..default()
                },
            ))
            .id();
        cmd.entity(create_dora_view(&self.doras)).insert((
            ChildOf(entity),
            Ui3dTransform::new(
                dora_view,
                Quat::from_rotation_y(5.0_f32.to_radians())
                    * Quat::from_rotation_x(10.0_f32.to_radians()),
                Vec3::ONE,
            ),
        ));

        // 裏ドラ表示牌
        if ctx.is_riichi {
            cmd.spawn((
                ChildOf(dora_area),
                Node {
                    width: Val::Percent(10.0),
                    justify_content: JustifyContent::End,
                    ..default()
                },
                children![create_text("裏ドラ".into(), 16.0)],
            ));
            let ura_dora_view = cmd
                .spawn((
                    ChildOf(dora_area),
                    Node {
                        width: Val::Percent(25.0),
                        ..default()
                    },
                ))
                .id();
            cmd.entity(create_dora_view(&self.event.ura_doras)).insert((
                ChildOf(entity),
                Ui3dTransform::new(
                    ura_dora_view,
                    Quat::from_rotation_y(5.0_f32.to_radians())
                        * Quat::from_rotation_x(10.0_f32.to_radians()),
                    Vec3::ONE,
                ),
            ));
        }

        // ドラ裏ドラ表示牌が中央に来るようにパッディング
        cmd.spawn((
            ChildOf(dora_area),
            Node {
                width: Val::Percent(10.0),
                ..default()
            },
        ));

        let sctx = &ctx.score_context;

        // 役
        cmd.spawn((
            ChildOf(entity),
            Node {
                justify_content: JustifyContent::Center,
                ..default()
            },
        ))
        .add_child(create_yaku_view(&sctx.yakus, sctx.yakuman != 0));

        // 得点
        let title_str = if sctx.yakuman == 0 {
            format!("{}符{}飜 {}", sctx.fu, sctx.fan, sctx.title)
        } else {
            sctx.title.clone()
        };
        let point_str = if ctx.is_drawn {
            // ツモ
            if ctx.is_dealer {
                format!("{}点オール", sctx.points.1)
            } else {
                format!("{}点・{}点", sctx.points.1, sctx.points.2)
            }
        } else {
            // ロン
            format!("{}点", sctx.points.0)
        };
        cmd.spawn((
            ChildOf(entity),
            Node {
                // height: Val::Px(50.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            children![create_text(format!("{} {}", title_str, point_str), 24.0)],
        ));

        self.entity = entity;

        true
    }

    fn show_result(&mut self) -> bool {
        if self.is_score_result {
            return false;
        }
        self.is_score_result = true;

        let players_info = create_players_info(
            self.camera_seat,
            self.dealer,
            &self.names,
            &self.scores,
            &self.event.delta_scores,
        );
        self.entity = create_round_dialog(
            round_string(self.round, self.dealer, Some(self.honba)),
            "".into(),
            players_info,
        );
        self.ui3d = cmd().spawn_empty().id();
        true
    }
}

impl Dialog for WinDialog {
    fn handle_event(&mut self, ok_buttons: &mut OkButtonQuery) -> bool {
        if handle_dialog_ok_button(ok_buttons) {
            cmd().entity(self.entity).despawn();
            cmd().entity(self.ui3d).despawn();
            self.hand = GuiHand::new();
            self.meld = GuiMeld::new();
            if self.show_next_win() {
                return false;
            }
            if self.show_result() {
                return false;
            }
            return true;
        }

        false
    }
}

fn create_dora_view(doras: &[Tile]) -> Entity {
    let cmd = cmd();

    let entity = cmd.spawn_empty().id();

    // 要素が5個(MAX)未満のときは5個になるようにZ8を詰める
    let dora_max = 5;
    let mut doras = doras.to_owned();
    doras.append(&mut vec![Z8; dora_max - usize::min(doras.len(), dora_max)]);

    let doras: Vec<GuiTile> = doras.into_iter().map(GuiTile::ui).collect();
    let mut offset_x = -GuiTile::WIDTH * (usize::max(doras.len(), 1) - 1) as f32 * 0.5;
    for tile in doras {
        tile.insert((
            ChildOf(entity),
            Transform {
                translation: Vec3::new(offset_x, 0.0, 0.0),
                rotation: if tile.tile() == Z8 {
                    Quat::from_rotation_x(PI)
                } else {
                    Quat::IDENTITY
                },
                scale: Vec3::ONE,
            },
        ));
        offset_x += GuiTile::WIDTH;
    }

    entity
}

fn create_yaku_view(yaku: &[Yaku], is_yakuman: bool) -> Entity {
    let cmd = cmd();

    let entity = cmd
        .spawn((Node {
            column_gap: Val::Px(16.0),
            ..default()
        },))
        .id();

    // 役の種類は1列に5種まで. 1~5種(1列), 6~10種(2列), 10~15種(3列)
    let yakus_in_col = 5;
    let mut cols = vec![];
    for _ in 0..((yaku.len() - 1) / yakus_in_col) + 1 {
        // 役名の列
        let col_name = cmd
            .spawn((
                ChildOf(entity),
                Node {
                    width: Val::Px(128.0),
                    flex_direction: FlexDirection::Column,
                    align_items: if is_yakuman {
                        AlignItems::Center
                    } else {
                        AlignItems::FlexStart
                    },
                    ..default()
                },
            ))
            .id();

        // 飜数の列
        let col_fan = cmd
            .spawn((
                ChildOf(entity),
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::FlexEnd, // '飜'の文字が縦に綺麗に揃うように右寄せ
                    ..default()
                },
            ))
            .id();

        cols.push((col_name, col_fan));
    }

    for (i, y) in yaku.iter().enumerate() {
        cmd.spawn(create_text(y.name.clone(), 20.0))
            .insert(ChildOf(cols[i / yakus_in_col].0));
        if !is_yakuman {
            cmd.spawn(create_text(y.fan.to_string() + "飜", 20.0))
                .insert(ChildOf(cols[i / yakus_in_col].1));
        }
    }
    entity
}

// (手牌から出した牌, Option(他家の河から鳴いた牌, 他家のとの位置関係(自家: 0, 上家: 1, 対面: 2, 下家: 3)))
fn parse_meld(self_seat: Seat, meld: &Meld) -> (Vec<Tile>, Option<(Tile, usize)>) {
    let mut tiles = vec![];
    let mut meld_tile = None;
    for (&t, &f) in meld.tiles.iter().zip(meld.froms.iter()) {
        if f == self_seat {
            tiles.push(t);
        } else {
            let meld_offset = calc_seat_offset(self_seat, f);
            meld_tile = Some((t, meld_offset));
        }
    }
    (tiles, meld_tile)
}
