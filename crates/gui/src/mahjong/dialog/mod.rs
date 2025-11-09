mod draw;
mod end;
mod players_info;
mod round;
mod win;

use bevy::prelude::*;

use super::{prelude::*, text::create_text};

pub use self::{draw::DrawDialog, end::EndDialog, round::RoundDialog, win::WinDialog};

#[derive(Component, Debug)]
pub struct OkButton;

pub type OkButtonQuery<'w, 's> = Query<
    'w,
    's,
    (&'static Interaction, &'static mut BorderColor),
    (Changed<Interaction>, With<OkButton>),
>;

pub trait Dialog: std::fmt::Debug + Sync + Send {
    // Sync, SendはResourceに含めるために必要

    // Dialogの処理が終了した場合はtrueを返却 呼び出し元からdestroy()を実行
    fn handle_event(&mut self, ok_buttons: &mut OkButtonQuery) -> bool;
}

fn handle_dialog_ok_button(buttons: &mut OkButtonQuery) -> bool {
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
