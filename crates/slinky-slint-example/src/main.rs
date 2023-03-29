use winit::dpi::LogicalSize;
use winit::dpi::Size;
use winit::window::Fullscreen;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    slinky::linky! {
        name: "Slinky Slint Steam Example",
        app_id from "steam_appid.txt",
        assets from "assets/steam",
        must_run_from_steam: true,
    };

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Slinky Slint Steam Example")
        .with_fullscreen(Some(Fullscreen::Borderless(None)))
        .with_inner_size(Size::Logical(LogicalSize::new(1280., 800.)))
        .build(&event_loop)
        .unwrap();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
            _ => (),
        }
    });
}
