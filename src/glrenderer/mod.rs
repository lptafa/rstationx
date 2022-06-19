mod buffer;

use gl;
use gl::types::{GLint, GLsizei, GLuint};
use sdl2;
use std::ffi::{c_void, CString};
use std::io::{self, Write};
use std::ptr;

use crate::gpu::{Color, Position};
use crate::renderer::Renderer;
use buffer::{Buffer, VERTEX_BUFFER_LEN};

pub struct GLRenderer {
    sdl_context: sdl2::Sdl,
    window: sdl2::video::Window,
    gl_context: sdl2::video::GLContext,

    vertex_shader: GLuint,
    fragment_shader: GLuint,
    program: GLuint,
    vertex_array_object: GLuint,
    positions: Buffer<Position>,
    colors: Buffer<Color>,
    nvertices: u32,
}

impl GLRenderer {
    pub fn new(sdl_context: sdl2::Sdl) -> GLRenderer {
        let video_subsystem = sdl_context.video().unwrap();

        let gl_attr = video_subsystem.gl_attr();

        gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
        gl_attr.set_context_version(3, 3);

        let window = video_subsystem
            .window("RStationX", 1024, 512)
            .opengl()
            .position_centered()
            .build()
            .unwrap();

        let gl_context = window.gl_create_context().unwrap();
        gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const c_void);

        println!("1");

        let vertex_shader = compile_shader(
            gl::VERTEX_SHADER,
            "
            #version 330 core

            in ivec2 vertex_position;
            in uvec3 vertex_color;

            out vec3 color;

            void main() {
            float xpos = (float(vertex_position.x) / 512) - 1.0;
            float ypos = 1.0 - (float(vertex_position.y) / 256);

            gl_Position.xyzw = vec4(xpos, ypos, 0.0, 1.0);
            color = vec3(float(vertex_color.r) / 255,
                         float(vertex_color.g) / 255,
                         float(vertex_color.b) / 255);
            }
        ",
        );

        let fragment_shader = compile_shader(
            gl::FRAGMENT_SHADER,
            "
            #version 330 core

            in vec3 color;
            out vec4 frag_color;

            void main() {
                frag_color = vec4(color, 1.0);
            }
        ",
        );

        let program = link_program(&[vertex_shader, fragment_shader]);

        // Clear the window
        unsafe {
            gl::UseProgram(program);
        }

        // Generate our vertex attribute object that will hold our vertex attributes
        let mut vertex_array_object = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut vertex_array_object);
            gl::BindVertexArray(vertex_array_object);
        }

        // Setup the "position" attribute. First we create the buffer holding the
        // positions (this call also binds it)
        let positions = Buffer::new();

        unsafe {
            let index = find_program_attrib(program, "vertex_position");
            gl::EnableVertexAttribArray(index);
            gl::VertexAttribIPointer(index, 2, gl::SHORT, 0, ptr::null());
        }

        // Setup the "color" attribute and bind it
        let colors = Buffer::new();

        unsafe {
            let index = find_program_attrib(program, "vertex_color");
            gl::EnableVertexAttribArray(index);
            gl::VertexAttribIPointer(index, 3, gl::UNSIGNED_BYTE, 0, ptr::null());
        }

        GLRenderer {
            sdl_context,
            window,
            gl_context,

            vertex_shader,
            fragment_shader,
            program,
            vertex_array_object,
            positions,
            colors,
            nvertices: 0,
        }
    }
}

impl Drop for GLRenderer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vertex_array_object);
            gl::DeleteShader(self.vertex_shader);
            gl::DeleteShader(self.fragment_shader);
            gl::DeleteProgram(self.program);
        }
    }
}

impl Renderer for GLRenderer {
    fn push_triangle(&mut self, positions: [Position; 3], colors: [Color; 3]) {
        // Make sure we have enough room left to queue the vertex
        if self.nvertices + 3 > VERTEX_BUFFER_LEN {
            println!("Vertex attribute buffers full, forcing draw");
            self.draw();
        }

        for i in 0..3 {
            // Push
            self.positions.set(self.nvertices, positions[i]);
            self.colors.set(self.nvertices, colors[i]);
            self.nvertices += 1;
        }
    }

    fn draw(&mut self) {
        // Make sure all the data from the persistent mappings is flushed to the buffer
        unsafe {
            gl::MemoryBarrier(gl::CLIENT_MAPPED_BUFFER_BARRIER_BIT);
            gl::DrawArrays(gl::TRIANGLES, 0, self.nvertices as GLsizei);

            let sync = gl::FenceSync(gl::SYNC_GPU_COMMANDS_COMPLETE, 0);
            loop {
                let r = gl::ClientWaitSync(sync, gl::SYNC_FLUSH_COMMANDS_BIT, 10000000);
                if r == gl::ALREADY_SIGNALED || r == gl::CONDITION_SATISFIED {
                    // Drawing done
                    break;
                }
            }
        }

        // Reset the buffers
        self.nvertices = 0;
    }

    /// Draw the buffered commands and display them
    fn display(&mut self) {
        self.draw();
        self.window.gl_swap_window();
    }
}

fn compile_shader(kind: gl::types::GLenum, source: &str) -> GLuint {
    unsafe {
        let id = gl::CreateShader(kind);
        let c_str = CString::new(source.as_bytes()).unwrap();
        gl::ShaderSource(id, 1, &c_str.as_ptr(), std::ptr::null());
        gl::CompileShader(id);

        let mut status = gl::FALSE as GLint;
        gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut status);

        if status != (gl::TRUE as GLint) {
            panic!("Compiling shaders failed.");
        }
        id
    }
}

fn link_program(shaders: &[GLuint]) -> GLuint {
    unsafe {
        let program = gl::CreateProgram();
        for shader in shaders {
            gl::AttachShader(program, *shader);
        }
        gl::LinkProgram(program);

        let mut status = gl::FALSE as GLint;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

        if status != (gl::TRUE as GLint) {
            panic!("Program linking failed.");
        }
        program
    }
}

pub fn find_program_attrib(program: GLuint, attr: &str) -> GLuint {
    let cstr = CString::new(attr).unwrap();
    let index = unsafe { gl::GetAttribLocation(program, cstr.as_ptr()) };
    if index < 0 {
        panic!("Attribute \"{}\" not found in program ({})", attr, index);
    }
    index as GLuint
}
