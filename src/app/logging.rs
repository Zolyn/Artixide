use log::info;

pub fn save_log() {
    info!("Saving log file");
    tui_logger::move_events()
}
