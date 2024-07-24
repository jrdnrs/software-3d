use std::path::{Path, PathBuf};

use maths::{geometry::AABB, linear::Vec3f};

use crate::{FAR, MAP_DEPTH_RANGE, MIP_FACTOR, MIP_LEVELS, NEAR};

/// Map a linear depth value, ranging from [NEAR] to [FAR], to a normalised depth value, ranging from 0.0 to 1.0.
pub fn normalise_depth(depth: f32) -> f32 {
    (depth - NEAR) * MAP_DEPTH_RANGE
}

/// Calculates an appropriate mip level based on the normalised depth and a bias.
pub fn mip_level(normal_depth: f32, bias: f32) -> usize {
    (((MIP_FACTOR + bias) * normal_depth) as usize).min(MIP_LEVELS - 1)
}

/// This is used during perspective projection to convert from camera space to screen space.
/// It is essentially a scaling factor that is used to get a pixel coordinate from a
/// coordinate in camera space, taking into account the field of view and screen size.
pub fn focal_dimensions(
    h_fov_rad: f32,
    v_fov_rad: f32,
    half_width: f32,
    half_height: f32,
) -> (f32, f32) {
    // Use similar triangles to calculate focal width/height, based on the screen we are
    // projecting onto and the field of view.
    let focal_width = half_width / (h_fov_rad * 0.5).tan();
    let focal_height = half_height / (v_fov_rad * 0.5).tan();

    (focal_width, focal_height)
}

/// Returns an AABB representing the view frustum, based on the given horizontal/vertical field of view.
pub fn view_frustum_bounds(h_fov_rad: f32, v_fov_rad: f32) -> AABB<Vec3f> {
    let h_tan = (h_fov_rad * 0.5).tan();
    let v_tan = (v_fov_rad * 0.5).tan();

    let h_opp_far = FAR * h_tan;
    let v_opp_far = FAR * v_tan;

    let near_bottom_left = Vec3f::new(-h_opp_far, -v_opp_far, NEAR);
    let far_top_right = Vec3f::new(h_opp_far, v_opp_far, FAR);

    AABB::new(near_bottom_left, far_top_right)
}

pub fn file_name(path: impl AsRef<Path>) -> Result<String, &'static str> {
    path.as_ref()
        .file_name()
        .ok_or_else(|| "Path has no file name")
        .and_then(|name| {
            name.to_str()
                .ok_or_else(|| "File name is not valid UTF-8")
                .map(|s| s.to_owned())
        })
}

pub fn normalise_path(path: impl AsRef<Path>) -> PathBuf {
    PathBuf::from(
        path.as_ref()
            .to_str()
            .expect("Path is not valid UTF-8")
            .replace("\\", "/"),
    )
}
