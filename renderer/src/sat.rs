use maths::linear::Vec2f;

// Tile has parallel sides so we only need 2 axes, and they are fixed
const TILE_AXES: [Vec2f; 2] = [Vec2f { x: 1.0, y: 0.0 }, Vec2f { x: 0.0, y: 1.0 }];

pub enum Overlap {
    None,
    Partial,
    Full,
}

fn project_polygon(axis: Vec2f, vertices: &[Vec2f]) -> (f32, f32) {
    let mut min = f32::MAX;
    let mut max = f32::MIN;

    for vertex in vertices.iter() {
        let projection = vertex.dot(axis);
        min = min.min(projection);
        max = max.max(projection);
    }

    (min, max)
}

fn overlap(main_min: f32, main_max: f32, other_min: f32, other_max: f32) -> bool {
    main_max >= other_min && other_max >= main_min
}

fn contains(main_min: f32, main_max: f32, other_min: f32, other_max: f32) -> bool {
    main_min <= other_min && main_max >= other_max
}

pub fn overlap_test(
    tile_points: &[Vec2f],
    triangle_points: &[Vec2f],
    triangle_axes: &[Vec2f],
) -> Overlap {
    let axes = triangle_axes.iter().chain(&TILE_AXES);

    let mut triangle_contains_tile = true;

    for axis in axes {
        let (tri_min, tri_max) = project_polygon(*axis, triangle_points);
        let (tile_min, tile_max) = project_polygon(*axis, tile_points);

        if !overlap(tri_min, tri_max, tile_min, tile_max) {
            return Overlap::None;
        }

        triangle_contains_tile &= contains(tri_min, tri_max, tile_min, tile_max);
    }

    if triangle_contains_tile {
        return Overlap::Full;
    } else {
        return Overlap::Partial;
    }
}
