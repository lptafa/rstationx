use gl::types::{GLsizeiptr, GLuint};
use std::{ptr, slice};

/// Maximum number of vertex that can be stored in an attribute buffers
pub const VERTEX_BUFFER_LEN: u32 = 64 * 1024;

// Write only buffer with enough size for VERTEX_BUFFER_LEN elements
pub struct Buffer<T> {
    /// OpenGL buffer object
    object: GLuint,
    /// Mapped buffer memory
    map: *mut T,
}

impl<T: Copy + Default> Buffer<T> {
    pub fn new() -> Buffer<T> {
        let mut object = 0;
        let map;

        unsafe {
            // Generate the buffer object
            gl::GenBuffers(1, &mut object);

            // Bind it
            gl::BindBuffer(gl::ARRAY_BUFFER, object);

            // Compute the size of the buffer
            let element_size = std::mem::size_of::<T>() as GLsizeiptr;
            let buffer_size = element_size * VERTEX_BUFFER_LEN as GLsizeiptr;

            // Write only persistent mapping. Not coherent!
            let access = gl::MAP_WRITE_BIT | gl::MAP_PERSISTENT_BIT;

            // Allocate buffer memory
            gl::BufferStorage(gl::ARRAY_BUFFER, buffer_size, ptr::null(), access);

            // Remap the entire buffer
            map = gl::MapBufferRange(gl::ARRAY_BUFFER, 0, buffer_size, access) as *mut T;

            // Reset the buffer to 0 to avoid hard-to-reproduce bugs
            // if we do something wrong with unitialized memory
            let s = slice::from_raw_parts_mut(map, VERTEX_BUFFER_LEN as usize);

            for x in s.iter_mut() {
                *x = Default::default();
            }
        }

        Buffer { object, map }
    }

    /// Set entry at `index` to `val` in the buffer.
    pub fn set(&mut self, index: u32, val: T) {
        if index >= VERTEX_BUFFER_LEN {
            panic!("buffer overflow!");
        }

        unsafe {
            let p = self.map.offset(index as isize);

            *p = val;
        }
    }
}

impl<T> Drop for Buffer<T> {
    fn drop(&mut self) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.object);
            gl::UnmapBuffer(gl::ARRAY_BUFFER);
            gl::DeleteBuffers(1, &self.object);
        }
    }
}
