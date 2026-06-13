mod app;
use app::*;

fn main() {
    let native_options = eframe::NativeOptions::default();
    let _ = eframe::run_native(
        "Todo",
        native_options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    );
}
