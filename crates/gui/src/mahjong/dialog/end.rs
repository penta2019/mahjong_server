use super::*;

#[derive(Debug)]
pub struct EndDialog {
    entity: Entity,
}

impl EndDialog {
    pub fn new(stage: &Stage) -> Self {
        let cmd = cmd();

        let entity = cmd
            .spawn((
                Node {
                    justify_self: JustifySelf::Center,
                    align_self: AlignSelf::Center,
                    width: Val::Px(600.0),
                    height: Val::Px(400.0),
                    padding: UiRect::top(Val::Px(8.0)),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(16.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
                children![create_text("終局".into(), 40.0)],
            ))
            .id();

        let mut players: Vec<(usize, String)> = stage
            .players
            .iter()
            .map(|pl| (pl.rank + 1, pl.name.to_owned()))
            .collect();
        players.sort();
        cmd.spawn((
            ChildOf(entity),
            Node {
                margin: UiRect::top(Val::Px(20.0)),
                flex_direction: FlexDirection::Column,
                ..default()
            },
        ))
        .with_children(|cmd| {
            for (rank, name) in players {
                cmd.spawn(create_text(format!("{}位    {}", rank, name), 25.0));
            }
        });

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

        Self { entity }
    }
}

impl Dialog for EndDialog {
    fn handle_event(&mut self, ok_buttons: &mut OkButtonQuery) -> bool {
        if handle_dialog_ok_button(ok_buttons) {
            cmd().entity(self.entity).despawn();
            true
        } else {
            false
        }
    }
}
