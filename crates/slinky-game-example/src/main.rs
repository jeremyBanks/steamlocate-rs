use eframe::egui;
use slinky::slinky;

fn main() {
    drop(slinky! {
        name: "Slinky Game Example",
        app_id from "steam_appid.txt",
        assets from "assets/3216842112",
        must_run_from_steam: true,
    });

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1280.0, 800.0)),
        fullscreen: true,
        ..Default::default()
    };
    eframe::run_native(
        "Slinky Game Example",
        options,
        Box::new(|_cc| Box::new(MyApp::default())),
    )
    .unwrap();
}

struct MyApp {
    name: String,
    age: u32,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            name: "Arthur".to_owned(),
            age: 42,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My egui Application");
            ui.horizontal(|ui| {
                let name_label = ui.label("Your name: ");
                ui.text_edit_singleline(&mut self.name)
                    .labelled_by(name_label.id);
            });
            ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
            if ui.button("Click each year").clicked() {
                self.age += 1;
            }
            ui.label(format!("Hello '{}', age {}", self.name, self.age));
        });
    }
}
// let event_loop = EventLoop::new();
// let window = WindowBuilder::new()
//     .with_title("Slinky Game Example")
//     .with_fullscreen(Some(Fullscreen::Borderless(None)))
//     .with_inner_size(Size::Logical(LogicalSize::new(1280., 800.)))
//     .build(&event_loop)
//     .unwrap();

// event_loop.run(move |event, _, control_flow| {
//     *control_flow = ControlFlow::Wait;

//     match event {
//         Event::WindowEvent {
//             event: WindowEvent::CloseRequested,
//             window_id,
//         } if window_id == window.id() => *control_flow = ControlFlow::Exit,
//         _ => (),
//     }
// });
// }
