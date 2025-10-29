use super::{players_info::PlayersInfo, *};

#[derive(Debug)]
pub struct DrawDialog {
    entity: Entity,
}

impl DrawDialog {
    pub fn new(event: &EventDraw, camera_seat: Seat) -> Self {
        let p = param();

        let draw_str = if event.nagashimangan_scores.iter().any(|score| *score != 0) {
            "流し満貫".into()
        } else {
            event.draw_type.to_string()
        };

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
                    children![create_text(draw_str, 40.0)],
                ));
            })
            .id();

        let score_container = p
            .cmd
            .spawn((
                ChildOf(entity),
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Percent(16.0),
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
            ))
            .id();

        let player_info = PlayersInfo::new(
            camera_seat,
            event.dealer,
            &event.names,
            &event.scores,
            &event.delta_scores,
        );
        player_info.insert(ChildOf(score_container));

        p.cmd.spawn((
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

        Self { entity }
    }
}

impl Dialog for DrawDialog {
    fn handle_event(&mut self, ok_buttons: &mut OkButtonQuery) -> bool {
        if handle_dialog_ok_button(ok_buttons) {
            cmd().entity(self.entity).despawn();
            true
        } else {
            false
        }
    }
}
