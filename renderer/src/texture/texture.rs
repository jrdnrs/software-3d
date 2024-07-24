use std::path::Path;

use crate::{asset_manager::Named, colour::RGB, util::file_name, DIM_POW_2, MIP_LEVELS};

use super::{
    bitmap::Bitmap,
    mipmap::{calculate_mip_levels, generate_mip_maps, MipLevel},
};

#[derive(Debug, Default)]
pub struct Texture {
    name: String,
    pub levels: [MipLevel; MIP_LEVELS],
    pub pixels: Vec<RGB>,
}

impl Named for Texture {
    fn name(&self) -> &str {
        &self.name
    }
}

impl Texture {
    pub fn from_path_png(path: impl AsRef<Path>) -> Result<Self, anyhow::Error> {
        let bitmap = Bitmap::from_path_png(path.as_ref())?;
        Ok(Self::from_bitmap(bitmap, file_name(path.as_ref()).unwrap()))
    }

    fn from_bitmap(bitmap: Bitmap, name: String) -> Self {
        if DIM_POW_2 {
            assert!(
                bitmap.width().is_power_of_two() && bitmap.height().is_power_of_two(),
                "Texture dimensions must be power of 2"
            );
        }

        let levels = calculate_mip_levels(&bitmap);
        let buffer_size = levels[MIP_LEVELS - 1].offset
            + levels[MIP_LEVELS - 1].width * levels[MIP_LEVELS - 1].height;

        let mut pixels = vec![RGB::default(); buffer_size];

        // Copy the pixels from the bitmap into the first level of the texture
        pixels[..bitmap.pixels().len()].copy_from_slice(bitmap.pixels());

        // Generate rest of levels to fill buffer
        generate_mip_maps(&levels, &mut pixels);

        Self {
            levels,
            pixels,
            name,
        }
    }

    pub unsafe fn sample_unchecked(&self, mut x: f32, mut y: f32, level: usize) -> RGB {
        debug_assert!(level < MIP_LEVELS);
        let level = self.levels.get_unchecked(level);

        if !DIM_POW_2 {
            x = x - x.floor();
            y = y - y.floor();
        }

        debug_assert!(x.is_finite() && x.abs() < usize::MAX as f32);
        debug_assert!(y.is_finite() && y.abs() < usize::MAX as f32);
        let mut x = (x * level.width_f).to_int_unchecked::<usize>();
        let mut y = (y * level.height_f).to_int_unchecked::<usize>();

        if DIM_POW_2 {
            x &= level.width - 1;
            y &= level.height - 1;
        }

        let local_offset = y * level.width + x;
        let global_offset = level.offset + local_offset;

        debug_assert!(global_offset < self.pixels.len());
        *self.pixels.get_unchecked(global_offset)
    }
}
