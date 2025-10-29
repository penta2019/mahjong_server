use super::{super::prelude::*, *};

#[derive(Debug)]
pub struct WinDialog {
    entity: Entity,
    event: EventWin,
    camera_seat: Seat,
    next_win_index: usize,
}

impl WinDialog {
    pub fn new(event: &EventWin, camera_seat: Seat) -> Self {
        let mut obj = Self {
            entity: cmd().spawn_empty().id(),
            event: event.clone(),
            camera_seat,
            next_win_index: 0,
        };
        obj.next_win();
        obj
    }

    fn next_win(&mut self) -> bool {
        let p = param();

        p.cmd.entity(self.entity).despawn();
        if self.next_win_index >= self.event.contexts.len() {
            return false;
        }
        let _ctx = &self.event.contexts[self.next_win_index];
        self.next_win_index += 1;

        let entity = p
            .cmd
            .spawn((
                Node {
                    justify_self: JustifySelf::Center,
                    align_self: AlignSelf::Center,
                    width: Val::Px(600.0),
                    height: Val::Px(600.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
            ))
            .id();

        p.cmd.spawn((
            ChildOf(entity),
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(8.0),
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            children![create_ok_button()],
        ));

        self.entity = entity;

        true
    }
}

impl Dialog for WinDialog {
    fn handle_event(&mut self, ok_buttons: &mut OkButtonQuery) -> bool {
        handle_dialog_ok_button(ok_buttons) && !self.next_win()
    }
}
