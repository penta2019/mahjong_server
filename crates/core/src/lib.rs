// 構造的な意味合いや一貫性を保つために以下のclippy警告は無効化
#![allow(clippy::collapsible_else_if)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::too_many_arguments)]

mod actor;
mod control;
mod convert;
mod hand;
mod listener;
mod util;

pub mod app;
pub mod model;
