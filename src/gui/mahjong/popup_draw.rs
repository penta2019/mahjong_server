use super::prelude::*;

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
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    children![create_text(event.draw_type.to_string(), 40.0)],
                ));

                // let positions = [];
                // for s in 0..1 {
                //     cmd.spawn((
                //         Node {
                //             position_type: PositionType::Absolute,
                //             top: Val::Percent(30.0),
                //             width: Val::Px(200.0),
                //             height: Val::Px(60.0),
                //             justify_content: JustifyContent::Center,
                //             align_items: AlignItems::Center,
                //             ..default()
                //         },
                //         BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
                //         children![create_text("25000".into(), 24.0)],
                //     ));
                // }
            })
            .id();

        Self { entity }
    }

    pub fn destroy(self) {
        cmd().entity(self.entity).despawn();
    }
}
