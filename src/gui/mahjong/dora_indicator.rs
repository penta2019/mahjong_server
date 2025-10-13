use bevy::color::palettes::css::BLACK;

use super::{
    prelude::*,
    stage::{CAMERA_LOOK_AT, CAMERA_POS},
};

#[derive(Debug)]
pub struct DoraIndicator {
    entity: Entity,
}

impl DoraIndicator {
    pub fn new() -> Self {
        let param = param();

        let tf_camera = Transform::from_translation(CAMERA_POS).looking_at(CAMERA_LOOK_AT, Vec3::Y);
        let tf_camera_self = Transform {
            translation: Vec3::new(-0.15, 0.15, -0.8),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        };
        let entity = param
            .commands
            .spawn((
                Name::new("DoraIndicator".to_string()),
                tf_camera * tf_camera_self,
            ))
            .with_child((
                Mesh3d(param.meshes.add(Plane3d::default().mesh().size(0.1, 0.03))),
                MeshMaterial3d(param.materials.add(Color::from(BLACK))),
                Transform::from_rotation(Quat::from_rotation_x(FRAC_PI_2)),
            ))
            .id();

        Self { entity }
    }
}

impl HasEntity for DoraIndicator {
    fn entity(&self) -> Entity {
        self.entity
    }
}
