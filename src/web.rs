use crate::FunctionEntry;

#[cfg(target_arch = "wasm32")]
use crate::Grapher;

#[cfg(target_arch = "wasm32")]
use eframe::wasm_bindgen::{self, prelude::*};

#[cfg(target_arch = "wasm32")]
use rodio::OutputStreamHandle;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn start_web(canvas_id: &str) -> Result<(), eframe::wasm_bindgen::JsValue> {
    use rodio::OutputStream;

    console_error_panic_hook::set_once();

    let mut _stream = None;
    let mut stream_handle = None;
    if let Ok((s, sh)) = OutputStream::try_default()
        .inspect_err(|e| eprintln!("could not open an audio stream: {e}"))
    {
        _stream = Some(s);
        stream_handle = Some(sh);
    }

    eframe::WebRunner::new()
        .start(
            canvas_id,
            Default::default(),
            Box::new(|_| Ok(Box::new(Grapher::new(stream_handle)))),
        )
        .await
}

#[cfg(target_arch = "wasm32")]
pub fn update_url(data: &Vec<FunctionEntry>) {
    let history = web_sys::window()
        .expect("Couldn't get window")
        .history()
        .expect("Couldn't get window.history");

    let info_str = url_string_from_data(data);

    history
        .push_state_with_url(&JsValue::NULL, "", Some(&info_str))
        .unwrap();
}

pub fn url_string_from_data(data: &Vec<FunctionEntry>) -> String {
    let mut info_str = String::from("#");

    for entry in data {
        info_str.push_str(format!("{},", entry.text).as_str());
    }

    info_str.pop();

    info_str
}

#[cfg(target_arch = "wasm32")]
pub fn get_data_from_url(
    data: &mut Vec<FunctionEntry>,
    stream_handle: Option<&OutputStreamHandle>,
) -> Option<String> {
    use rodio::Sink;

    use crate::audio::Math;

    let href = web_sys::window()
        .expect("Couldn't get window")
        .document()
        .expect("Couldn't get document")
        .location()
        .expect("Couldn't get location")
        .href()
        .expect("Couldn't get href");

    if !href.contains('#') {
        return None;
    }

    let func_string = match href.split('#').last() {
        Some(x) => x,
        None => return None,
    };

    if func_string.is_empty() {
        return None;
    }

    let mut error: Option<String> = None;

    for entry in func_string.split(',') {
        let sink = stream_handle.and_then(|sh| Sink::try_new(sh).ok().inspect(|s| s.pause()));

        let func = match exmex::parse::<f64>(entry) {
            Ok(func) => {
                if let Some(ref s) = sink {
                    s.append(Math::new(unsafe {
                        exmex::parse::<f32>(entry).unwrap_unchecked()
                    }));
                }
                Some(func)
            }
            Err(e) => {
                error = Some(e.to_string());
                None
            }
        };

        data.push(FunctionEntry {
            text: entry.to_string(),
            func,
            sink,
        });
    }

    error
}
