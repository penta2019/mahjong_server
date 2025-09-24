use bevy::prelude::*;

use super::{control::ControlContext, util::MsecTimer};

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DebugState {
    On,
    Off,
}

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.insert_state(DebugState::Off)
            .add_systems(Startup, setup)
            .add_systems(OnEnter(DebugState::On), state_on)
            .add_systems(OnExit(DebugState::On), state_off)
            .add_systems(Update, update.run_if(in_state(DebugState::On)));
    }
}

#[derive(Component, Debug)]
struct Container;

#[derive(Component, Debug)]
enum Info {
    Fps,
    Pos,
    Yaw,
    Pitch,
}

fn setup(mut commands: Commands) {
    let root = commands
        .spawn((
            Container,
            Node {
                padding: UiRect::all(Val::Px(20.0)),
                justify_self: JustifySelf::Stretch,
                align_self: AlignSelf::Stretch,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                row_gap: Val::Px(4.0),
                ..default()
            },
            ZIndex(0),
            Visibility::Hidden,
        ))
        .id();
    commands.spawn((
        ChildOf(root),
        Info::Fps,
        Text::new("fps: 0"),
        Node { ..default() },
    ));
    commands.spawn((
        ChildOf(root),
        Info::Pos,
        Text::new("pos:"),
        Node { ..default() },
    ));
    commands.spawn((
        ChildOf(root),
        Info::Yaw,
        Text::new("yaw:"),
        Node { ..default() },
    ));
    commands.spawn((
        ChildOf(root),
        Info::Pitch,
        Text::new("pitch:"),
        Node { ..default() },
    ));
}

fn state_on(mut visibility: Single<&mut Visibility, With<Container>>) {
    **visibility = Visibility::Visible;
}

fn state_off(mut visibility: Single<&mut Visibility, With<Container>>) {
    **visibility = Visibility::Hidden;
}

fn update(
    time: Res<Time>,
    mut timer: Local<MsecTimer<500>>,
    camera: Single<&Transform, With<super::control::MainCamera>>,
    control_context: Res<ControlContext>,
    mut texts: Query<(&mut Text, &Info)>,
) {
    let is_update_period = timer.tick(&time);

    for (mut text, info) in &mut texts {
        match *info {
            Info::Fps => {
                if !is_update_period {
                    continue;
                }
                let delta = time.delta_secs();
                let fps = 1.0 / delta;
                text.0 = format!("fps: {:.1}", fps);
            }
            Info::Pos => {
                let t = camera.translation;
                text.0 = format!("pos: ({:.2}, {:.2}, {:.2})", t.x, t.y, t.z);
            }
            Info::Yaw => {
                text.0 = format!("yaw: {:.2}", control_context.yaw());
            }
            Info::Pitch => {
                text.0 = format!("pitch: {:.2}", control_context.pitch());
            }
        }
    }
}
