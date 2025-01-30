mod app;

use crate::app::App;
use winit::event_loop::{ControlFlow, EventLoop};

#[cfg(target_arch = "wasm32")]
fn run_app(event_loop: EventLoop<()>, app: App) {
    // Sets up panics to go to the console.error in browser environments
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");

    // Runs the app async via the browsers event loop
    use winit::platform::web::EventLoopExtWebSys;
    wasm_bindgen_futures::spawn_local(async move {
        event_loop.spawn_app(app);
    });
}

#[cfg(not(target_arch = "wasm32"))]
fn run_app(event_loop: EventLoop<()>, mut app: App) {
    // Allows the setting of the log level through RUST_LOG env var.
    // It also allows wgpu logs to be seen.
    env_logger::init();

    // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
    // dispatched any events. This is ideal for games and similar applications.
    event_loop.set_control_flow(ControlFlow::Poll);

    // ControlFlow::Wait pauses the event loop if no events are available to process.
    // This is ideal for non-game applications that only update in response to user
    // input, and uses significantly less power/CPU time than ControlFlow::Poll.
    //event_loop.set_control_flow(ControlFlow::Wait);

    // Runs the app on the current thread.
    let _ = event_loop.run_app(&mut app);
}

fn main() {
    // <T> (T -> AppEvent) extends regular platform specific events (resize, mouse, etc.).
    // This allows the our app to inject custom events and handle them alongside regular ones.
    // let event_loop = EventLoop::with_user_event().build().unwrap();
    let event_loop = EventLoop::<()>::new().unwrap();

    // TODO: what is App and why do we need it?
    let app = App::default();
    // let app = App::new(event_loop.create_proxy());

    run_app(event_loop, app);
}
