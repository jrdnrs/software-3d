use glow::HasContext;

#[derive(Clone, Copy, Debug)]
pub enum Format {
    RGB,
    RGBA,
    RGBU8,
    RGBAU8,
}

impl Format {
    fn colour(&self) -> u32 {
        match self {
            Format::RGB => glow::RGB,
            Format::RGBA => glow::RGBA,
            Format::RGBU8 => glow::RGB8,
            Format::RGBAU8 => glow::RGBA8,
        }
    }

    fn data(&self) -> u32 {
        match self {
            Format::RGB => glow::FLOAT,
            Format::RGBA => glow::FLOAT,
            Format::RGBU8 => glow::UNSIGNED_BYTE,
            Format::RGBAU8 => glow::UNSIGNED_BYTE,
        }
    }

    fn bytes(&self) -> usize {
        match self {
            Format::RGB => 3 * 4,
            Format::RGBA => 4 * 4,
            Format::RGBU8 => 3,
            Format::RGBAU8 => 4,
        }
    }
}

/// For now, this is strictly for 2D RGBA8 textures, with nearest filtering, clamping to border,
/// and no mipmaps.
#[derive(Debug)]
pub struct Texture {
    pub handle: glow::Texture,
    pub width: usize,
    pub height: usize,

    pub data_bytes: usize,
    pub data_colour: u32,
    pub data_format: u32,
}

impl Texture {
    pub fn new(gl: &glow::Context, width: usize, height: usize, format: Format) -> Self {
        let handle = unsafe { gl.create_texture().expect("Failed to create texture") };

        let data_colour = format.colour();
        let data_format = format.data();

        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, Some(handle));

            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA8 as i32,
                width as i32,
                height as i32,
                0,
                data_colour,
                data_format,
                None,
            );

            gl.texture_parameter_i32(handle, glow::TEXTURE_WRAP_S, glow::CLAMP_TO_BORDER as i32);
            gl.texture_parameter_i32(handle, glow::TEXTURE_WRAP_T, glow::CLAMP_TO_BORDER as i32);
            gl.texture_parameter_i32(handle, glow::TEXTURE_MIN_FILTER, glow::NEAREST as i32);
            gl.texture_parameter_i32(handle, glow::TEXTURE_MAG_FILTER, glow::NEAREST as i32);

            gl.bind_texture(glow::TEXTURE_2D, None);
        }

        Self {
            handle,
            width,
            height,
            data_bytes: format.bytes(),
            data_colour,
            data_format,
        }
    }

    /// `data` must represent a 2D bitmap of the correct size and format, cast to bytes.
    pub fn set_data(&self, gl: &glow::Context, data: &[u8]) {
        debug_assert_eq!(data.len(), self.width * self.height * self.data_bytes);

        unsafe {
            gl.tex_sub_image_2d(
                glow::TEXTURE_2D,
                0,
                0,
                0,
                self.width as i32,
                self.height as i32,
                self.data_colour,
                self.data_format,
                glow::PixelUnpackData::Slice(data),
            );
        }
    }

    pub fn bind(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, Some(self.handle));
        }
    }

    pub fn unbind(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, None);
        }
    }

    pub fn delete(&self, gl: &glow::Context) {
        unsafe {
            gl.delete_texture(self.handle);
        }
    }
}
