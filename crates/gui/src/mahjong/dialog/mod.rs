mod draw;
mod players_info;
mod win;

use bevy::prelude::*;

use super::text::create_text;

pub use self::draw::DrawDialog;

#[derive(Component, Debug)]
pub struct OkButton;

pub fn handle_dialog_ok_button(
    buttons: &mut Query<
        (&'static Interaction, &'static mut BorderColor),
        (Changed<Interaction>, With<OkButton>),
    >,
) -> bool {
    for (iteraction, mut border) in buttons {
        match iteraction {
            Interaction::Pressed => return true,
            Interaction::Hovered => border.set_all(Color::WHITE),
            Interaction::None => border.set_all(Color::BLACK),
        };
    }
    false
}

fn create_ok_button() -> impl Bundle {
    (
        OkButton,
        Button,
        Node {
            width: Val::Px(100.0),
            height: Val::Px(32.0),
            border: UiRect::all(Val::Px(1.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        // BorderRadius::all(Val::Px(4.0)),
        BorderColor::all(Color::BLACK),
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.1)),
        children![create_text("OK".into(), 20.0)],
    )
}
