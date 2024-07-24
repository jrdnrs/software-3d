use winit::event::{Event as WinitEvent, WindowEvent as WinitWindowEvent};

use crate::{
    event::Event,
    window::{Window, WindowPosition, WindowSize},
    Renderer,
};

pub trait WindowApplication<R: Renderer>: Sized + 'static {
    fn get_window(&self) -> &Window<R>;
    fn get_window_mut(&mut self) -> &mut Window<R>;
    fn on_event(&mut self, event: &Event);

    /// Default implementation of `run` for `WindowApplication` handles the backend event loop
    /// and calls the user defined event handler `on_event`.
    ///
    /// Typical window functions are handled by this, such as resizing, moving, closing, etc.
    fn run(mut self) -> ! {
        let event_loop = self.get_window_mut().sys_window.take_event_loop();

        event_loop.run(move |event, window_target, control_flow| {
            // TODO: Allow user to define control flow (default is Poll)

            let window = self.get_window_mut();

            // Let renderer access the event
            window.renderer.on_event(&event);

            // Call user defined event handler
            let user_event = Event::try_from_winit_event(&event, window);
            let _ = user_event.map(|event| self.on_event(&event));

            // Native handling of winit events
            match event {
                WinitEvent::Resumed => {}

                WinitEvent::Suspended => {}

                WinitEvent::DeviceEvent { event, .. } => match event {
                    _ => (),
                },

                WinitEvent::WindowEvent { event, .. } => match event {
                    WinitWindowEvent::Resized(ref size) => {
                        let width = size.width as usize;
                        let height = size.height as usize;

                        let window = self.get_window_mut();

                        if size.width == 0 && size.height == 0 {
                            window.attributes.minimised = true;
                        } else {
                            window.attributes.minimised = false;

                            // let target_aspect_ratio = 4.0 / 3.0;
                            // let aspect_ratio = width as f32 / height as f32;

                            // println!("Aspect ratio: {}", aspect_ratio);

                            // if (aspect_ratio - target_aspect_ratio).abs() > 0.001 {
                            //     let size = WindowSize::new(
                            //         width,
                            //         (width as f32 / target_aspect_ratio).round() as usize,
                            //     );
                            //     window.set_size(size);

                            //     println!("Resizing to: {:?}", size);

                            //     return;
                            // }

                            let size = WindowSize::new(width, height);
                            window.attributes.size = size;
                            window.set_surface_size(size);
                        }
                    }

                    WinitWindowEvent::Moved(ref position) => {
                        self.get_window_mut().attributes.position =
                            WindowPosition::new(position.x as usize, position.y as usize);
                    }

                    WinitWindowEvent::Focused(focused) => {
                        self.get_window_mut().attributes.focused = focused;
                    }

                    WinitWindowEvent::CloseRequested => control_flow.set_exit(),

                    _ => {}
                },

                WinitEvent::MainEventsCleared => {
                    self.get_window().request_redraw();
                }

                WinitEvent::RedrawRequested(_) => {
                    self.get_window_mut().swap_buffers();
                }

                WinitEvent::LoopDestroyed => {
                    self.get_window_mut().dispose();
                }

                _ => (),
            }
        });
    }
}
