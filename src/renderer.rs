use crate::gpu::{Color, Position};

pub trait Renderer {
    fn push_triangle(&mut self, positions: [Position; 3], colors: [Color; 3]);
    fn push_quad(&mut self, positions: [Position; 4], colors: [Color; 4]);
    fn draw(&mut self);
    fn display(&mut self);
}
