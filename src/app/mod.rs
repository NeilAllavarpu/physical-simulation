//! Core application logic
//!
//! This consists of the body of the event loop as well as managing all the state regarding the whole application

mod app;
use std::rc::Rc;

use self::app::Application;
use log::{error, info, warn};
use pollster::block_on;
use wgpu::SurfaceError;
use winit::{
    application::ApplicationHandler,
    event::{self, WindowEvent},
    event_loop,
    platform::web::WindowAttributesExtWebSys,
    window::{self, WindowAttributes},
};

/// A wrapper around the application state to handle creation/destruction of windows.
/// Manages and dispatches events.
pub struct AppWrapper<'app> {
    /// The application. `None` if the application has not yet been initialized
    app: Option<Application<'app>>,
}

impl AppWrapper<'_> {
    /// Creates a new, uninitialized application
    pub const fn new() -> Self {
        Self { app: None }
    }
}

impl ApplicationHandler for AppWrapper<'_> {
    fn resumed(&mut self, event_loop: &event_loop::ActiveEventLoop) {
        if self.app.is_none() {
            let window = Rc::new(if cfg!(target_arch = "wasm32") {
                use wasm_bindgen::JsCast;
                // let canvas =
                let document = web_sys::window()
                    .expect("Window should be loaded")
                    .document()
                    .expect("Document should be loaded");
                let attributes = WindowAttributes::default().with_prevent_default(false);
                if let Some(canvas) = document
                    .get_element_by_id("canvas")
                    .and_then(|canvas| canvas.dyn_into::<web_sys::HtmlCanvasElement>().ok())
                {
                    event_loop
                        .create_window(attributes.with_canvas(Some(canvas)))
                        .expect("Creating a window should not fail")
                } else {
                    unreachable!("Canvas should exist");
                }
            } else {
                unimplemented!("Application resuming not implemented for current platform");
            });
            self.app = Some(
                block_on(Application::new(Rc::clone(&window)))
                    .expect("Creating application should not fail"),
            );
        }
    }

    fn window_event(
        &mut self,
        _: &event_loop::ActiveEventLoop,
        window_id: window::WindowId,
        event: WindowEvent,
    ) {
        if let Some(ref mut app) = self.app {
            assert_eq!(
                app.window().id(),
                window_id,
                "Only the created window should emit events"
            );
            match event {
                WindowEvent::RedrawRequested => {
                    app.update();
                    match app.render() {
                        Ok(()) => {}
                        // Reconfigure the surface if lost
                        Err(SurfaceError::Lost) => app
                            .resize(app.window().inner_size())
                            .expect("Window size should be valid"),

                        Err(err) => error!("{:?}", err),
                    }
                    app.window().request_redraw();
                }
                WindowEvent::Resized(physical_size) => {
                    app.resize(physical_size).expect("New size should be valid");
                }
                event => {
                    info!("Received window event {event:?}");
                }
            }
        } else {
            info!("[UNINITIALIZED] Receieved window event {event:?} from window {window_id:?}");
        }
    }

    fn new_events(&mut self, _: &event_loop::ActiveEventLoop, cause: event::StartCause) {
        info!("Received new OS event: {cause:?}");
    }

    fn user_event(&mut self, _: &event_loop::ActiveEventLoop, (): ()) {
        #[expect(clippy::unreachable, reason = "No user events should ever be sent")]
        {
            unreachable!("Custom event sent");
        }
    }

    fn device_event(
        &mut self,
        _: &event_loop::ActiveEventLoop,
        device_id: event::DeviceId,
        event: event::DeviceEvent,
    ) {
        info!("Received new device event {event:?} from device {device_id:?}");
    }

    fn about_to_wait(&mut self, _: &event_loop::ActiveEventLoop) {}

    fn suspended(&mut self, _: &event_loop::ActiveEventLoop) {
        info!("Application suspended");
    }

    fn exiting(&mut self, _: &event_loop::ActiveEventLoop) {
        info!("Application exiting");
    }

    fn memory_warning(&mut self, _: &event_loop::ActiveEventLoop) {
        warn!("Memory warning");
    }
}
