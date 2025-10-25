use super::prelude::*;

pub struct PopupDraw {
    entity: Entity,
}

impl PopupDraw {
    pub fn new(event: &EventDraw) -> Self {
        let cmd = cmd();

        let entity = cmd
            .spawn((
                Node {
                    justify_self: JustifySelf::Stretch,
                    align_self: AlignSelf::Stretch,
                    width: Val::Px(500.0),
                    height: Val::Px(500.0),
                    ..default()
                },
                BackgroundColor(Color::BLACK),
            ))
            .id();

        Self { entity }
    }

    pub fn destroy(self) {
        cmd().entity(self.entity).despawn();
    }
}
