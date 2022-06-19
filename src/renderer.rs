use crate::gpu::{Color, Position};

pub trait Renderer {
    fn push_triangle(&mut self, positions: [Position; 3], colors: [Color; 3]);
    fn draw(&mut self);
    fn display(&mut self);
}
