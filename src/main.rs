mod app;

use app::*;

#[cfg(target_os = "linux")]
fn get_options() -> eframe::NativeOptions {
    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default(),
        ..Default::default()
    };

    native_options
}

#[cfg(target_os = "windows")]
fn get_options() -> eframe::NativeOptions {
    let ico_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets/clipboard_paste_20260217012532.png.ico");
    let icon_file = fs::File::open(ico_path).unwrap();
    let icon_dir = ico::IconDir::read(icon_file).unwrap();
    let largest_entry = icon_dir
        .entries()
        .iter()
        .max_by_key(|entry| entry.width() * entry.height())
        .unwrap();
    let icon_image = largest_entry.decode().unwrap();
    let icon = eframe::egui::IconData {
        width: icon_image.width(),
        height: icon_image.height(),
        rgba: icon_image.into_rgba_data(),
    };

    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default().with_icon(icon),
        ..Default::default()
    };

    native_options
}

fn main() {
    let _ = eframe::run_native(
        "Todo",
        get_options(),
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    );
}
