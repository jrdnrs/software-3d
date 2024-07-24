mod free_movement;

use std::time::Instant;

use free_movement::FreeMovement;
use maths::linear::Mat4f;
use platform::{
    Event, Input, KeyCode, MouseButton, PixelRenderer, Window, WindowApplication, WindowAttributes,
    WindowSize,
};
use renderer::Renderer;

struct Demo {
    window: Window<PixelRenderer>,
    input: Input,
    renderer: Renderer,
    player_camera: FreeMovement,

    time_buffer: [f32; 128],
}

impl Demo {
    pub fn new() -> Self {
        let window = Window::new(WindowAttributes {
            title: String::from("software renderer"),
            size: WindowSize::new(1200, 800),
            swap_interval: 0,
            fps_limit: Some(60),
            ..Default::default()
        });
        let input = Input::new();
        let renderer = Renderer::new(800, 600, 75.0);
        let player_camera = FreeMovement::new(25.0, 50.0, 33.0, 5.0);

        Self {
            window,
            input,
            renderer,
            player_camera,

            time_buffer: [0.0; 128],
        }
    }

    pub fn init(&mut self) {
        // let model_id = self
        //     .renderer
        //     .assets_mut()
        //     .model_from_obj_path(
        //         "Tom Nook/tomNook.obj",
        //         true,
        //         false,
        //         true,
        //     )
        //     .unwrap();

        // let instance_id = self
        //     .renderer
        //     .assets_mut()
        //     .spawn_model_instance(model_id, &Mat4f::translate(0.0, -10.0, 15.0));
    }

    fn handle_mouse_grab(&mut self) {
        if !self.window.cursor_grabbed() && self.input.mouse.is_button_pressed(MouseButton::Left) {
            self.window.set_cursor_grab(true);
            self.window.set_cursor_visible(false);
        }

        if self.window.cursor_grabbed() && self.input.keyboard.is_key_pressed(KeyCode::Escape) {
            self.window.set_cursor_grab(false);
            self.window.set_cursor_visible(true);
        }
    }
}

impl WindowApplication<PixelRenderer> for Demo {
    fn get_window(&self) -> &Window<PixelRenderer> {
        &self.window
    }

    fn get_window_mut(&mut self) -> &mut Window<PixelRenderer> {
        &mut self.window
    }

    fn on_event(&mut self, event: &Event) {
        match event {
            Event::WindowEvent(event) => match event {
                platform::WindowEvent::Resized(size) => {
                    self.renderer.update_viewport(size.width, size.height);
                }

                _ => {}
            },

            Event::PointerEvent(_) | Event::KeyboardEvent(_) => self.input.handle_event(event),

            Event::RenderEvent(event) => match event {
                platform::RenderEvent::RedrawRequested(timings) => {
                    self.handle_mouse_grab();
                    self.player_camera.update(
                        &self.input,
                        self.renderer.camera_mut(),
                        self.window.cursor_grabbed(),
                        timings.delta_seconds,
                    );

                    let start = Instant::now();
                    self.renderer.render();
                    let elapsed = start.elapsed().as_secs_f32();
                    self.time_buffer[timings.frame_count % 128] = elapsed;
                    if timings.frame_count % 128 == 0 {
                        let avg = self.time_buffer.iter().sum::<f32>() / 128.0;
                        print!("Render time: {:.4}ms\r", avg * 1000.0);
                        std::io::Write::flush(&mut std::io::stdout()).unwrap();
                    }

                    self.window.renderer_context().set_pixels(
                        self.renderer.pixels_bytes(),
                        self.renderer.internal_width(),
                        self.renderer.internal_height(),
                    );

                    self.input.clear();
                }
            },

            _ => {}
        }
    }
}

fn main() {
    let mut game = Demo::new();
    game.init();

    game.run();
}
