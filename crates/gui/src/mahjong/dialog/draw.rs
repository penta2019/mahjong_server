use mahjong_core::control::common::{get_names, get_scores};

use super::*;

#[derive(Debug)]
pub struct DrawDialog {
    entity: Entity,
}

impl DrawDialog {
    pub fn new(stage: &Stage, event: &EventDraw, camera_seat: Seat) -> Self {
        let players_info = create_players_info(
            camera_seat,
            stage.dealer,
            &get_names(stage),
            &get_scores(stage),
            &event.delta_scores,
        );

        let round_title = round_string(stage.round, stage.dealer, Some(stage.honba));
        let draw_str = if event.nagashimangan_scores.iter().any(|score| *score != 0) {
            "流し満貫".into()
        } else {
            event.draw_type.to_string()
        };

        let entity = create_round_dialog(round_title, draw_str, players_info);

        Self { entity }
    }
}

impl Dialog for DrawDialog {
    fn handle_event(&mut self, ok_buttons: &mut OkButtonQuery) -> bool {
        if handle_dialog_ok_button(ok_buttons) {
            cmd().entity(self.entity).despawn();
            true
        } else {
            false
        }
    }
}
