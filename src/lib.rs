use eframe::{
    egui::{
        self,
        plot::{Legend, Line, Plot, Values},
        CollapsingHeader, Frame, RichText, ScrollArea, SidePanel, Slider, Style, TextEdit,
        TextStyle, Visuals,
    },
    epaint::{Color32, Vec2},
    epi::{self},
};
use exmex::{Express, FlatEx};

#[cfg(target_arch = "wasm32")]
use eframe::wasm_bindgen::{self, prelude::*};

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn start(canvas_id: &str) -> Result<(), eframe::wasm_bindgen::JsValue> {
    console_error_panic_hook::set_once();

    eframe::start_web(canvas_id, Box::new(Grapher::new()))
}

const EULER: &'static str = "2.7182818284590452353602874713527";

const COLORS: &'static [Color32; 18] = &[
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

#[derive(Clone, Debug)]
pub struct Grapher {
    data: Vec<FunctionEntry>,
    error: Option<String>,
    points: usize,
}

impl Grapher {
    pub fn new() -> Self {
        let mut data = Vec::new();
        data.push(FunctionEntry::new());

        Self {
            data,
            error: None,
            points: 500,
        }
    }

    fn side_panel(&mut self, ctx: &egui::Context) {
        SidePanel::left("left_panel").show(ctx, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(6.0);
                ui.heading("Grapher");
                ui.small("© 2022 Grant Handy");
                ui.separator();

                ui.horizontal_top(|ui| {
                    if self.data.len() < 18 && ui.button("Add").clicked() {
                        self.data.push(FunctionEntry::new());
                    }

                    if self.data.len() > 1 && ui.button("Delete").clicked() {
                        self.data.pop();
                    }
                });
                ui.add_space(4.5);

                for (n, entry) in self.data.iter_mut().enumerate() {
                    let mut changed = false;

                    let hint_text = match n {
                        0 => "x^2",
                        1 => "sin(x)",
                        2 => "x+2",
                        3 => "x*3",
                        4 => "abs(x)",
                        5 => "cos(x)",
                        _ => "",
                    };

                    ui.horizontal(|ui| {
                        ui.label(RichText::new(" ").strong().background_color(COLORS[n]));

                        if ui.add(TextEdit::singleline(&mut entry.text).hint_text(hint_text)).changed() {
                            if entry.text != "" {
                                changed = true;
                            } else {
                                entry.func = None;
                            }
                        }
                    });

                    if changed {
                        self.error = None;

                        // for nathan
                        let text = &entry.text.replace("e", EULER);

                        entry.func = match exmex::parse::<f64>(text) {
                            Ok(func) => Some(func),
                            Err(e) => {
                                self.error = Some(e.to_string());
                                continue;
                            }
                        };
                    }
                }

                ui.separator();
                ui.label("Grapher is a free and open source graphing calculator available online. Add functions on the left and they'll appear on the right in the graph.");
                ui.label("Hold control and scroll to zoom and drag to move around the graph.");
                ui.hyperlink_to("Source Code ", "https://github.com/grantshandy/grapher");
                ui.separator();
                CollapsingHeader::new("Settings").show(ui, |ui| {
                    ui.add(Slider::new(&mut self.points, 10..=1000).text("Resolution"));
                    ui.label("Set to a lower resolution for better performance and a higher resolution for more accuracy. It's also pretty funny if you bring it down ridiculously low.");
                });
            });
        });
    }

    fn graph(&mut self, ctx: &egui::Context) {
        let mut lines: Vec<Line> = Vec::new();

        for (n, entry) in self.data.clone().into_iter().enumerate() {
            if let Some(func) = entry.func {
                let name = format!("y = {}", entry.text.clone());
                let values = Values::from_explicit_callback(
                    move |x| match func.eval(&[x]) {
                        Ok(y) => y,
                        Err(e) => {
                            // DIRTY HACK THEY DON'T WANT YOU TO KNOW ABOUT!
                            if e.to_string() == "parsed expression contains 0 vars but passed slice has 1 elements" {
                                entry.text.parse().unwrap_or(0.0)
                            } else {
                                0.0
                            }
                        }
                    },
                    ..,
                    self.points,
                );

                let line = Line::new(values).name(name).color(COLORS[n]);

                lines.push(line);
            }
        }

        let frame = Frame::window(&Style::default()).margin(Vec2 { x: 0.0, y: 0.0 });

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

impl epi::App for Grapher {
    fn name(&self) -> &str {
        "Grapher"
    }

    // imma assume you aren't this rich
    fn max_size_points(&self) -> Vec2 {
        Vec2 {
            x: 4096.0,
            y: 2160.0,
        }
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &epi::Frame) {
        ctx.set_visuals(Visuals::dark());

        self.side_panel(ctx);
        self.graph(ctx);
    }
}

#[derive(Clone, Debug)]
struct FunctionEntry {
    pub text: String,
    pub func: Option<FlatEx<f64>>,
}

impl FunctionEntry {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            func: None,
        }
    }
}
