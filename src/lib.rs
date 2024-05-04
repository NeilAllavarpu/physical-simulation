//! A physical simulation with corresponding graphical niceities

#![feature(lint_reasons)]

mod app;

use crate::app::AppWrapper;
use log::Level;
use std::panic;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use winit::event_loop::EventLoop;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
#[inline]
/// The global entry point. Initializes logging and begins the event loop.
///
/// # Errors
/// Returns an error if configuring logging or configuring/starting the event loop fails
pub fn main() -> Result<(), JsValue> {
    if cfg!(target_arch = "wasm32") {
        panic::set_hook(Box::new(console_error_panic_hook::hook));
        if console_log::init_with_level(Level::Trace).is_err() {
            return Err("Failed to attach to console".into());
        }
    } else {
        env_logger::init();
    }

    let event_loop = EventLoop::new()
        .map_err::<JsValue, _>(|err| format!("Failed to create event loop: {err:?}").into())?;

    if cfg!(target_arch = "wasm32") {
        use winit::platform::web::EventLoopExtWebSys;
        event_loop.spawn_app(AppWrapper::new());
    } else {
        event_loop
            .run_app(&mut AppWrapper::new())
            .map_err::<JsValue, _>(|err| format!("Failed to create event loop: {err:?}").into())?;
    }
    Ok(())
}
