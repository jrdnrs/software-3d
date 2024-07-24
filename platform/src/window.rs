use crate::{frame_tracker::FrameTracker, opengl::GlWindow};

pub trait Renderer {
    type Context<'a>
    where
        Self: 'a;

    fn new(window: &GlWindow) -> Self;
    fn clear(&mut self, window: &GlWindow);
    fn draw(&mut self, window: &GlWindow);
    fn swap_buffers(&mut self, window: &GlWindow);
    fn set_viewport(&mut self, window: &GlWindow, width: usize, height: usize);
    fn dispose(&mut self, window: &GlWindow);
    fn on_event(&mut self, event: &winit::event::Event<()>);
    fn context<'a>(&'a mut self, window: &'a GlWindow) -> Self::Context<'_>;
}

pub struct Window<R: Renderer> {
    pub(crate) sys_window: GlWindow,
    pub(crate) renderer: R,
    pub(crate) attributes: WindowAttributes,
    pub(crate) frame_tracker: FrameTracker,
}

impl<R: Renderer> Window<R> {
    pub fn new(config: WindowAttributes) -> Self {
        let sys_window = GlWindow::new(&config);
        let renderer = R::new(&sys_window);
        let frame_tracker = FrameTracker::new(config.fps_limit.unwrap_or_default() as f32);

        Self {
            sys_window,
            renderer,
            attributes: config,
            frame_tracker,
        }
    }

    pub fn set_title(&mut self, title: &str) {
        if self.attributes.title == title {
            return;
        };

        self.attributes.title = title.to_owned();
        self.sys_window.set_title(title);
    }

    pub fn title(&self) -> &str {
        &self.attributes.title
    }

    pub fn set_size(&mut self, size: WindowSize) {
        if self.attributes.size == size {
            return;
        }

        self.attributes.maximised = false;
        self.attributes.size = size;
        self.sys_window.set_window_size(size);

        // surface and renderer viewport are resized in the event loop in response
        // to the winit event emitted by set_window_size
    }

    pub fn size(&self) -> WindowSize {
        self.attributes.size
    }

    pub fn set_resizable(&mut self, resizable: bool) {
        if self.attributes.resizable == resizable {
            return;
        }

        self.attributes.resizable = resizable;
        self.sys_window.set_resizable(resizable);
    }

    pub fn resizable(&self) -> bool {
        self.attributes.resizable
    }

    pub fn set_maximised(&mut self, maximised: bool) {
        if self.attributes.maximised == maximised {
            return;
        }

        self.attributes.maximised = maximised;
        self.sys_window.set_maximised(maximised);
    }

    pub fn maximised(&self) -> bool {
        self.attributes.maximised
    }

    pub fn set_minimised(&mut self, minimised: bool) {
        if self.attributes.minimised == minimised {
            return;
        }

        self.attributes.minimised = minimised;
        self.sys_window.set_minimised(minimised);
    }

    pub fn minimised(&self) -> bool {
        self.attributes.minimised
    }

    pub fn focus(&mut self) {
        self.sys_window.focus();
    }

    pub fn focused(&self) -> bool {
        self.attributes.focused
    }

    pub fn set_cursor_grab(&mut self, grabbed: bool) {
        if self.attributes.grabbed_cursor == grabbed {
            return;
        }

        self.attributes.grabbed_cursor = grabbed;
        self.sys_window.set_cursor_grab(grabbed);
    }

    pub fn cursor_grabbed(&self) -> bool {
        self.attributes.grabbed_cursor
    }

    pub fn set_cursor_visible(&mut self, visible: bool) {
        self.sys_window.set_cursor_visible(visible);
        self.attributes.visible_cursor = visible;
    }

    pub fn cursor_visible(&self) -> bool {
        self.attributes.visible_cursor
    }

    pub fn set_fullscreen(&mut self, fullscreen: bool) {
        self.sys_window.set_fullscreen(fullscreen);
    }

    pub fn fullscreen(&self) -> bool {
        self.attributes.fullscreen
    }

    pub fn clear(&mut self) {
        self.renderer.clear(&self.sys_window);
    }

    pub fn renderer_context<'a>(&'a mut self) -> R::Context<'a> {
        self.renderer.context(&self.sys_window)
    }

    pub(crate) fn request_redraw(&self) {
        self.sys_window.request_redraw();
    }

    pub(crate) fn swap_buffers(&mut self) {
        self.renderer.draw(&self.sys_window);
        self.frame_tracker.sleep_until_target();
        self.sys_window.swap_buffers();
        self.renderer.swap_buffers(&self.sys_window);
        self.frame_tracker.update();
    }

    pub(crate) fn set_surface_size(&mut self, size: WindowSize) {
        self.sys_window.set_surface_size(size);
        self.renderer
            .set_viewport(&self.sys_window, size.width, size.height);
    }

    pub(crate) fn dispose(&mut self) {
        self.renderer.dispose(&self.sys_window);
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WindowSize {
    pub width: usize,
    pub height: usize,
}

impl WindowSize {
    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height }
    }
}

impl From<WindowSize> for winit::dpi::PhysicalSize<u32> {
    fn from(value: WindowSize) -> Self {
        winit::dpi::PhysicalSize::new(value.width as u32, value.height as u32)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WindowPosition {
    pub x: usize,
    pub y: usize,
}

impl WindowPosition {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

impl From<WindowPosition> for winit::dpi::PhysicalPosition<u32> {
    fn from(value: WindowPosition) -> Self {
        winit::dpi::PhysicalPosition::new(value.x as u32, value.y as u32)
    }
}

pub struct WindowAttributes {
    pub title: String,
    pub resizable: bool,
    pub maximised: bool,
    pub minimised: bool,
    pub focused: bool,
    pub fullscreen: bool,
    pub position: WindowPosition,
    pub size: WindowSize,
    pub max_size: Option<WindowSize>,
    pub min_size: Option<WindowSize>,
    pub swap_interval: usize,
    pub fps_limit: Option<usize>,
    pub grabbed_cursor: bool,
    pub visible_cursor: bool,
}

impl Default for WindowAttributes {
    fn default() -> Self {
        Self {
            title: String::from("App"),
            resizable: true,
            maximised: false,
            minimised: false,
            focused: false,
            fullscreen: false,
            position: WindowPosition::new(0, 0),
            size: WindowSize::new(640, 480),
            max_size: None,
            min_size: None,
            swap_interval: 1,
            fps_limit: None,
            grabbed_cursor: false,
            visible_cursor: true,
        }
    }
}
