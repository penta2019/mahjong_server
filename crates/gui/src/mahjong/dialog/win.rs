use mahjong_core::control::common::{calc_seat_wind, get_names, get_scores};

use super::{
    super::{
        prelude::*,
        text::{round_string, wind_to_char_jp},
    },
    players_info::{create_players_info, create_round_dialog},
    *,
};

#[derive(Debug)]
pub struct WinDialog {
    entity: Entity,
    round: usize,
    dealer: Seat,
    honba: usize,
    names: [String; SEAT],
    scores: [Score; SEAT],
    event: EventWin,
    camera_seat: Seat,
    next_win_index: usize,
    is_score_result: bool,
}

impl WinDialog {
    pub fn new(stage: &Stage, event: &EventWin, camera_seat: Seat) -> Self {
        let mut obj = Self {
            entity: cmd().spawn_empty().id(),
            round: stage.round,
            dealer: stage.dealer,
            honba: stage.honba,
            names: get_names(stage),
            scores: get_scores(stage),
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

        let entity = cmd
            .spawn((
                Node {
                    justify_self: JustifySelf::Center,
                    align_self: AlignSelf::Center,
                    width: Val::Px(600.0),
                    height: Val::Px(400.0),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
            ))
            .id();

        let wind = wind_to_char_jp(calc_seat_wind(self.dealer, ctx.seat));
        let name = &self.names[ctx.seat];
        let win_type = if ctx.is_drawn { "ツモ" } else { "ロン" };
        // プレイヤー (風,名前)
        cmd.spawn((
            ChildOf(entity),
            Node {
                margin: UiRect::vertical(Val::Px(8.0)),
                justify_content: JustifyContent::Center,
                ..default()
            },
            children![create_text(format!("{wind}家 {name} {win_type}"), 32.0)],
        ));

        // 手牌
        cmd.spawn((
            ChildOf(entity),
            Node {
                height: Val::Px(64.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
        ));

        // ドラ,裏ドラ
        cmd.spawn((
            ChildOf(entity),
            Node {
                height: Val::Px(64.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
        ));

        // 役
        cmd.spawn((
            ChildOf(entity),
            Node {
                justify_content: JustifyContent::Center,
                ..default()
            },
        ))
        .add_child(create_yaku_view(&ctx.score_context.yakus));

        // 得点
        let sctx = &ctx.score_context;
        let title_str = if sctx.title.is_empty() {
            format!("{}符{}飜", sctx.fu, sctx.fan)
        } else {
            ctx.score_context.title.clone()
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

        // OKボタン
        cmd.spawn((
            ChildOf(entity),
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(8.0),
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            children![create_ok_button()],
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
        true
    }
}

impl Dialog for WinDialog {
    fn handle_event(&mut self, ok_buttons: &mut OkButtonQuery) -> bool {
        if handle_dialog_ok_button(ok_buttons) {
            cmd().entity(self.entity).despawn();
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

fn create_yaku_view(yaku: &[Yaku]) -> Entity {
    let cmd = cmd();

    let entity = cmd
        .spawn((Node {
            // justify_content: JustifyContent::Center,
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
                    // margin: UiRect::left(Val::Px(32.0)),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
            ))
            .id();

        // 飜数の列
        let col_fan = cmd
            .spawn((
                ChildOf(entity),
                Node {
                    // margin: UiRect::left(Val::Px(16.0)),
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
        cmd.spawn(create_text(y.fan.to_string() + "飜", 20.0))
            .insert(ChildOf(cols[i / yakus_in_col].1));
    }
    entity
}
