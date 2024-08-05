#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    eframe::run_native(
        "Grapher",
        eframe::NativeOptions::default(),
        Box::new(|_| Ok(Box::new(bpptng_grapher::Grapher::new()))),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {}
