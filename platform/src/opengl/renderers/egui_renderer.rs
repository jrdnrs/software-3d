use egui::{
    epaint::ClippedShape, Context, Modifiers, MouseWheelUnit, Pos2, RawInput, Rect, TexturesDelta,
    Vec2,
};
use egui_glow::Painter;

use crate::{opengl::GlWindow, Renderer};

pub struct EguiRenderer {
    egui: Context,
    egui_input: RawInput,
    egui_painter: Painter,

    // Cached state to pass to egui
    max_texture_side: usize,
    logical_window_size: Rect,
    window_focus: bool,
    mouse_pos: Pos2,
    modifiers: Modifiers,

    // Egui feedback for the renderer
    shapes: Vec<ClippedShape>,
    textures_delta: TexturesDelta,
    pixels_per_point: f32,
}

impl EguiRenderer {
    pub fn new(gl_window: &GlWindow) -> Self {
        let egui = Context::default();
        let egui_input = RawInput::default();
        let egui_painter = Painter::new(gl_window.gl.clone(), "", None).unwrap();
        let max_texture_side = egui_painter.max_texture_side();
        let pixels_per_point = egui.pixels_per_point();
        let dimensions: [u32; 2] = gl_window.winit_window.inner_size().into();
        let logical_window_size = Rect::from_min_size(
            Pos2::default(),
            Vec2::new(
                dimensions[0] as f32 / pixels_per_point,
                dimensions[1] as f32 / pixels_per_point,
            ),
        );
        let window_focus = gl_window.winit_window.has_focus();

        Self {
            egui,
            egui_input,
            egui_painter,
            max_texture_side,
            logical_window_size,
            window_focus,
            mouse_pos: Pos2::default(),
            modifiers: Modifiers::default(),
            shapes: Vec::new(),
            textures_delta: TexturesDelta::default(),
            pixels_per_point,
        }
    }

    pub fn draw(&mut self, gl_window: &GlWindow) {
        let shapes = std::mem::take(&mut self.shapes);
        let mut textures_delta = std::mem::take(&mut self.textures_delta);

        for (id, image_delta) in textures_delta.set {
            self.egui_painter.set_texture(id, &image_delta);
        }

        let clipped_primitives = self.egui.tessellate(shapes, self.pixels_per_point);
        let dimensions: [u32; 2] = gl_window.winit_window.inner_size().into();
        self.egui_painter
            .paint_primitives(dimensions, self.pixels_per_point, &clipped_primitives);

        for id in textures_delta.free.drain(..) {
            self.egui_painter.free_texture(id);
        }
    }

    pub fn clear(&mut self, gl_window: &GlWindow) {
        self.egui_painter.clear(
            gl_window.winit_window.inner_size().into(),
            [0.0, 0.0, 0.0, 1.0],
        )
    }

    pub fn update_ui(&mut self, update: impl FnOnce(&Context)) {
        self.egui_input.screen_rect = Some(self.logical_window_size);
        self.egui_input.max_texture_side = Some(self.max_texture_side);
        self.egui_input.modifiers = self.modifiers;
        self.egui_input.focused = self.window_focus;

        let full_output = self.egui.run(self.egui_input.take(), update);

        self.shapes = full_output.shapes;
        self.textures_delta.append(full_output.textures_delta);
        self.pixels_per_point = full_output.pixels_per_point;
    }

    pub fn dispose(&mut self, _gl_window: &GlWindow) {
        self.egui_painter.destroy();
    }

    pub fn on_event(&mut self, event: &winit::event::Event<'_, ()>) {
        match event {
            winit::event::Event::WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::CursorMoved { position, .. } => {
                    self.mouse_pos = Pos2::new(position.x as f32, position.y as f32);
                    self.egui_input
                        .events
                        .push(egui::Event::PointerMoved(self.mouse_pos));
                }

                winit::event::WindowEvent::MouseInput { state, button, .. } => {
                    match try_winit_mouse_button_to_egui(*button) {
                        Ok(button) => {
                            self.egui_input.events.push(egui::Event::PointerButton {
                                pos: self.mouse_pos,
                                button,
                                pressed: *state == winit::event::ElementState::Pressed,
                                modifiers: self.modifiers,
                            });
                        }
                        Err(e) => eprintln!("Error: {}", e),
                    }
                }

                winit::event::WindowEvent::MouseWheel { delta, .. } => {
                    match delta {
                        winit::event::MouseScrollDelta::LineDelta(x, y) => {
                            let delta = egui::Vec2::new(*x, *y);
                            self.egui_input.events.push(egui::Event::MouseWheel {
                                unit: MouseWheelUnit::Line,
                                delta,
                                modifiers: self.modifiers,
                            });
                        }
                        winit::event::MouseScrollDelta::PixelDelta(pos) => {
                            let delta = egui::Vec2::new(pos.x as f32, pos.y as f32);
                            self.egui_input.events.push(egui::Event::MouseWheel {
                                unit: MouseWheelUnit::Point,
                                delta,
                                modifiers: self.modifiers,
                            });
                        }
                    };
                }

                winit::event::WindowEvent::KeyboardInput { input, .. } => {
                    if let Some(key) = input.virtual_keycode {
                        match try_winit_key_to_egui(key) {
                            Ok(key) => {
                                self.egui_input.events.push(egui::Event::Key {
                                    key,
                                    physical_key: None,
                                    pressed: input.state == winit::event::ElementState::Pressed,
                                    repeat: false, // handled by egui
                                    modifiers: self.modifiers,
                                });
                            }
                            Err(e) => eprintln!("Error: {}", e),
                        }
                    }
                }

                winit::event::WindowEvent::CursorLeft { .. } => {
                    self.egui_input.events.push(egui::Event::PointerGone);
                }

                winit::event::WindowEvent::Focused(focused) => {
                    self.egui_input
                        .events
                        .push(egui::Event::WindowFocused(*focused));
                    self.window_focus = *focused;
                }

                winit::event::WindowEvent::ModifiersChanged(new_modifiers) => {
                    self.modifiers = Modifiers {
                        alt: new_modifiers.alt(),
                        ctrl: new_modifiers.ctrl(),
                        shift: new_modifiers.shift(),
                        mac_cmd: cfg!(target_os = "macos") && new_modifiers.logo(),
                        command: if cfg!(target_os = "macos") {
                            new_modifiers.logo()
                        } else {
                            new_modifiers.ctrl()
                        },
                    };
                }

                winit::event::WindowEvent::Resized(size) => {
                    self.logical_window_size = Rect::from_min_size(
                        Pos2::new(0.0, 0.0),
                        Vec2::new(size.width as f32, size.height as f32) / self.pixels_per_point,
                    );
                }

                _ => {}
            },

            _ => {}
        }
    }
}

pub struct EguiRendererContext<'a> {
    egui_renderer: &'a mut EguiRenderer,
}

impl<'a> EguiRendererContext<'a> {
    pub fn new(egui_renderer: &'a mut EguiRenderer) -> Self {
        Self { egui_renderer }
    }

    pub fn egui_context(&mut self) -> &mut Context {
        &mut self.egui_renderer.egui
    }

    pub fn update_ui(&mut self, update: impl FnOnce(&Context)) {
        self.egui_renderer.update_ui(update);
    }
}

impl Renderer for EguiRenderer {
    type Context<'a> = EguiRendererContext<'a>;

    fn new(window: &GlWindow) -> EguiRenderer {
        EguiRenderer::new(window)
    }

    fn clear(&mut self, window: &GlWindow) {
        EguiRenderer::clear(self, window);
    }

    fn draw(&mut self, window: &GlWindow) {
        EguiRenderer::draw(self, window);
    }

    fn swap_buffers(&mut self, window: &GlWindow) {
        // nothing to swap
    }

    fn set_viewport(&mut self, window: &GlWindow, width: usize, height: usize) {
        // egui painter sets viewport when drawing
    }

    fn dispose(&mut self, window: &GlWindow) {
        EguiRenderer::dispose(self, window);
    }

    fn on_event(&mut self, event: &winit::event::Event<()>) {
        EguiRenderer::on_event(self, event);
    }

    fn context<'a>(&'a mut self, window: &'a GlWindow) -> Self::Context<'a> {
        EguiRendererContext::new(self)
    }
}

fn try_winit_mouse_button_to_egui(
    button: winit::event::MouseButton,
) -> Result<egui::PointerButton, &'static str> {
    match button {
        winit::event::MouseButton::Left => Ok(egui::PointerButton::Primary),
        winit::event::MouseButton::Right => Ok(egui::PointerButton::Secondary),
        winit::event::MouseButton::Middle => Ok(egui::PointerButton::Middle),
        winit::event::MouseButton::Other(_) => Err("Unsupported mouse button"),
    }
}

fn try_winit_key_to_egui(key: winit::event::VirtualKeyCode) -> Result<egui::Key, &'static str> {
    match key {
        winit::event::VirtualKeyCode::Key1 => Ok(egui::Key::Num1),
        winit::event::VirtualKeyCode::Key2 => Ok(egui::Key::Num2),
        winit::event::VirtualKeyCode::Key3 => Ok(egui::Key::Num3),
        winit::event::VirtualKeyCode::Key4 => Ok(egui::Key::Num4),
        winit::event::VirtualKeyCode::Key5 => Ok(egui::Key::Num5),
        winit::event::VirtualKeyCode::Key6 => Ok(egui::Key::Num6),
        winit::event::VirtualKeyCode::Key7 => Ok(egui::Key::Num7),
        winit::event::VirtualKeyCode::Key8 => Ok(egui::Key::Num8),
        winit::event::VirtualKeyCode::Key9 => Ok(egui::Key::Num9),
        winit::event::VirtualKeyCode::Key0 => Ok(egui::Key::Num0),

        winit::event::VirtualKeyCode::A => Ok(egui::Key::A),
        winit::event::VirtualKeyCode::B => Ok(egui::Key::B),
        winit::event::VirtualKeyCode::C => Ok(egui::Key::C),
        winit::event::VirtualKeyCode::D => Ok(egui::Key::D),
        winit::event::VirtualKeyCode::E => Ok(egui::Key::E),
        winit::event::VirtualKeyCode::F => Ok(egui::Key::F),
        winit::event::VirtualKeyCode::G => Ok(egui::Key::G),
        winit::event::VirtualKeyCode::H => Ok(egui::Key::H),
        winit::event::VirtualKeyCode::I => Ok(egui::Key::I),
        winit::event::VirtualKeyCode::J => Ok(egui::Key::J),
        winit::event::VirtualKeyCode::K => Ok(egui::Key::K),
        winit::event::VirtualKeyCode::L => Ok(egui::Key::L),
        winit::event::VirtualKeyCode::M => Ok(egui::Key::M),
        winit::event::VirtualKeyCode::N => Ok(egui::Key::N),
        winit::event::VirtualKeyCode::O => Ok(egui::Key::O),
        winit::event::VirtualKeyCode::P => Ok(egui::Key::P),
        winit::event::VirtualKeyCode::Q => Ok(egui::Key::Q),
        winit::event::VirtualKeyCode::R => Ok(egui::Key::R),
        winit::event::VirtualKeyCode::S => Ok(egui::Key::S),
        winit::event::VirtualKeyCode::T => Ok(egui::Key::T),
        winit::event::VirtualKeyCode::U => Ok(egui::Key::U),
        winit::event::VirtualKeyCode::V => Ok(egui::Key::V),
        winit::event::VirtualKeyCode::W => Ok(egui::Key::W),
        winit::event::VirtualKeyCode::X => Ok(egui::Key::X),
        winit::event::VirtualKeyCode::Y => Ok(egui::Key::Y),
        winit::event::VirtualKeyCode::Z => Ok(egui::Key::Z),

        winit::event::VirtualKeyCode::F1 => Ok(egui::Key::F1),
        winit::event::VirtualKeyCode::F2 => Ok(egui::Key::F2),
        winit::event::VirtualKeyCode::F3 => Ok(egui::Key::F3),
        winit::event::VirtualKeyCode::F4 => Ok(egui::Key::F4),
        winit::event::VirtualKeyCode::F5 => Ok(egui::Key::F5),
        winit::event::VirtualKeyCode::F6 => Ok(egui::Key::F6),
        winit::event::VirtualKeyCode::F7 => Ok(egui::Key::F7),
        winit::event::VirtualKeyCode::F8 => Ok(egui::Key::F8),
        winit::event::VirtualKeyCode::F9 => Ok(egui::Key::F9),
        winit::event::VirtualKeyCode::F10 => Ok(egui::Key::F10),
        winit::event::VirtualKeyCode::F11 => Ok(egui::Key::F11),
        winit::event::VirtualKeyCode::F12 => Ok(egui::Key::F12),
        winit::event::VirtualKeyCode::F13 => Ok(egui::Key::F13),
        winit::event::VirtualKeyCode::F14 => Ok(egui::Key::F14),
        winit::event::VirtualKeyCode::F15 => Ok(egui::Key::F15),
        winit::event::VirtualKeyCode::F16 => Ok(egui::Key::F16),
        winit::event::VirtualKeyCode::F17 => Ok(egui::Key::F17),
        winit::event::VirtualKeyCode::F18 => Ok(egui::Key::F18),
        winit::event::VirtualKeyCode::F19 => Ok(egui::Key::F19),
        winit::event::VirtualKeyCode::F20 => Ok(egui::Key::F20),

        winit::event::VirtualKeyCode::Left => Ok(egui::Key::ArrowLeft),
        winit::event::VirtualKeyCode::Up => Ok(egui::Key::ArrowUp),
        winit::event::VirtualKeyCode::Right => Ok(egui::Key::ArrowRight),
        winit::event::VirtualKeyCode::Down => Ok(egui::Key::ArrowDown),

        winit::event::VirtualKeyCode::Escape => Ok(egui::Key::Escape),
        winit::event::VirtualKeyCode::Tab => Ok(egui::Key::Tab),
        winit::event::VirtualKeyCode::Back => Ok(egui::Key::Backspace),
        winit::event::VirtualKeyCode::Return => Ok(egui::Key::Enter),
        winit::event::VirtualKeyCode::Space => Ok(egui::Key::Space),

        winit::event::VirtualKeyCode::Insert => Ok(egui::Key::Insert),
        winit::event::VirtualKeyCode::Delete => Ok(egui::Key::Delete),
        winit::event::VirtualKeyCode::Home => Ok(egui::Key::Home),
        winit::event::VirtualKeyCode::End => Ok(egui::Key::End),
        winit::event::VirtualKeyCode::PageUp => Ok(egui::Key::PageUp),
        winit::event::VirtualKeyCode::PageDown => Ok(egui::Key::PageDown),

        winit::event::VirtualKeyCode::Colon => Ok(egui::Key::Colon),
        winit::event::VirtualKeyCode::Comma => Ok(egui::Key::Comma),
        winit::event::VirtualKeyCode::Period => Ok(egui::Key::Period),
        winit::event::VirtualKeyCode::Backslash => Ok(egui::Key::Backslash),
        winit::event::VirtualKeyCode::Slash => Ok(egui::Key::Slash),
        winit::event::VirtualKeyCode::Minus => Ok(egui::Key::Minus),
        winit::event::VirtualKeyCode::Equals => Ok(egui::Key::Equals),
        winit::event::VirtualKeyCode::Plus => Ok(egui::Key::Plus),
        winit::event::VirtualKeyCode::Semicolon => Ok(egui::Key::Semicolon),
        winit::event::VirtualKeyCode::LBracket => Ok(egui::Key::OpenBracket),
        winit::event::VirtualKeyCode::RBracket => Ok(egui::Key::CloseBracket),
        _ => Err("Unsupported key"),
    }
}
