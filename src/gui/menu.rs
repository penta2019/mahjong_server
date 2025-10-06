use bevy::prelude::*;

use super::{
    camera::CameraContext,
    slider::{Slider, SliderState, create_slider},
};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.insert_state(MenuState::Hidden)
            .insert_resource(MenuConfig::default())
            .add_systems(Startup, setup)
            .add_systems(OnEnter(MenuState::Visible), state_visible)
            .add_systems(OnEnter(MenuState::Hidden), state_hidden)
            .add_systems(
                Update,
                handler_button_interaction.run_if(in_state(MenuState::Visible)),
            );
    }
}

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MenuState {
    Visible,
    Hidden,
}

#[derive(Resource, Debug, Default)]
struct MenuConfig {
    mouse_sensitivity_slider: Option<Entity>,
}

const NORMAL_BUTTON: Color = Color::srgb(0.3, 0.3, 0.3);
const HOVERED_BUTTON: Color = Color::srgb(0.4, 0.4, 0.4);
const PRESSED_BUTTON: Color = Color::srgb(0.5, 0.5, 0.5);
const TEXT: Color = Color::srgb(1.0, 1.0, 1.0);

#[derive(Component)]
struct MenuUI;

#[derive(Component, Debug)]
enum MenuButton {
    Quit,
}

fn setup(mut commands: Commands, mut config: ResMut<MenuConfig>) {
    let container = commands
        .spawn((
            MenuUI,
            Node {
                padding: UiRect::axes(Val::Px(100.0), Val::Px(40.0)),
                margin: UiRect::all(Val::Px(25.0)),
                justify_self: JustifySelf::Stretch,
                align_self: AlignSelf::Stretch,
                flex_direction: FlexDirection::Column,
                // justify_content: JustifyContent::FlexStart,
                // align_content: AlignContent::FlexStart,
                align_items: AlignItems::FlexEnd,
                row_gap: Val::Px(10.0),
                ..default()
            },
            ZIndex(1),
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
            Visibility::Hidden, // 初期は非表示
        ))
        .id();

    let text_entity = commands
        .spawn((
            MenuButton::Quit,
            Button,
            Node {
                padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor::from(NORMAL_BUTTON),
            BorderRadius::all(Val::Px(4.0)),
            children![(Text::new("Quit"), TextColor(TEXT))],
        ))
        .id();
    commands.entity(container).add_child(text_entity);

    let e_sensitivity_container = commands
        .spawn((
            Node {
                justify_self: JustifySelf::Stretch,
                align_self: AlignSelf::Stretch,
                justify_content: JustifyContent::FlexEnd,
                align_items: AlignItems::Center,
                ..default()
            },
            // BackgroundColor::from(bevy::color::palettes::basic::AQUA),
        ))
        .id();
    commands
        .entity(container)
        .add_child(e_sensitivity_container);

    let sensitivity_text = commands
        .spawn((
            Node {
                margin: UiRect::right(Val::Px(50.0)),
                ..default()
            },
            Text::new("mouse sensitivity"),
        ))
        .id();
    commands
        .entity(e_sensitivity_container)
        .add_child(sensitivity_text);

    let sensitivity_slider = create_slider(&mut commands, 300.0);
    commands
        .entity(e_sensitivity_container)
        .add_child(sensitivity_slider);
    config.mouse_sensitivity_slider = Some(sensitivity_slider);
}

fn state_visible(
    config: Res<MenuConfig>,
    control_context: Res<CameraContext>,
    mut visibility: Single<&mut Visibility, With<MenuUI>>,
    mut slider_state: ResMut<NextState<SliderState>>,
    sliders: Query<(Entity, &mut Slider)>,
) {
    **visibility = Visibility::Visible;
    slider_state.set(SliderState::On);

    for (e_slider, mut slider) in sliders {
        if e_slider == config.mouse_sensitivity_slider.unwrap() {
            slider.set(control_context.get_mouse_sensitivity())
        }
    }
}

fn state_hidden(
    config: Res<MenuConfig>,
    mut control_context: ResMut<CameraContext>,
    mut visivility: Single<&mut Visibility, With<MenuUI>>,
    mut slider_state: ResMut<NextState<SliderState>>,
    sliders: Query<(Entity, &Slider)>,
) {
    **visivility = Visibility::Hidden;
    slider_state.set(SliderState::Off);

    for (e_slider, slider) in sliders {
        if e_slider == config.mouse_sensitivity_slider.unwrap() {
            control_context.set_mouse_sensitivity(slider.get());
        }
    }
}

fn handler_button_interaction(
    buttons: Query<
        (&Interaction, &mut BackgroundColor, &MenuButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut app_exit: MessageWriter<AppExit>,
) {
    for (interaction, mut color, button) in buttons {
        match *interaction {
            Interaction::Pressed => {
                *color = BackgroundColor(PRESSED_BUTTON);
                match button {
                    MenuButton::Quit => app_exit.write(AppExit::Success),
                };
            }
            Interaction::Hovered => {
                *color = BackgroundColor(HOVERED_BUTTON);
            }
            Interaction::None => {
                *color = BackgroundColor(NORMAL_BUTTON);
            }
        }
    }
}
