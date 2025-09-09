use bevy::prelude::*;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SliderState {
    On,
    Off,
}

pub struct SliderPlugin;

impl Plugin for SliderPlugin {
    fn build(&self, app: &mut App) {
        app.insert_state(SliderState::Off)
            .insert_resource(SliderConfig {
                is_interacting: false,
            })
            .add_systems(
                Update,
                (
                    slider_changed,
                    slider_interaction,
                    slider_interaction_mouse.run_if(is_interacting),
                )
                    .run_if(in_state(SliderState::On)),
            );
    }
}

pub fn create_slider(commands: &mut Commands, width: f32) -> Entity {
    let slider = commands
        .spawn((
            Slider {
                dragging: false,
                percent: 50.0,
                dirty: false,
            },
            Button,
            Node {
                width: Val::Px(width),
                height: Val::Px(8.0),
                ..default()
            },
            BorderRadius::all(Val::Px(4.0)),
            BackgroundColor::from(Color::srgb(0.3, 0.3, 0.3)),
        ))
        .id();

    let slider_inner = commands
        .spawn((
            SliderInnder,
            Node {
                width: Val::Percent(50.0),
                justify_self: JustifySelf::Stretch,
                align_self: AlignSelf::Stretch,
                ..default()
            },
            BorderRadius::all(Val::Px(4.0)),
            BackgroundColor::from(Color::WHITE),
        ))
        .id();
    commands.entity(slider).add_child(slider_inner);

    slider
}

#[derive(Component, Debug)]
pub struct Slider {
    dragging: bool,
    percent: f32,
    dirty: bool, // perncetの値がsetで変更されたのでInnerSliderの更新が必要
}

impl Slider {
    pub fn get(&self) -> f32 {
        self.percent
    }

    pub fn set(&mut self, percent: f32) {
        self.percent = percent;
        self.dirty = true;
    }
}

#[derive(Resource, Debug)]
struct SliderConfig {
    is_interacting: bool,
}

#[derive(Component, Debug)]
struct SliderInnder;

fn is_interacting(config: Res<SliderConfig>) -> bool {
    config.is_interacting
}

fn slider_changed(
    mut slider_query: Query<(&mut Slider, &Children), Changed<Slider>>,
    mut slider_inner_query: Query<&mut Node, (With<SliderInnder>, Without<Slider>)>,
) {
    for (mut slider, children) in &mut slider_query {
        if !slider.dirty {
            continue;
        }
        slider.dirty = false;
        let mut inner = slider_inner_query.get_mut(children[0]).unwrap();
        inner.width = Val::Percent(slider.get());
    }
}

fn slider_interaction(
    mut config: ResMut<SliderConfig>,
    mut slider_query: Query<(&Interaction, &mut Slider), (Changed<Interaction>, With<Slider>)>,
) {
    for (interaction, mut slider) in &mut slider_query {
        match *interaction {
            Interaction::Pressed => {
                slider.dragging = true;
                config.is_interacting = true;
            }
            Interaction::Hovered | Interaction::None => {
                // dragging中に別のsliderのinteractionが発生する場合を除外
                if slider.dragging {
                    config.is_interacting = false;
                    slider.dragging = false;
                }
            }
        };
    }
}

fn slider_interaction_mouse(
    windows: Query<&Window>,
    mut slider_query: Query<(&GlobalTransform, &mut Slider, &Node, &Children)>,
    mut slider_inner_query: Query<&mut Node, (With<SliderInnder>, Without<Slider>)>,
) {
    let window = windows.single().unwrap();
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    for (transform, mut slider, node, children) in &mut slider_query {
        if !slider.dragging {
            continue;
        }

        let trans = transform.translation();
        let Val::Px(width) = node.width else {
            continue;
        };
        let percent = ((cursor_pos.x - trans.x) / width + 0.5) * 100.0;
        let percent = percent.round().clamp(0.0, 100.0);
        slider.percent = percent;
        let mut inner = slider_inner_query.get_mut(children[0]).unwrap();
        inner.width = Val::Percent(percent);
    }
}
