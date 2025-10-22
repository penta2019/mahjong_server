use super::prelude::*;

#[derive(Component, Debug)]
pub enum SettingButton {
    Root,
}

#[derive(Debug)]
pub struct Setting {
    entity: Entity,
}

impl Setting {
    pub fn new() -> Self {
        let p = param();
        let entity = p
            .cmd
            .spawn((
                SettingButton::Root,
                Button,
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Px(100.0),
                    height: Val::Px(40.0),
                    bottom: Val::Percent(0.0),
                    left: Val::Percent(10.0),
                    border: UiRect::all(Val::Px(1.0)),
                    margin: UiRect::horizontal(Val::Px(5.0)),
                    // 内部のテキストを中央に表示(横方向)
                    justify_content: JustifyContent::Center,
                    // 内部のテキストを中央に表示(縦方向)
                    align_items: AlignItems::Center,
                    ..default()
                },
                BorderRadius::all(Val::Px(4.0)),
                BorderColor::all(Color::BLACK),
                BackgroundColor(Color::BLACK),
                children![(
                    Text::new("Settings"),
                    TextFont {
                        // font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    TextShadow::default(),
                )],
            ))
            .id();

        // p.cmd.spawn();

        Self { entity }
    }

    pub fn destroy(self) {
        cmd().entity(self.entity).despawn();
    }

    // pub fn handle_gui_events(&mut self) {
    //     let p = param();
    //     for (entity, interaction) in &mut p.setting_button_interactions {
    //         let Ok((button, mut border, mut background)) = p.setting_buttons.get_mut(entity) else {
    //             continue;
    //         };

    //         match *interaction {
    //             Interaction::Pressed => {
    //                 println!("{button:?}: pressed");
    //             }
    //             Interaction::Hovered => {
    //                 border.set_all(Color::WHITE);
    //             }
    //             Interaction::None => {
    //                 border.set_all(Color::BLACK);
    //             }
    //         }
    //     }
    // }
}
