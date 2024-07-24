use std::{fs::File, path::Path};

use crate::colour::RGB;

#[derive(Debug)]
pub struct Bitmap {
    width: usize,
    height: usize,
    pixels: Vec<RGB>,
}

impl Bitmap {
    pub fn new(width: usize, height: usize, pixels: Vec<RGB>) -> Self {
        Self {
            width,
            height,
            pixels,
        }
    }

    pub fn from_path_png(path: impl AsRef<Path>) -> Result<Self, anyhow::Error> {
        let mut decoder = png::Decoder::new(File::open(path)?);
        decoder.set_transformations(
            png::Transformations::EXPAND
                | png::Transformations::ALPHA
                | png::Transformations::STRIP_16,
        );

        let mut reader = decoder.read_info()?;
        let mut buffer = vec![0; reader.output_buffer_size()];

        let info = reader.next_frame(&mut buffer)?;

        assert_eq!(
            info.bit_depth,
            png::BitDepth::Eight,
            "Unsupported bit depth"
        );

        assert_eq!(
            info.color_type,
            png::ColorType::Rgba,
            "Unsupported colour type"
        );

        let pixels = buffer
            .chunks_exact(4)
            .map(|bytes| {
                let r = bytes[0];
                let g = bytes[1];
                let b = bytes[2];
                // let a = bytes[3];
                RGB::from_u8(r, g, b)
            })
            .collect();

        Ok(Self::new(info.width as usize, info.height as usize, pixels))
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn pixels(&self) -> &[RGB] {
        &self.pixels
    }
}
