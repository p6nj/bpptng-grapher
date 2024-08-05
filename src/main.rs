#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    use rodio::OutputStream;

    let mut _stream = None;
    let mut stream_handle = None;
    if let Ok((s, sh)) = OutputStream::try_default() {
        _stream = Some(s);
        stream_handle = Some(sh);
    }
    eframe::run_native(
        "Grapher",
        eframe::NativeOptions::default(),
        Box::new(|_| Ok(Box::new(bpptng_grapher::Grapher::new(stream_handle)))),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {}
