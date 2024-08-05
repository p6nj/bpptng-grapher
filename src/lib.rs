use std::ops::Add;

use audio::Math;
use eframe::{
    egui::{
        self, CollapsingHeader, Frame, RichText, ScrollArea, SidePanel, Slider, Style, TextEdit,
        TextStyle, Visuals,
    },
    epaint::{Color32, Vec2},
    App,
};
use egui_plot::{Legend, Line, Plot, PlotPoints};
use exmex::{Express, FlatEx};

mod audio;
mod web;

use rodio::{OutputStreamHandle, Sink};
#[cfg(target_arch = "wasm32")]
pub use web::start_web;

const COLORS: &[Color32; 18] = &[
    Color32::RED,
    Color32::GREEN,
    Color32::YELLOW,
    Color32::BLUE,
    Color32::BROWN,
    Color32::GOLD,
    Color32::GRAY,
    Color32::WHITE,
    Color32::LIGHT_YELLOW,
    Color32::LIGHT_GREEN,
    Color32::LIGHT_BLUE,
    Color32::LIGHT_GRAY,
    Color32::LIGHT_RED,
    Color32::DARK_GRAY,
    Color32::DARK_RED,
    Color32::KHAKI,
    Color32::DARK_GREEN,
    Color32::DARK_BLUE,
];

#[derive(Clone)]
pub struct Grapher {
    data: Vec<FunctionEntry>,
    error: Option<String>,
    points: usize,
    stream_handle: Option<OutputStreamHandle>,
    listening: bool,
    examples_index: usize,
}

impl Grapher {
    pub fn new(stream_handle: Option<OutputStreamHandle>) -> Self {
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                let mut data = Vec::new();
                let error: Option<String> = web::get_data_from_url(&mut data, stream_handle.as_ref());
            } else {
                let data = Vec::new();
                let error: Option<String> = None;
            }
        }

        // if data.is_empty() {
        //     data.push(Default::default());
        // }

        Self {
            data,
            error,
            points: 500,
            stream_handle,
            listening: false,
            examples_index: 0,
        }
    }

    fn side_panel(&mut self, ctx: &egui::Context) {
        SidePanel::left("left_panel").show(ctx, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(6.0);
                ui.heading("BpptNG Grapher");
                ui.small("© 2022 Grant Handy - © 2024 Breval Ferrari");

                ui.separator();

                let mut outer_changed = false;

                ui.horizontal_top(|ui| {
                    if self.data.len() < 18 && ui.button("New").clicked() {
                        self.data.push(FunctionEntry::new("", self.stream_handle.as_ref()));
                        outer_changed = true;
                    }
                    if self.data.len() < 18 && ui.button("Random").clicked() {
                        let mut random = FunctionEntry::random(self.stream_handle.as_ref(), &mut self.examples_index);
                        match exmex::parse::<f64>(&random.text) {
                            Ok(func) => {
                                if func.var_names().len() > 1 {
                                    self.error = Some("too much variables, only one allowed".into());
                                }
                                random.func = Some(func);
                            },
                            Err(e) => {
                                self.error = Some(e.to_string());
                            }
                        };
                        self.data.push(random);
                    }
                });

                ui.add_space(4.5);

                {
                    let mut remove = None;

                    for (n, entry) in self.data.iter_mut().enumerate() {
                        let mut inner_changed = false;

                        ui.horizontal(|ui| {
                            ui.label(RichText::new(" ").strong().background_color(COLORS[n]));

                            if ui.add(TextEdit::singleline(&mut entry.text)).changed() {
                                if !entry.text.is_empty() {
                                    inner_changed = true;
                                } else {
                                    entry.func = None;
                                }

                                outer_changed = true;
                            }

                            if ui.button("X").clicked(){ remove = Some(n); }
                        });

                        if inner_changed {
                            self.error = None;

                            match exmex::parse::<f64>(&entry.text) {
                                Ok(func) => {
                                    if func.var_names().len() > 1 {
                                        self.error = Some("too much variables, only one allowed".into());
                                    }
                                    else {
                                        if self.stream_handle.is_some()  {
                                            if let Some(ref sink) = entry.sink {
                                                if !sink.empty() {
                                                    sink.clear();
                                                    if self.listening { sink.play(); }
                                                }
                                                sink.append(Math::new(unsafe{exmex::parse::<f32>(&entry.text).unwrap_unchecked()}));
                                            }
                                        }
                                        entry.func = Some(func);
                                    }
                                },
                                Err(e) => {
                                    self.error = Some(e.to_string());
                                }
                            };
                        }
                    }

                    if let Some(i) = remove {
                        self.data.remove(i);
                    }
                }

                if ui.add_enabled(
                    self.stream_handle.is_some(),
                    egui::Checkbox::new(&mut self.listening, "Listen live")
                ).changed() {
                    match self.listening {
                        true => self.all_sinks(Sink::play),
                        false => self.all_sinks(Sink::pause),
                    }
                }

                #[cfg(target_arch = "wasm32")]
                if outer_changed {
                    web::update_url(&self.data);
                }

                ui.separator();
                ui.label("BpptNG Grapher is a free and open source graphing calculator available online. Add functions on the left and they'll appear on the right in the graph.");
                ui.label("Hold control and scroll to zoom and drag to move around the graph.");
                ui.hyperlink_to("Source Code ", "https://github.com/p6nj/bpptng-grapher");
                #[cfg(not(target_arch = "wasm32"))]
                ui.hyperlink_to("View Graph Online", {
                    let mut base_url = "https://p6nj.github.io/bpptng-grapher/".to_string();
                    base_url.push_str(&web::url_string_from_data(&self.data));

                    base_url
                });
                #[cfg(target_arch = "wasm32")]
                ui.hyperlink_to("Download for Desktop", "https://github.com/p6nj/bpptng-grapher/releases");
                ui.separator();

                CollapsingHeader::new("Settings").show(ui, |ui| {
                    ui.add(Slider::new(&mut self.points, 10..=1000).text("Resolution"));
                    ui.label("Set to a lower resolution for better performance and a higher resolution for more accuracy. It's also pretty funny if you bring it down ridiculously low.");
                });
            });
        });
    }

    fn all_sinks(&mut self, f: impl Fn(&Sink)) {
        self.data
            .iter_mut()
            .filter(|f| f.sink.is_some())
            .for_each(|e| f(unsafe { e.sink.as_ref().unwrap_unchecked() }));
    }

    fn graph(&mut self, ctx: &egui::Context) {
        let lines: Vec<Line> = self
            .data
            .clone()
            .into_iter()
            .enumerate()
            .filter_map(|(n, entry)| {
                entry.func.map(|func| {
                    let name = format!("y = {}", entry.text);
                    let values = PlotPoints::from_explicit_callback(
                        move |x| func.eval(&[x]).unwrap_or_default(),
                        ..,
                        self.points,
                    );

                    Line::new(values).name(name).color(COLORS[n])
                })
            })
            .collect();

        let frame = Frame::window(&Style::default()).inner_margin(Vec2 { x: 0.0, y: 0.0 });

        egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
            if let Some(error) = &self.error {
                ui.centered_and_justified(|ui| {
                    ui.heading(format!("Error: {}", error));
                });
            } else {
                Plot::new("grapher")
                    .legend(Legend::default().text_style(TextStyle::Body))
                    .data_aspect(1.0)
                    .show(ui, |plot_ui| {
                        for line in lines {
                            plot_ui.line(line);
                        }
                    });
            }
        });
    }
}

impl Default for Grapher {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl App for Grapher {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(Visuals::dark());

        self.side_panel(ctx);
        self.graph(ctx);
    }
}

/// An entry in the sidebar
#[derive(Default)]
pub struct FunctionEntry {
    pub text: String,
    pub func: Option<FlatEx<f64>>,
    pub sink: Option<Sink>,
}

impl Clone for FunctionEntry {
    fn clone(&self) -> Self {
        Self {
            text: self.text.clone(),
            func: self.func.clone(),
            ..Default::default()
        }
    }
}

impl FunctionEntry {
    fn new(text: impl ToString, stream: Option<&OutputStreamHandle>) -> Self {
        Self {
            text: text.to_string(),
            sink: stream.and_then(|s| Sink::try_new(s).ok()),
            ..Default::default()
        }
    }
    fn random(stream: Option<&OutputStreamHandle>, index: &mut usize) -> Self {
        let rand = FunctionEntry::new(EXAMPLES[*index], stream);
        *index = index.add(1).clamp(0, EXAMPLES_NUMBER);
        rand
    }
}

const EXAMPLES_NUMBER: usize = 20;
const EXAMPLES: [&str; EXAMPLES_NUMBER] = [
    "e^x * sin(x)",                     // Exponential Spiral
    "e^(-x) * cos(x)",                  // Damped Oscillation
    "sinh(x) + cosh(x)",                // Hyperbolic Combination
    "ln(x) * tan(x)",                   // Logarithmic Spiral
    "sin(x) * cos(x) + tan(x)",         // Complex Trigonometric
    "sqrt(x^2 + x)",                    // Power and Root Combination
    "fract(x) + trunc(x)",              // Fractional Part and Integer Part
    "abs(sin(x))",                      // Absolute Sine
    "tan(sinh(x))",                     // Tangent with Hyperbolic Sine
    "exp(ln(x)) + log2(x)",             // Exponential Logarithm
    "asin(x) + acos(x) + atan(x)",      // Inverse Trigonometric
    "x^3 - 3*(x^2) + 3*x - 1 + sin(x)", // Polynomial with Trigonometric
    "signum(x) * tanh(x)",              // Signum with Hyperbolic Tangent
    "ceil(x) + floor(x)",               // Ceiling and Floor Combination
    "e^sin(x)",                         // Exponential and Sine Combination
    "cbrt(x) + cos(x)",                 // Cube Root and Cosine
    "τ * cos(π * x)",                   // Tau and Pi Constants
    "x^2 + log10(x)",                   // Quadratic Logarithm
    "log2(e^x)",                        // Logarithm with Exponential Base
    "sin(ln(x)) + cos(log10(x))",       // Combined Trigonometric and Logarithmic
];

#[test]
fn valid_example_expressions() {
    for example in EXAMPLES {
        exmex::parse::<f32>(example).unwrap().eval(&[1.0]).unwrap();
    }
}
