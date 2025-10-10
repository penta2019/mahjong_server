use bevy::{
    camera::{RenderTarget, visibility::RenderLayers},
    color::palettes::css::WHITE,
    prelude::*,
};

use super::*;

#[derive(Debug)]
pub struct StageInfo {
    entity: Entity,
    camera: Entity,
    ui: Entity,
    round: Option<Entity>,
}

impl StageInfo {
    pub fn new() -> Self {
        let param = param();
        let commands = &mut param.commands;

        // Camera2DからUIを画像にレンダリングしてテクスチャとして平面に貼る

        let mesh_handle = param.meshes.add(Plane3d::default().mesh().size(0.12, 0.12));
        let material_handle = param.materials.add(StandardMaterial {
            base_color_texture: Some(param.info_texture.0.clone()),
            reflectance: 0.02,
            unlit: false,
            ..default()
        });
        let entity = commands
            .spawn((
                Mesh3d(mesh_handle),
                MeshMaterial3d(material_handle),
                Transform::from_xyz(0.0, 0.001, 0.0),
            ))
            .id();

        // UIレンダリング用のCamera2D
        let camera = commands
            .spawn((
                Camera2d,
                Camera {
                    // render before the "main pass" camera
                    order: -1,
                    target: RenderTarget::Image(param.info_texture.0.clone().into()),
                    ..default()
                },
                RenderLayers::layer(1),
            ))
            .id();

        let ui = commands
            .spawn((
                Transform::IDENTITY,
                Visibility::Visible,
                RenderLayers::layer(1),
            ))
            .id();
        Self {
            entity,
            camera,
            ui,
            round: None,
        }
    }

    pub fn init(&mut self, event: &EventNew) {
        let param = param();
        let commands = &mut param.commands;

        let wind = ["E", "S", "W", "N"];

        let round_text = format!("{} {}", wind[event.round], (event.dealer + 1));
        let round = commands
            .spawn((
                Text2d(round_text.into()),
                TextFont {
                    font_size: 80.0,
                    ..default()
                },
                TextColor(WHITE.into()),
                ChildOf(self.ui),
                Transform::IDENTITY,
                RenderLayers::layer(1),
            ))
            .id();
        self.round = Some(round);

        for s in 0..SEAT {
            let i_wind = (s + SEAT - event.dealer) % SEAT;
            commands.spawn((
                ChildOf(self.ui),
                Transform::from_rotation(Quat::from_rotation_z(s as f32 * FRAC_PI_2)),
                Visibility::Visible,
                RenderLayers::layer(1),
                children![
                    (
                        Text2d(wind[i_wind].into()),
                        TextFont {
                            font_size: 60.0,
                            ..default()
                        },
                        TextColor(WHITE.into()),
                        Transform::from_xyz(-150.0, -200.0, 0.0),
                        RenderLayers::layer(1),
                    ),
                    (
                        Text2d(event.scores[s].to_string().into()),
                        TextFont {
                            font_size: 60.0,
                            ..default()
                        },
                        TextColor(WHITE.into()),
                        Transform::from_xyz(0.0, -200.0, 0.0),
                        RenderLayers::layer(1),
                    )
                ],
            ));
        }
    }

    pub fn destroy(self) {
        let param = param();
        param.commands.entity(self.entity).despawn();
        param.commands.entity(self.camera).despawn();
        param.commands.entity(self.ui).despawn();
    }

    pub fn set_camera_seat(&self, seat: Seat) {
        if let Some(e_round) = self.round {
            param()
                .commands
                .entity(e_round)
                .insert(Transform::from_rotation(Quat::from_rotation_z(
                    FRAC_PI_2 * seat as f32,
                )));
        }
    }
}
