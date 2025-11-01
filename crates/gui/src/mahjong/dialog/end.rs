use super::*;

#[derive(Debug)]
pub struct EndDialog {
    entity: Entity,
}

impl EndDialog {
    pub fn new() -> Self {
        let entity = cmd().spawn_empty().id();
        Self { entity }
    }
}

impl Dialog for EndDialog {
    fn handle_event(&mut self, ok_buttons: &mut OkButtonQuery) -> bool {
        if handle_dialog_ok_button(ok_buttons) {
            cmd().entity(self.entity).despawn();
            true
        } else {
            false
        }
    }
}
