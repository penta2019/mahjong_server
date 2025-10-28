use mahjong_core::control::common::{calc_seat_offset, calc_seat_wind};

use crate::impl_has_entity;

use super::super::{prelude::*, text::wind_to_char};

#[derive(Debug)]
pub struct PopupPlayerInfo {
    entity: Entity,
}

impl_has_entity!(PopupPlayerInfo);

impl PopupPlayerInfo {
    pub fn new(
        camera_seat: Seat,
        dealer: Seat,
        // names: [String; SEAT],
        // scores: [Score; SEAT],
        // deltas: [Score; SEAT],
    ) -> Self {
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
            let wind = wind_to_char(calc_seat_wind(dealer, seat));
            cmd.spawn((
                ChildOf(match offset_seat {
                    3 => cols[0], // 左
                    1 => cols[2], // 右
                    _ => cols[1], // 中央　上(2), 下(0)
                }),
                Node {
                    width: Val::Px(180.0),
                    height: Val::Px(60.0),
                    border: UiRect::all(Val::Px(1.0)),
                    // justify_content: JustifyContent::Center,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
                BorderColor::all(Color::srgba(0.2, 0.2, 0.2, 1.0)),
                children![create_text(format!("{}({})", wind, seat), 20.0)],
            ));
        }

        Self { entity }
    }
}
