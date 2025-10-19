#[allow(dead_code)]
fn silence_unused_warning() {
    let _ = crate::model::Tile::unicode;
    let _ = crate::control::common::tiles_to_tile_table;
    let _ = crate::listener::StageSender::new;
    let _ = crate::util::waiter::Waiter::wait_timeout;
    let _ = crate::util::connection::WsConnection::new;
    let _ = crate::control::stage_controller::StageController::swap_actor;
    let _ = crate::hand::YakuDefine::get_from_id;
}
