use crate::{colour::RGB, MIP_LEVELS};

use super::bitmap::Bitmap;

#[derive(Clone, Copy, Debug, Default)]
pub struct MipLevel {
    pub width: usize,
    pub height: usize,
    pub width_f: f32,
    pub height_f: f32,
    pub offset: usize,
}

pub fn calculate_mip_levels(bitmap: &Bitmap) -> [MipLevel; MIP_LEVELS] {
    let mut mip_width = bitmap.width();
    let mut mip_height = bitmap.height();
    let mut offset = 0;

    core::array::from_fn(|_| {
        let level = MipLevel {
            width: mip_width,
            height: mip_height,
            width_f: mip_width as f32,
            height_f: mip_height as f32,
            offset,
        };

        offset += mip_width * mip_height;
        mip_width /= 2;
        mip_height /= 2;

        level
    })
}

/// Generates mip maps for the given texture, assuming that the first level is already filled
pub fn generate_mip_maps(levels: &[MipLevel], buffer: &mut [RGB]) {
    for i in 1..MIP_LEVELS {
        let src_width = levels[i - 1].width;
        let src_height = levels[i - 1].height;
        let read_index = levels[i - 1].offset;
        let write_index = levels[i].offset;

        // `src` is slice of the pixels from the previous level, and `dst` is current level
        let (src, dst) = buffer.split_at_mut(write_index);
        let src = &src[read_index..];

        downscale_3x3_box_filter(src, src_width, src_height, dst);
    }
}

pub fn sample_clamp(src: &[RGB], src_width: usize, src_height: usize, x: isize, y: isize) -> RGB {
    let x = x.clamp(0, src_width as isize - 1) as usize;
    let y = y.clamp(0, src_height as isize - 1) as usize;

    src[y * src_width + x]
}

pub fn sample_wrap(src: &[RGB], src_width: usize, src_height: usize, x: isize, y: isize) -> RGB {
    let x = x.rem_euclid(src_width as isize) as usize;
    let y = y.rem_euclid(src_height as isize) as usize;

    src[y * src_width + x]
}

pub fn downscale_3x3_box_filter(src: &[RGB], src_width: usize, src_height: usize, dst: &mut [RGB]) {
    let dst_width = src_width / 2;
    let dst_height = src_height / 2;
    assert!(dst.len() >= dst_width * dst_height);

    for dst_y in 0..dst_height {
        for dst_x in 0..dst_width {
            let src_x = (dst_x * 2) as isize;
            let src_y = (dst_y * 2) as isize;

            // [a, b, c
            //  d, e, f
            //  g, h, i]

            let samples = [
                sample_wrap(src, src_width, src_height, src_x - 1, src_y - 1),
                sample_wrap(src, src_width, src_height, src_x, src_y - 1),
                sample_wrap(src, src_width, src_height, src_x + 1, src_y - 1),
                sample_wrap(src, src_width, src_height, src_x - 1, src_y),
                sample_wrap(src, src_width, src_height, src_x, src_y),
                sample_wrap(src, src_width, src_height, src_x + 1, src_y),
                sample_wrap(src, src_width, src_height, src_x - 1, src_y + 1),
                sample_wrap(src, src_width, src_height, src_x, src_y + 1),
                sample_wrap(src, src_width, src_height, src_x + 1, src_y + 1),
            ];

            let mut r = 0.0;
            let mut g = 0.0;
            let mut b = 0.0;

            for sample in samples {
                r += sample.r;
                g += sample.g;
                b += sample.b;
            }

            r /= 9.0;
            g /= 9.0;
            b /= 9.0;

            dst[dst_y * dst_width + dst_x] = RGB::new(r, g, b);
        }
    }
}
