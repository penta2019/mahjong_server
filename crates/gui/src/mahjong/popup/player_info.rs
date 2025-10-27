use crate::impl_has_entity;

use super::super::prelude::*;

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
        let mut rows = vec![];

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
                    rows.push(e);
                }
            })
            .id();

        for s in 0..SEAT {
            cmd.spawn((
                ChildOf(match s {
                    0 => rows[0],
                    3 => rows[2],
                    _ => rows[1],
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
            ));
        }

        Self { entity }
    }
}
