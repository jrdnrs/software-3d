use glow::HasContext;

use crate::{
    opengl::{
        buffer::{BufferElement, BufferLayout, BufferUsage, VertexArray},
        program::{Program, ShaderDataType, ShaderSource},
        texture::{Format, Texture},
        GlWindow,
    },
    Renderer,
};

struct PixelBuffer {
    format: Format,
    write: Texture,
    read: Texture,
    width: usize,
    height: usize,
}

impl PixelBuffer {
    fn new(gl: &glow::Context, width: usize, height: usize, format: Format) -> Self {
        let write = Texture::new(gl, width, height, format);
        let read = Texture::new(gl, width, height, format);

        write.bind(gl);

        Self {
            format,
            write,
            read,
            width,
            height,
        }
    }

    fn set_pixels(&mut self, gl: &glow::Context, pixels: &[u8], width: usize, height: usize) {
        if width != self.width || height != self.height {
            self.resize(gl, width, height);
        }

        self.write.set_data(gl, pixels);
    }

    fn resize(&mut self, gl: &glow::Context, width: usize, height: usize) {
        self.width = width;
        self.height = height;
        self.write.delete(gl);
        self.read.delete(gl);
        self.write = Texture::new(gl, width, height, self.format);
        self.read = Texture::new(gl, width, height, self.format);
        self.write.bind(gl);
    }

    fn swap(&mut self, gl: &glow::Context) {
        core::mem::swap(&mut self.write, &mut self.read);
        self.write.bind(gl);
    }

    fn dispose(&self, gl: &glow::Context) {
        self.write.delete(gl);
        self.read.delete(gl);
    }
}

pub struct PixelRenderer {
    program: Program,
    vertex_array: VertexArray,
    pixel_buffer: PixelBuffer,
}

impl PixelRenderer {
    pub fn new(gl_window: &GlWindow) -> Self {
        let gl = gl_window.gl();

        let width = gl_window.winit_window.inner_size().width as usize;
        let height = gl_window.winit_window.inner_size().height as usize;

        Self::init_pipeline(gl);
        let pixel_buffer = PixelBuffer::new(gl, width, height, Format::RGB);
        let vertex_array = Self::load_vertex_array(gl);
        let program = Self::load_program(gl);

        // For now, we just leave these bound the entire time
        vertex_array.bind(gl);
        program.bind(gl);

        Self {
            program,
            vertex_array,
            pixel_buffer,
        }
    }

    pub fn clear(&self, gl_window: &GlWindow) {
        let gl = gl_window.gl();
        unsafe {
            gl.clear(glow::COLOR_BUFFER_BIT);
        }
    }

    pub fn swap_buffers(&mut self, gl_window: &GlWindow) {
        self.pixel_buffer.swap(gl_window.gl());
    }

    pub fn set_viewport(&self, gl_window: &GlWindow, width: usize, height: usize) {
        let gl = gl_window.gl();
        unsafe {
            gl.viewport(0, 0, width as i32, height as i32);
        }
    }

    pub fn draw(&mut self, gl_window: &GlWindow) {
        let gl = gl_window.gl();
        unsafe {
            gl.draw_arrays(glow::TRIANGLES, 0, 6);
        }
    }

    pub fn set_pixels(&mut self, gl_window: &GlWindow, pixels: &[u8], width: usize, height: usize) {
        self.pixel_buffer
            .set_pixels(gl_window.gl(), pixels, width, height);
    }

    pub fn dispose(&self, gl: &glow::Context) {
        self.pixel_buffer.dispose(gl);
        self.vertex_array.dispose_buffers(gl);
        self.vertex_array.dispose(gl);
        self.program.dispose(gl);
    }

    fn init_pipeline(gl: &glow::Context) {
        unsafe {
            println!("GL Version: {}", gl.get_parameter_string(glow::VERSION));

            // Requires mutable reference to the context
            // gl.debug_message_callback(|src, typ, id, sev, msg: &str| {
            //     let source = error_source(src);
            //     let severity = error_severity(sev);
            //     let type_ = error_type(typ);
            //     println!(
            //         "{} :: {} :: {} :: {} :: {}",
            //         id, severity, type_, source, msg
            //     );
            // });

            gl.clear_color(0.0, 0.0, 0.0, 1.0);

            gl.disable(glow::BLEND);
            gl.disable(glow::COLOR_LOGIC_OP);
            gl.disable(glow::CULL_FACE);
            gl.disable(glow::DEPTH_CLAMP);
            gl.disable(glow::DEPTH_TEST);
            gl.disable(glow::DITHER);
            gl.disable(glow::FRAMEBUFFER_SRGB);
            gl.disable(glow::LINE_SMOOTH);
            gl.disable(glow::MULTISAMPLE);
            gl.disable(glow::POLYGON_SMOOTH);
            gl.disable(glow::POLYGON_OFFSET_FILL);
            gl.disable(glow::POLYGON_OFFSET_LINE);
            gl.disable(glow::POLYGON_OFFSET_POINT);
            gl.disable(glow::PROGRAM_POINT_SIZE);
            gl.disable(glow::PRIMITIVE_RESTART);
            gl.disable(glow::SAMPLE_ALPHA_TO_COVERAGE);
            gl.disable(glow::SAMPLE_ALPHA_TO_ONE);
            gl.disable(glow::SAMPLE_COVERAGE);
            gl.disable(glow::SAMPLE_MASK);
            gl.disable(glow::SCISSOR_TEST);
            gl.disable(glow::STENCIL_TEST);
            gl.disable(glow::TEXTURE_CUBE_MAP_SEAMLESS);

            gl.depth_mask(false);
            gl.front_face(glow::CCW);
            gl.polygon_mode(glow::FRONT, glow::FILL);
            gl.active_texture(glow::TEXTURE0);
        }
    }

    fn load_vertex_array(gl: &glow::Context) -> VertexArray {
        let mut vertex_array = VertexArray::new(
            gl,
            vec![BufferLayout::new(
                vec![
                    BufferElement::new(ShaderDataType::Float2, "coords"),
                    BufferElement::new(ShaderDataType::Float2, "tex_coords"),
                ],
                6,
                0,
                BufferUsage::StaticDraw,
            )],
        );

        vertex_array.vertex_buffers[0].bind(gl);
        vertex_array.vertex_buffers[0].set_data(
            gl,
            0,
            &[
                // Positions     // Texture Coords
                -1.0f32, 1.0, 0.0, 0.0, // Top Left
                1.0, -1.0, 1.0, 1.0, // Bottom Right
                -1.0, -1.0, 0.0, 1.0, // Bottom Left
                -1.0, 1.0, 0.0, 0.0, // Top Left
                1.0, 1.0, 1.0, 0.0, // Top Right
                1.0, -1.0, 1.0, 1.0, // Bottom Right
            ],
        );

        vertex_array
    }

    fn load_program(gl: &glow::Context) -> Program {
        let vertex_shader = include_str!("pixel.vert");
        let fragment_shader = include_str!("pixel.frag");

        let shader_source = ShaderSource::new(vertex_shader, fragment_shader);
        Program::new(gl, shader_source)
    }
}

pub struct PixelRendererContext<'a> {
    gl_window: &'a GlWindow,
    pixel_renderer: &'a mut PixelRenderer,
}

impl<'a> PixelRendererContext<'a> {
    pub fn new(gl_window: &'a GlWindow, pixel_renderer: &'a mut PixelRenderer) -> Self {
        Self {
            gl_window,
            pixel_renderer,
        }
    }

    /// `pixels` must be a slice of length `width * height * 4`, representing the
    /// RGBA pixels of the window
    pub fn set_pixels(&mut self, pixels: &[u8], width: usize, height: usize) {
        self.pixel_renderer
            .set_pixels(self.gl_window, pixels, width, height);
    }
}

impl Renderer for PixelRenderer {
    type Context<'a> = PixelRendererContext<'a>;

    fn new(gl_window: &GlWindow) -> PixelRenderer {
        PixelRenderer::new(gl_window)
    }

    fn clear(&mut self, gl_window: &GlWindow) {
        PixelRenderer::clear(self, gl_window);
    }

    fn draw(&mut self, gl_window: &GlWindow) {
        PixelRenderer::draw(self, gl_window);
    }

    fn swap_buffers(&mut self, gl_window: &GlWindow) {
        PixelRenderer::swap_buffers(self, gl_window);
    }

    fn set_viewport(&mut self, gl_window: &GlWindow, width: usize, height: usize) {
        PixelRenderer::set_viewport(self, gl_window, width, height);
    }

    fn dispose(&mut self, gl_window: &GlWindow) {
        PixelRenderer::dispose(self, gl_window.gl());
    }

    fn on_event(&mut self, event: &winit::event::Event<()>) {
        // nothing to do
    }

    fn context<'a>(&'a mut self, gl_window: &'a GlWindow) -> Self::Context<'a> {
        PixelRendererContext::new(gl_window, self)
    }
}

fn error_source(source: u32) -> &'static str {
    match source {
        glow::DEBUG_SOURCE_API => "Calls to the OpenGL API",
        glow::DEBUG_SOURCE_WINDOW_SYSTEM => "Calls to a window-system API",
        glow::DEBUG_SOURCE_SHADER_COMPILER => "A compiler for a shading language",
        glow::DEBUG_SOURCE_THIRD_PARTY => "An application associated with OpenGL",
        glow::DEBUG_SOURCE_APPLICATION => "Generated by the user of this application",
        _ => "",
    }
}

fn error_severity(severity: u32) -> &'static str {
    match severity {
        glow::DEBUG_SEVERITY_HIGH => "High",
        glow::DEBUG_SEVERITY_MEDIUM => "Medium",
        glow::DEBUG_SEVERITY_LOW => "Low",
        glow::DEBUG_SEVERITY_NOTIFICATION => "Notification",
        _ => "",
    }
}

fn error_type(type_: u32) -> &'static str {
    match type_ {
        glow::DEBUG_TYPE_ERROR => "An error, typically from the API",
        glow::DEBUG_TYPE_DEPRECATED_BEHAVIOR => "Some behavior marked deprecated has been used",
        glow::DEBUG_TYPE_UNDEFINED_BEHAVIOR => "Something has invoked undefined behavior",
        glow::DEBUG_TYPE_PORTABILITY => "Some functionality the user relies upon is not portable",
        glow::DEBUG_TYPE_PERFORMANCE => "	Code has triggered possible performance issues",
        glow::DEBUG_TYPE_MARKER => "Command stream annotation",
        glow::DEBUG_TYPE_PUSH_GROUP => "Group pushing",
        glow::DEBUG_TYPE_POP_GROUP => "Group popping",
        _ => "",
    }
}
