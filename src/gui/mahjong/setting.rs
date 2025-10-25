use super::prelude::*;

use bevy::{ecs::system::SystemParam, input::mouse::MouseButtonInput};

const BUTTON_ACTIVE: Color = Color::srgba(0.0, 0.40, 0.0, 0.9);
const BUTTON_INACTIVE: Color = Color::srgba(0.0, 0.0, 0.0, 0.9);

#[derive(SystemParam)]
pub struct SettingParam<'w, 's> {
    // Setting Button
    setting_buttons: Query<
        'w,
        's,
        (
            &'static mut SettingButton,
            &'static mut BorderColor,
            &'static mut BackgroundColor,
        ),
    >,
    setting_button_interactions:
        Query<'w, 's, (Entity, &'static Interaction), (Changed<Interaction>, With<SettingButton>)>,
    mouse_input: MessageReader<'w, 's, MouseButtonInput>,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum SettingButton {
    Root,
    Wall,
    Hand,
    Seat(usize),
}

#[derive(Debug, Clone)]
pub struct SettingProps {
    pub show_wall: bool,
    pub show_hand: bool,
    pub camera_seat: usize,
}

#[derive(Debug)]
pub struct Setting {
    button: Entity,
    menu: Entity,
    is_expanded: bool,
    props: SettingProps,
    update_request: bool,
}

impl Setting {
    pub fn new() -> Self {
        let cmd = cmd();

        let button = cmd
            .spawn((
                SettingButton::Root,
                Button,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(0.0),
                    bottom: Val::Percent(0.0),
                    width: Val::Px(100.0),
                    height: Val::Px(32.0),
                    border: UiRect::all(Val::Px(1.0)),
                    // 内部のテキストを中央に表示(横方向)
                    justify_content: JustifyContent::Center,
                    // 内部のテキストを中央に表示(縦方向)
                    align_items: AlignItems::Center,

                    ..default()
                },
                BorderColor::all(Color::BLACK),
                BackgroundColor(BUTTON_INACTIVE),
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

        let menu = cmd
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    bottom: Val::Px(32.0),
                    width: Val::Px(200.0),
                    border: UiRect::all(Val::Px(1.0)),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                BorderColor::all(Color::BLACK),
                Visibility::Hidden,
            ))
            .with_children(|c| {
                c.spawn(create_button(SettingButton::Wall, "Show Wall"));
                c.spawn(create_button(SettingButton::Hand, "Show Hand"));
                for s in 0..SEAT {
                    c.spawn(create_button(SettingButton::Seat(s), &format!("Seat {s}")));
                }
            })
            .id();

        Self {
            button,
            menu,
            is_expanded: false,
            props: SettingProps {
                show_wall: false,
                show_hand: false,
                camera_seat: 0,
            },
            update_request: true,
        }
    }

    pub fn destroy(self) {
        let cmd = cmd();
        cmd.entity(self.button).despawn();
        cmd.entity(self.menu).despawn();
    }

    pub fn get_props(&self) -> &SettingProps {
        &self.props
    }

    pub fn set_props(&mut self, props: SettingProps) {
        self.props = props;
        self.update_request = true; // handle_gui_eventsでGUI側からの変更と一緒に処理
    }

    pub fn handle_gui_events(&mut self, setting_param: &mut SettingParam) -> bool {
        // Gui側からの変更を適用
        let mut is_changed = false;
        for (entity, interaction) in &mut setting_param.setting_button_interactions {
            let Ok((button, mut border, _)) = setting_param.setting_buttons.get_mut(entity) else {
                continue;
            };

            match *button {
                SettingButton::Root => match *interaction {
                    Interaction::Pressed => {
                        if self.is_expanded {
                            self.close_menu();
                        } else {
                            self.open_menu();
                        };
                    }
                    Interaction::Hovered => {
                        border.set_all(Color::WHITE);
                    }
                    Interaction::None => {
                        border.set_all(Color::BLACK);
                    }
                },
                _ => match *interaction {
                    Interaction::Pressed => {
                        is_changed = true;
                        match *button {
                            SettingButton::Root => {}
                            SettingButton::Wall => self.props.show_wall ^= true,
                            SettingButton::Hand => self.props.show_hand ^= true,
                            SettingButton::Seat(s) => self.props.camera_seat = s,
                        }
                    }
                    Interaction::Hovered => {
                        border.set_all(Color::WHITE);
                    }
                    Interaction::None => {
                        border.set_all(BUTTON_INACTIVE);
                    }
                },
            }
        }

        // setまたはGui側からの変更があった場合にすべて更新
        if self.update_request | is_changed {
            for (button, _, mut background) in &mut setting_param.setting_buttons {
                let flag = match *button {
                    SettingButton::Root => None,
                    SettingButton::Wall => Some(self.props.show_wall),
                    SettingButton::Hand => Some(self.props.show_hand),
                    SettingButton::Seat(s) => {
                        if s == self.props.camera_seat {
                            *background = BUTTON_ACTIVE.into();
                        } else {
                            *background = BUTTON_INACTIVE.into();
                        }
                        None
                    }
                };

                if let Some(flag) = flag {
                    *background = if flag {
                        BUTTON_ACTIVE.into()
                    } else {
                        BUTTON_INACTIVE.into()
                    };
                }
            }
        }
        self.update_request = false;

        // Interactionが関係ないところでマウスボタン入力が発生した場合,メニューを閉じる
        if self.is_expanded
            && !setting_param.mouse_input.is_empty()
            && setting_param.setting_button_interactions.is_empty()
        {
            self.close_menu();
        }

        is_changed
    }

    fn open_menu(&mut self) {
        self.is_expanded = true;
        cmd().entity(self.menu).insert(Visibility::Visible);
    }

    fn close_menu(&mut self) {
        self.is_expanded = false;
        cmd().entity(self.menu).insert(Visibility::Hidden);
    }
}

fn create_button(button: SettingButton, text: &str) -> impl Bundle + use<> {
    (
        button,
        Button,
        Node {
            // width: Val::Percent(100.0),
            justify_self: JustifySelf::Stretch,
            height: Val::Px(30.0),
            border: UiRect::all(Val::Px(1.0)),
            // 内部のテキストを中央に表示(横方向)
            justify_content: JustifyContent::Center,
            // 内部のテキストを中央に表示(縦方向)
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(BUTTON_INACTIVE),
        BorderColor::all(BUTTON_INACTIVE),
        children![(
            Text::new(text),
            TextFont {
                // font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
            TextShadow::default(),
        )],
    )
}
