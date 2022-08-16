#[allow(dead_code)]
fn silence_unused_warning() {
    let _ = crate::model::Tile::is_simple;

    let _ = crate::hand::is_normal_win;
    let _ = crate::hand::is_chiitoitsu_win;

    let _ = crate::convert::tenhou::TenhouLog::new;

    let _ = crate::listener::TenhouEventWriter::new;
    let _ = crate::listener::StageStepPrinter::new;
    let _ = crate::listener::StageDebugPrinter::new;
}
