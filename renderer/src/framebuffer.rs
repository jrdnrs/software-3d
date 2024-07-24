use crate::colour::RGB;

pub struct Framebuffer {
    width: usize,
    height: usize,
    half_width: f32,
    half_height: f32,
    aspect_ratio: f32,
    depth: Vec<f32>,
    pixels: Vec<RGB>,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        let len = width * height;
        let depth = vec![f32::MAX; len];
        let pixels = vec![RGB::default(); len];

        Self {
            width,
            height,
            half_width: width as f32 / 2.0,
            half_height: height as f32 / 2.0,
            aspect_ratio: width as f32 / height as f32,
            depth,
            pixels,
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn half_width(&self) -> f32 {
        self.half_width
    }

    pub fn half_height(&self) -> f32 {
        self.half_height
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.aspect_ratio
    }

    pub fn depth(&self) -> &[f32] {
        &self.depth
    }

    pub fn depth_mut(&mut self) -> &mut [f32] {
        &mut self.depth
    }

    pub fn pixels(&self) -> &[RGB] {
        &self.pixels
    }

    pub fn pixels_mut(&mut self) -> &mut [RGB] {
        &mut self.pixels
    }

    pub fn clear_depth_buffer(&mut self) {
        self.depth.fill(f32::MAX);
    }

    pub fn clear_colour_buffer(&mut self, colour: RGB) {
        self.pixels.fill(colour);
    }
}
