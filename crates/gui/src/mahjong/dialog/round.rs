use super::*;

#[derive(Debug)]
pub struct RoundDialog {
    entity: Entity,
}

impl RoundDialog {
    pub fn new(event: &EventNew, camera_seat: Seat) -> Self {
        let players_info = create_players_info(
            camera_seat,
            event.dealer,
            &event.names,
            &event.scores,
            &[0, 0, 0, 0],
        );

        let game_title = match event.rule.round {
            1 => "東風戦",
            2 => "半荘戦",
            4 => "一荘戦",
            _ => "",
        }
        .into();
        let round_title = round_string(event.round, event.dealer, Some(event.honba));
        let entity = create_round_dialog(game_title, round_title, players_info);

        Self { entity }
    }
}

impl Dialog for RoundDialog {
    fn handle_event(&mut self, ok_buttons: &mut OkButtonQuery) -> bool {
        if handle_dialog_ok_button(ok_buttons) {
            cmd().entity(self.entity).despawn();
            true
        } else {
            false
        }
    }
}
