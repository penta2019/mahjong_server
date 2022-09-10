#[allow(dead_code)]
fn silence_unused_warning() {
    let _ = crate::model::Action::riichi_drawn;

    let _ = crate::hand::is_normal_win;
    let _ = crate::hand::is_chiitoitsu_win;

    let _ = crate::listener::EventPrinter::new;
    let _ = crate::listener::StageSender::new;
}
