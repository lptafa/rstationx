pub mod gl_renderer;

pub trait Renderer {
    fn new() -> Self; // Setup and create a window
    fn start();
}

pub struct Position {
    pub x: i16,
    pub y: i16,
}

impl Position {
    pub fn from_gp0(val: u32) -> Position {
        let x = val as i16;
        let y = (val >> 16) as i16;

        Position{x, y}
    }
}

pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn from_gp0(val: u32) -> Color {
        let r = val as u8;
        let g = (val >> 8) as u8;
        let b = (val >> 16) as u8;

        Color{r, g, b}
    }
}