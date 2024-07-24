use maths::{
    geometry::{Segment, AABB},
    linear::Vec2f,
};

use crate::{colour::RGB, model::ProjectedTriangle, renderer::RendererState};

#[derive(Default)]
pub struct LineRenderer;

impl LineRenderer {
    pub fn update_viewport(&mut self, width: usize, height: usize) {}

    pub fn render(
        &mut self,
        state: &mut RendererState,
        triangles: &[ProjectedTriangle],
        colour: RGB,
    ) {
        for triangle in triangles.iter() {
            self.render_triangle(state, triangle, colour);
        }
    }

    pub unsafe fn render_line_unchecked(
        &mut self,
        state: &mut RendererState,
        a: Vec2f,
        b: Vec2f,
        colour: RGB,
    ) {
        let width = state.width();
        let height = state.height();

        let delta = b - a;
        let steps = delta.x.abs().max(delta.y.abs()) as usize;
        let step = delta / steps as f32;

        let mut current = a;
        for _ in 0..=steps {
            let x = current.x.round() as usize;
            let y = current.y.round() as usize;

            if x < width && y < height {
                *state.pixels_mut().get_unchecked_mut((y * width) + x) = colour;
            }

            current += step;
        }
    }

    pub fn render_line(&mut self, state: &mut RendererState, a: Vec2f, b: Vec2f, colour: RGB) {
        let screen_bounds = AABB::new(
            Vec2f::ZERO,
            Vec2f::new(state.width() as f32, state.height() as f32),
        );
        let line = Segment::new(a, b).clip_bounds(&screen_bounds);

        // SAFETY: points have been clipped to screen bounds
        unsafe {
            self.render_line_unchecked(state, line.a, line.b, colour);
        }
    }

    pub fn draw_h_line(
        &mut self,
        state: &mut RendererState,
        x: usize,
        y: usize,
        length: usize,
        colour: RGB,
    ) {
        debug_assert!(x < state.width() && y < state.height());
        debug_assert!(x + length <= state.width());

        let start = (y * state.width()) + x;
        let end = start + length;

        for i in start..end {
            state.pixels_mut()[i] = colour
        }
    }

    pub unsafe fn draw_h_line_unchecked(
        &mut self,
        state: &mut RendererState,
        x: usize,
        y: usize,
        length: usize,
        colour: RGB,
    ) {
        debug_assert!(x <= state.width() && y < state.height());
        debug_assert!(x + length <= state.width());

        let start = (y * state.width()) + x;
        let end = start + length;

        for i in start..end {
            *state.pixels_mut().get_unchecked_mut(i) = colour;
        }
    }

    pub fn draw_v_line(
        &mut self,
        state: &mut RendererState,
        x: usize,
        y: usize,
        length: usize,
        colour: RGB,
    ) {
        debug_assert!(x < state.width() && y <= state.height());
        debug_assert!(y + length <= state.height());

        let start = (y * state.width()) + x;
        let end = start + (length * state.width());

        for i in (start..end).step_by(state.width()) {
            state.pixels_mut()[i] = colour
        }
    }

    pub unsafe fn draw_v_line_unchecked(
        &mut self,
        state: &mut RendererState,
        x: usize,
        y: usize,
        length: usize,
        colour: RGB,
    ) {
        debug_assert!(x < state.width() && y < state.height());
        debug_assert!(y + length <= state.height());

        let start = (y * state.width()) + x;
        let end = start + (length * state.width());

        for i in (start..end).step_by(state.width()) {
            *state.pixels_mut().get_unchecked_mut(i) = colour;
        }
    }

    fn render_triangle(
        &mut self,
        state: &mut RendererState,
        triangle: &ProjectedTriangle,
        colour: RGB,
    ) {
        self.render_line(state, triangle.vertices.a, triangle.vertices.b, colour);
        self.render_line(state, triangle.vertices.b, triangle.vertices.c, colour);
        self.render_line(state, triangle.vertices.c, triangle.vertices.a, colour);
    }
}
