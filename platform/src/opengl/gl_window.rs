use std::{num::NonZeroU32, sync::Arc};

use glutin::{
    config::{Config, ConfigTemplateBuilder},
    context::{
        ContextApi, ContextAttributesBuilder, GlProfile, NotCurrentContext, PossiblyCurrentContext,
        Version,
    },
    display::GetGlDisplay,
    prelude::*,
    surface::{Surface, SurfaceAttributesBuilder, SwapInterval, WindowSurface},
};
use glutin_winit::DisplayBuilder;
use raw_window_handle::HasRawWindowHandle;
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event_loop::EventLoop,
    window::{Fullscreen, Window, WindowBuilder},
};

use crate::{WindowAttributes, WindowSize};

const SAMPLES: u8 = 0;

pub struct GlWindow {
    // the surface must be dropped before the window.
    pub(crate) config: Config,
    pub(crate) surface: Surface<WindowSurface>,
    pub(crate) gl: Arc<glow::Context>,
    pub(crate) context: PossiblyCurrentContext,
    pub(crate) winit_window: Window,
    pub(crate) event_loop: Option<EventLoop<()>>,
}

impl GlWindow {
    pub fn new(config: &WindowAttributes) -> Self {
        let event_loop = EventLoop::new();

        let window_builder = WindowBuilder::new()
            .with_title(config.title.to_owned())
            .with_resizable(config.resizable)
            .with_inner_size::<PhysicalSize<u32>>(config.size.into())
            .with_position::<PhysicalPosition<u32>>(config.position.into())
            .with_fullscreen(config.fullscreen.then(|| Fullscreen::Borderless(None)));

        let gl_config_builder = ConfigTemplateBuilder::new();

        let (window, gl_config) =
            build_window_and_gl_config(window_builder, gl_config_builder, &event_loop);

        let surface = create_surface(&window, &gl_config);

        let gl_context = create_gl_context(&window, &gl_config).treat_as_possibly_current();
        gl_context
            .make_current(&surface)
            .expect("Failed to make context current");

        let gl = unsafe {
            glow::Context::from_loader_function_cstr(|addr| {
                gl_context.display().get_proc_address(addr)
            })
        };
        let gl = Arc::new(gl);

        let mut gl_window = Self {
            config: gl_config,
            surface,
            gl,
            context: gl_context,
            winit_window: window,
            event_loop: Some(event_loop),
        };

        // Set other window attributes not handled by the window builder
        gl_window.set_swap_interval(config.swap_interval);
        gl_window.set_minimised(config.minimised);
        gl_window.set_maximised(config.maximised);
        gl_window.set_min_window_size(config.min_size);
        gl_window.set_max_window_size(config.max_size);
        gl_window.set_cursor_grab(config.grabbed_cursor);
        gl_window.set_cursor_visible(config.visible_cursor);
        config.focused.then(|| gl_window.focus());

        gl_window
    }

    pub fn gl(&self) -> &glow::Context {
        self.gl.as_ref()
    }

    pub fn take_event_loop(&mut self) -> EventLoop<()> {
        self.event_loop.take().expect("Event loop already taken")
    }

    pub fn set_title(&self, title: &str) {
        self.winit_window.set_title(title);
    }

    pub fn set_min_window_size(&mut self, size: Option<WindowSize>) {
        self.winit_window
            .set_min_inner_size(size.map(winit::dpi::PhysicalSize::from));
    }

    pub fn set_max_window_size(&mut self, size: Option<WindowSize>) {
        self.winit_window
            .set_max_inner_size(size.map(winit::dpi::PhysicalSize::from));
    }

    pub fn set_surface_size(&self, size: WindowSize) {
        // Some platforms like EGL require resizing GL surface to update the size
        // Notable platforms here are Wayland and macOS, other don't require it
        // and the function is no-op, but it's wise to resize it for portability
        // reasons.
        self.surface.resize(
            &self.context,
            NonZeroU32::new(size.width.max(1) as u32).unwrap(),
            NonZeroU32::new(size.height.max(1) as u32).unwrap(),
        )
    }

    pub fn set_window_size(&self, size: WindowSize) {
        self.winit_window
            .set_inner_size(winit::dpi::PhysicalSize::from(size));
    }

    pub fn set_resizable(&self, resizable: bool) {
        self.winit_window.set_resizable(resizable);
    }

    pub fn set_maximised(&self, maximised: bool) {
        self.winit_window.set_maximized(maximised);
    }

    pub fn set_minimised(&self, minimised: bool) {
        self.winit_window.set_minimized(minimised);
    }

    pub fn set_cursor_grab(&self, grabbed: bool) {
        if grabbed {
            self.winit_window
                .set_cursor_grab(winit::window::CursorGrabMode::Locked)
                .or_else(|_| {
                    self.winit_window
                        .set_cursor_grab(winit::window::CursorGrabMode::Confined)
                })
                .unwrap();
        } else {
            self.winit_window
                .set_cursor_grab(winit::window::CursorGrabMode::None)
                .unwrap();
        }
    }

    pub fn set_cursor_visible(&self, visible: bool) {
        self.winit_window.set_cursor_visible(visible);
    }

    pub fn focus(&self) {
        self.winit_window.focus_window();
    }

    pub fn set_fullscreen(&self, fullscreen: bool) {
        self.winit_window.set_fullscreen(if fullscreen {
            Some(winit::window::Fullscreen::Borderless(None))
        } else {
            None
        });
    }

    pub fn set_swap_interval(&self, interval: usize) {
        let swap_interval = match interval {
            0 => SwapInterval::DontWait,
            _ => SwapInterval::Wait(NonZeroU32::new(interval as u32).unwrap()),
        };

        if let Err(res) = self.surface.set_swap_interval(&self.context, swap_interval) {
            println!("Error setting vsync: {:?}", res);
        }
    }

    pub fn request_redraw(&self) {
        self.winit_window.request_redraw();
    }

    pub fn swap_buffers(&self) {
        self.surface
            .swap_buffers(&self.context)
            .expect("Failed to swap buffers");
    }

    fn make_context_current(&self) {
        self.context
            .make_current(&self.surface)
            .expect("Failed to make context current");
    }

    // fn make_context_not_current(&self) {
    //     self.context
    //         .make_not_current()
    //         .expect("failed to make context not current");
    // }
}

fn build_window_and_gl_config(
    window_builder: winit::window::WindowBuilder,
    gl_config_builder: glutin::config::ConfigTemplateBuilder,
    event_loop: &EventLoop<()>,
) -> (Window, Config) {
    let display_builder = DisplayBuilder::new().with_window_builder(Some(window_builder));

    let (window, gl_config) = display_builder
        .build(&event_loop, gl_config_builder, |configs| {
            // iter configs to find one that best supports our template config
            configs
                .reduce(|accum, config| {
                    if accum.num_samples() == SAMPLES && accum.hardware_accelerated() {
                        accum
                    } else {
                        config
                    }
                })
                .unwrap()
        })
        .unwrap();

    (window.expect("Failed to create window"), gl_config)
}

fn create_surface(window: &Window, gl_config: &Config) -> Surface<WindowSurface> {
    let (width, height): (u32, u32) = window.inner_size().into();
    let raw_window_handle = window.raw_window_handle();
    let surface_attributes = SurfaceAttributesBuilder::<WindowSurface>::new().build(
        raw_window_handle,
        NonZeroU32::new(width).unwrap(),
        NonZeroU32::new(height).unwrap(),
    );

    unsafe {
        gl_config
            .display()
            .create_window_surface(&gl_config, &surface_attributes)
            .unwrap()
    }
}

fn create_gl_context(window: &Window, gl_config: &Config) -> NotCurrentContext {
    let raw_window_handle = window.raw_window_handle();

    let context_attributes = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::OpenGl(Some(Version::new(3, 3))))
        .with_profile(GlProfile::Core)
        .build(Some(raw_window_handle));

    // try Gles as backup
    let fallback_context_attributes = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::Gles(None))
        .with_profile(GlProfile::Core)
        .build(Some(raw_window_handle));

    let gl_display = gl_config.display();

    let not_current_gl_context = unsafe {
        gl_display
            .create_context(&gl_config, &context_attributes)
            .unwrap_or_else(|_| {
                gl_display
                    .create_context(&gl_config, &fallback_context_attributes)
                    .expect("Failed to create context")
            })
    };

    not_current_gl_context
}
