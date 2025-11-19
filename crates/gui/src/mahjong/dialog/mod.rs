mod draw;
mod end;
mod internal;
mod round;
mod win;

use bevy::prelude::*;

use self::internal::*;
use super::{
    prelude::*,
    text::{round_string, wind_to_char_jp},
};

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

    // Dialogの処理が終了した場合はtrueを返却
    fn handle_event(&mut self, ok_buttons: &mut OkButtonQuery) -> bool;
}
