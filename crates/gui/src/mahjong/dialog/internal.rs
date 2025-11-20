use mahjong_core::control::common::{calc_seat_offset, calc_seat_wind};

use super::*;
use crate::mahjong::text::create_text_with_color;

pub fn handle_dialog_ok_button(buttons: &mut OkButtonQuery) -> bool {
    for (iteraction, mut border) in buttons {
        match iteraction {
            Interaction::Pressed => return true,
            Interaction::Hovered => border.set_all(Color::WHITE),
            Interaction::None => border.set_all(Color::BLACK),
        };
    }
    false
}

pub fn create_dialog() -> impl Bundle {
    (
        Node {
            justify_self: JustifySelf::Center,
            align_self: AlignSelf::Center,
            width: Val::Px(640.0),
            height: Val::Px(400.0),
            padding: UiRect::top(Val::Px(8.0)),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::Px(16.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.95)),
        // OKボタン
        children![(
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(12.0),
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            children![(
                OkButton,
                Button,
                Node {
                    width: Val::Px(100.0),
                    height: Val::Px(32.0),
                    border: UiRect::all(Val::Px(1.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BorderColor::all(Color::BLACK),
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.1)),
                children![create_text("OK".into(), 20.0)],
            )],
        )],
    )
}

pub fn create_round_dialog(title: String, sub_title: String, players_info: Entity) -> Entity {
    let cmd = cmd();

    let entity = cmd
        .spawn(create_dialog())
        .with_children(|cmd| {
            cmd.spawn(create_text(title, 40.0));
            cmd.spawn(create_text(sub_title, 30.0));
        })
        .id();

    cmd.spawn((
        ChildOf(entity),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Percent(16.0),
            width: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            ..default()
        },
    ))
    .add_child(players_info);

    entity
}

pub fn create_players_info(
    camera_seat: Seat,
    dealer: Seat,
    names: &[String; SEAT],
    scores: &[Score; SEAT],
    deltas: &[Score; SEAT],
) -> Entity {
    let cmd = cmd();
    let mut cols = vec![];

    let entity = cmd
        .spawn((Node {
            align_self: AlignSelf::FlexStart,
            align_content: AlignContent::Center,
            ..default()
        },))
        .with_children(|cmd| {
            for _ in 0..3 {
                let e = cmd
                    .spawn((Node {
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },))
                    .id();
                cols.push(e);
            }
        })
        .id();

    // カメラがある座席基準のオフセットからseatへの変換配列
    let mut offset_to_seat = [0; SEAT];
    for s in 0..SEAT {
        offset_to_seat[calc_seat_offset(camera_seat, s)] = s;
    }

    for offset_seat in [3, 2, 0, 1] {
        let seat = offset_to_seat[offset_seat];
        let wind = wind_to_char_jp(calc_seat_wind(dealer, seat));
        cmd.spawn(create_player_info(
            wind,
            &names[seat],
            scores[seat],
            deltas[seat],
        ))
        .insert(ChildOf(match offset_seat {
            3 => cols[0], // 左
            1 => cols[2], // 右
            _ => cols[1], // 中央　上(2), 下(0)
        }));
    }

    entity
}

fn create_player_info(wind: char, name: &str, score: Score, delta: Score) -> impl Bundle {
    let font_size = 20.0;
    let delta_str = if delta == 0 {
        "".into()
    } else {
        format!(" {:+}", delta)
    };
    let delta_text = if delta > 0 {
        create_text_with_color(delta_str, font_size, Color::srgb(1.0, 0.0, 0.0))
    } else {
        create_text_with_color(delta_str, font_size, Color::srgb(0.0, 1.0, 0.0))
    };

    (
        Node {
            width: Val::Px(180.0),
            height: Val::Px(60.0),
            border: UiRect::all(Val::Px(1.0)),
            padding: UiRect::left(Val::Px(8.0)),
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
        BorderColor::all(Color::srgba(0.2, 0.2, 0.2, 1.0)),
        children![
            create_text(wind.into(), 32.0),
            (
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(4.0),
                    left: Val::Px(50.0),
                    ..default()
                },
                create_text(name.into(), font_size),
            ),
            (
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(4.0),
                    left: Val::Px(50.0),
                    ..default()
                },
                children![create_text(score.to_string(), font_size), delta_text]
            )
        ],
    )
}
