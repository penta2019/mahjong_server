use bevy::{
    camera::{RenderTarget, visibility::RenderLayers},
    color::palettes::css::WHITE,
};

use super::super::prelude::*;

#[derive(Debug)]
pub struct StageInfo {
    entity: Entity,
    camera: Entity,
    ui: Entity,
    round: Option<Entity>,
}
crate::impl_has_entity!(StageInfo);

impl StageInfo {
    pub fn new() -> Self {
        let p = param();

        // テクスチャを中央パネルに貼る
        let mesh_handle = p.meshes.add(Plane3d::default().mesh().size(0.12, 0.12));
        let material_handle = p.materials.add(StandardMaterial {
            base_color_texture: Some(p.info_texture.0.clone()),
            reflectance: 0.02,
            unlit: false,
            ..default()
        });
        let entity = p
            .cmd
            .spawn((Mesh3d(mesh_handle), MeshMaterial3d(material_handle)))
            .id();

        // UIをテクスチャにレンダリングするためのCamera2Dの初期化
        let camera = p
            .cmd
            .spawn((
                Camera2d,
                Camera {
                    // render before the "main pass" camera
                    order: -1,
                    target: RenderTarget::Image(p.info_texture.0.clone().into()),
                    ..default()
                },
                RenderLayers::layer(1),
            ))
            .id();

        // UIのルートEntity
        let ui = p
            .cmd
            .spawn((
                Name::new("StageInfo".to_string()),
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
        let p = param();

        let wind = ["東", "南", "西", "北"];
        // let wind = ["E", "S", "W", "N"];

        let font = p.asset_server.load("font/NotoSerifCJKjp-Regular.otf");

        let round_text = format!("{}{}局", wind[event.round], (event.dealer + 1));
        let round = p
            .cmd
            .spawn((
                Text2d(round_text),
                TextFont {
                    font: font.clone(),
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
            p.cmd.spawn((
                ChildOf(self.ui),
                Transform::from_rotation(Quat::from_rotation_z(s as f32 * FRAC_PI_2)),
                Visibility::Visible,
                RenderLayers::layer(1),
                children![
                    (
                        Text2d(wind[i_wind].into()),
                        TextFont {
                            font: font.clone(),
                            font_size: 60.0,
                            ..default()
                        },
                        TextColor(WHITE.into()),
                        Transform::from_xyz(-150.0, -200.0, 0.0),
                        RenderLayers::layer(1),
                    ),
                    (
                        Text2d(event.scores[s].to_string()),
                        TextFont {
                            font: font.clone(),
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
        let cmd = cmd();
        cmd.entity(self.camera).despawn();
        cmd.entity(self.ui).despawn();
    }

    pub fn set_camera_seat(&self, seat: Seat) {
        if let Some(e_round) = self.round {
            cmd()
                .entity(e_round)
                .insert(Transform::from_rotation(Quat::from_rotation_z(
                    FRAC_PI_2 * seat as f32,
                )));
        }
    }
}
