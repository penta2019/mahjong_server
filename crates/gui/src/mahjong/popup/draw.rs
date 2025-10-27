use crate::mahjong::popup::player_info::PopupPlayerInfo;

use super::super::prelude::*;

#[derive(Debug)]
pub struct PopupDraw {
    entity: Entity,
}

impl PopupDraw {
    pub fn new(event: &EventDraw) -> Self {
        let p = param();

        let entity = p
            .cmd
            .spawn((
                Node {
                    justify_self: JustifySelf::Center,
                    align_self: AlignSelf::Center,
                    width: Val::Px(600.0),
                    height: Val::Px(400.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
            ))
            .with_children(|cmd| {
                cmd.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        top: Val::Percent(10.0),
                        width: Val::Percent(100.0),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    children![create_text(event.draw_type.to_string(), 40.0)],
                ));
            })
            .id();

        let score_container = p
            .cmd
            .spawn((
                ChildOf(entity),
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Percent(60.0),
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
            ))
            .id();

        let player_info = PopupPlayerInfo::new(3, 1);
        player_info.insert(ChildOf(score_container));

        Self { entity }
    }

    pub fn destroy(self) {
        cmd().entity(self.entity).despawn();
    }
}
