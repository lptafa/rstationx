use sdl2;
use gl;
use std::ffi::c_void;

use crate::renderer::Renderer;

pub struct GLRenderer {
    sdl_context: sdl2::Sdl,
    window: sdl2::video::Window,
    gl_context: sdl2::video::GLContext,
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

        // Clear the window
        unsafe {
            gl::ClearColor(1., 1., 1., 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        window.gl_swap_window();

        GLRenderer { sdl_context, window, gl_context }
    }
}

impl Renderer for GLRenderer {
    // TODO
}