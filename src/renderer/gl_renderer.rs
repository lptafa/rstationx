use crate::renderer::Renderer;
use crate::renderer::{Color, Position};

use glium::glutin::dpi::LogicalSize;
use glium::glutin::event_loop::EventLoop;
use glium::index::PrimitiveType;
use glium::{glutin, program, uniform, Display, IndexBuffer, Program, Surface, VertexBuffer, implement_vertex};
use glutin::window::Window;

use super::Vertex;

pub struct GLRenderer {
    display: Display,
    program: Program,
}

impl Renderer for GLRenderer {
    fn new() -> Self {
        let event_loop = glutin::event_loop::EventLoop::new();
        let mut wb =
            glutin::window::WindowBuilder::new().with_inner_size(LogicalSize::new(1024, 512));
        wb.window.title = String::from("RStationX");
        let cb = glutin::ContextBuilder::new();
        let display = glium::Display::new(wb, cb, &event_loop).unwrap();
        let mut target = display.draw();
        target.clear_color(1.0, 1.0, 1.0, 1.0);
        target.finish().unwrap();

        let program = program!(&display,
            330 => {
                vertex: "
                  #version 330 core

                  in ivec2 vertex_position;
                  in uvec3 vertex_color;

                  out vec3 color;

                  void main() {
                    // Convert VRAM coordinates (0;1023, 0;511) into
                    // OpenGL coordinates (-1;1, -1;1)
                    float xpos = (float(vertex_position.x) / 512) - 1.0;
                    // VRAM puts 0 at the top, OpenGL at the bottom,
                    // we must mirror vertically
                    float ypos = 1.0 - (float(vertex_position.y) / 256);

                    gl_Position.xyzw = vec4(xpos, ypos, 0.0, 1.0);

                    // Convert the components from [0;255] to [0;1]
                    color = vec3(float(vertex_color.r) / 255,
                                 float(vertex_color.g) / 255,
                                 float(vertex_color.b) / 255);
                  }",
                fragment: "
                    #version 330 core

                    in vec3 color;
                    out vec4 frag_color;

                    void main() {
                        frag_color = vec4(color, 1.0);
                    }"
            },)
        .unwrap();
        GLRenderer { display, program }
    }

    fn push_triangle(&mut self, positions: [Position; 3], colors: [Color; 3]) {
        let draw = move || {
            // building the uniforms
            let uniforms = uniform! {
                matrix: [
                    [1.0, 0.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [0.0, 0.0, 0.0, 1.0f32]
                ]
            };
            let vertex_buffer = {
                #[derive(Copy, Clone)]
                struct Vertex {
                    vertex_position: [i16; 2],
                    vertex_color: [u8; 3],
                }

                implement_vertex!(Vertex, vertex_position, vertex_color);

                glium::VertexBuffer::new(
                    &self.display,
                    &[
                        Vertex {
                            vertex_position: [positions[0].x, positions[0].y],
                            vertex_color: [colors[0].r,colors[0].g,colors[0].b],
                        },
                        Vertex {
                            vertex_position: [positions[1].x, positions[1].y],
                            vertex_color: [colors[1].r,colors[1].g,colors[1].b],
                        },
                        Vertex {
                            vertex_position: [positions[2].x, positions[2].y],
                            vertex_color: [colors[2].r,colors[2].g,colors[2].b],
                        },
                    ],
                )
                .unwrap()
            };

            let index_buffer =
                glium::IndexBuffer::new(&self.display, PrimitiveType::TrianglesList, &[0u16, 1, 2])
                    .unwrap();

            // drawing a frame
            let mut target = self.display.draw();
            target
                .draw(
                    &vertex_buffer,
                    &index_buffer,
                    &self.program,
                    &uniforms,
                    &Default::default(),
                )
                .unwrap();
            target.finish().unwrap();
        };
        draw();
    }

    fn start() {}
}
