//! Core application logic
//!
//! This consists of the main implementation logic, excluding event handling

use core::iter;
use std::rc::Rc;

use wgpu::{LoadOp, StoreOp, SurfaceError};
use winit::{dpi, window::Window};

/// The main application struct, managing the rendering process and all application state
pub(super) struct Application<'app> {
    /// The surface to render to
    surface: wgpu::Surface<'app>,
    /// The device doing the rendering and computation (e.g. GPU)
    device: wgpu::Device,
    /// The queue of commands for the device
    queue: wgpu::Queue,
    /// Configuration for the surface
    config: wgpu::SurfaceConfiguration,
    /// The window to render to
    window: Rc<Window>,
}

#[derive(Debug)]
/// Errors that may arise from resizing the application
pub(super) enum ResizeError {
    /// One of the given dimensions was zero
    ZeroDimension,
}

#[derive(Debug)]
pub(super) enum AppError {
    /// Error creating the surface to render to
    Surface(wgpu::CreateSurfaceError),
    /// Could not get an adapter
    Adapter,
    /// Could not get a handle to the device
    Device(wgpu::RequestDeviceError),
    /// No surface formats found,
    NoSurfaceFormats,
    /// No present mode found
    NoPresentMode,
    /// No alpha mode found
    NoAlphaMode,
}

impl Application<'_> {
    /// Creates a new application that renders using the given window.
    /// Note that the surface is not necessarily configured yet, and should be done separately.
    pub async fn new(window: Rc<Window>) -> Result<Self, AppError> {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        let surface = instance
            .create_surface(Rc::clone(&window))
            .map_err(AppError::Surface)?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or(AppError::Adapter)?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web, we'll have to disable some.
                    required_limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                },
                None, // Trace path
            )
            .await
            .map_err(AppError::Device)?;

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result in all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_caps
                .formats
                .iter()
                .find(|format| format.is_srgb())
                .or(surface_caps.formats.first())
                .copied()
                .ok_or(AppError::NoSurfaceFormats)?,
            width: size.width,
            height: size.height,
            present_mode: surface_caps
                .present_modes
                .first()
                .copied()
                .ok_or(AppError::NoPresentMode)?,
            alpha_mode: surface_caps
                .alpha_modes
                .first()
                .copied()
                .ok_or(AppError::NoAlphaMode)?,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        Ok(Self {
            surface,
            device,
            queue,
            config,
            window,
        })
    }

    /// Attempts to resize the window. Returns any errors that may occur
    pub fn resize(&mut self, new_size: dpi::PhysicalSize<u32>) -> Result<(), ResizeError> {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            Ok(())
        } else {
            Err(ResizeError::ZeroDimension)
        }
    }

    /// Updates the application state
    pub fn update(&mut self) {}

    /// Returns the window to which this application is attached
    pub fn window(&self) -> &Window {
        &self.window
    }

    /// Renders the current state of the application to the surface
    pub fn render(&mut self) -> Result<(), SurfaceError> {
        /// Background color of the simulation
        const BACKGROUND_COLOR: wgpu::Color = wgpu::Color {
            r: 0.0,
            g: 0.372_549_03,
            b: 0.372_549_03,
            a: 1.0,
        };
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: LoadOp::Clear(BACKGROUND_COLOR),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
