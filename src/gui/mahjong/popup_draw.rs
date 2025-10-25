use super::prelude::*;

#[derive(Debug)]
pub struct PopupDraw {
    entity: Entity,
}

impl PopupDraw {
    pub fn new(event: &EventDraw) -> Self {
        let p = param();

        let reason = match event.draw_type {
            DrawType::Kouhaiheikyoku => "荒牌平局",
            _ => "Draw",
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
                children![(
                    Node {
                        position_type: PositionType::Absolute,
                        top: Val::Percent(10.0),
                        width: Val::Percent(100.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    children![(
                        Text::new(reason),
                        TextFont {
                            font: p.asset_server.load("font/NotoSerifCJKjp-Regular.otf"),
                            font_size: 30.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                        TextShadow::default(),
                    )]
                )],
            ))
            .id();

        Self { entity }
    }

    pub fn destroy(self) {
        cmd().entity(self.entity).despawn();
    }
}
