use crate::renderer::Renderer;

use glium::glutin::dpi::LogicalSize;
use glium::glutin::event_loop::EventLoop;
use glium::{glutin, Surface};
use glium::index::PrimitiveType;

pub struct GLRenderer {
    display: glium::Display,
}

impl Renderer for GLRenderer {
    fn new() -> Self {
        let event_loop = glutin::event_loop::EventLoop::new();
        let mut wb = glutin::window::WindowBuilder::new().with_inner_size(LogicalSize::new(1024, 512));
        wb.window.title = String::from("RStationX");
        let cb = glutin::ContextBuilder::new();
        let display = glium::Display::new(wb, cb, &event_loop).unwrap();

        GLRenderer { display }
    }

    fn start() {}
}
