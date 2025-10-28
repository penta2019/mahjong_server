use super::{super::prelude::*, *};

#[derive(Debug)]
pub struct WinDialog {
    entity: Entity,
}

impl WinDialog {
    pub fn new(event: &EventWin, camera_seat: Seat) -> Self {
        let p = param();

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
            ))
            .id();

        p.cmd.spawn((
            ChildOf(entity),
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Percent(4.0),
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            children![create_ok_button()],
        ));
        Self { entity }
    }
}

impl Dialog for WinDialog {
    fn handle_event(&mut self, ok_buttons: &mut OkButtonQuery) -> bool {
        handle_dialog_ok_button(ok_buttons)
    }

    fn destroy(self: Box<Self>) {
        cmd().entity(self.entity).despawn();
    }
}
