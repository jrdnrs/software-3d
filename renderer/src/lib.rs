mod asset_manager;
mod camera;
mod colour;
mod framebuffer;
mod line;
mod model;
mod renderer;
mod sat;
mod shapes;
mod texture;
mod tile;
mod util;

pub use camera::Camera;
pub use renderer::Renderer;
pub use shapes::*;

pub const THREADS: usize = 0;
pub const RES_SCALE: f32 = 1.0 / 2.0;
pub const TILE_WIDTH: usize = 8;
pub const TILE_HEIGHT: usize = 8;

/*
  Camera
*/
pub const NEAR: f32 = 0.1;
pub const FAR: f32 = 256.0;
pub const MAP_DEPTH_RANGE: f32 = 1.0 / (FAR - NEAR);

/*
  Textures
*/
/// The number of mip levels to generate for each texture, where level 0 is the original size and
/// subsequent levels are half the size of the previous level
pub const MIP_LEVELS: usize = 3;
/// Arbitrary factor to scale the mip level distance thresholds by. A higher value will result in
/// more mip levels being used for a given distance
pub const MIP_FACTOR: f32 = 14.0;
/// Restricts texture dimensions to powers of two on both axes if true. This provides a performance 
/// boost when sampling textures as we can use a bitwise AND operation to wrap texture coordinates
pub const DIM_POW_2: bool = false;

/*
  Debug
*/
pub const DEBUG_TILES: bool = false;
