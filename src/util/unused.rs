#[allow(dead_code)]
fn silence_unused_warning() {
    let _ = crate::hand::is_normal_win;
    let _ = crate::hand::is_chiitoitsu_win;

    let _ = crate::model::Action::riichi_drawn;

    let _ = crate::control::common::tiles_to_tile_table;

    let _ = crate::listener::EventPrinter::new;
    let _ = crate::listener::StageSender::new;

    let _ = crate::util::waiter::Waiter::wait_timeout;

    // for J mode
    let _ = crate::util::connection::WsConnection::new;
    let _ = crate::util::misc::as_usize;
    let _ = crate::util::misc::as_i32;
    let _ = crate::util::misc::as_str;
    let _ = crate::util::misc::as_bool;
    let _ = crate::util::misc::as_array;
    let _ = crate::util::misc::as_enumerate;
    let _ = crate::util::misc::as_vec(|e| e.clone(), &serde_json::json!(null));
    let _ = crate::control::stage_controller::StageController::swap_actor;
    let _ = crate::hand::YakuDefine::get_from_id;
}
