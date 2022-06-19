// #![allow(dead_code)]
mod bios;
mod cpu;
mod gpu;
mod memory;
mod utils;
mod renderer;
mod opengl_renderer;

#[macro_use]
extern crate log;
extern crate env_logger;
extern crate gl;
extern crate sdl2;

use sdl2::{event::Event};
use sdl2::keyboard::Keycode;

fn main() {
    env_logger::Builder::from_default_env()
        .format_timestamp(None)
        .init();

    let sdl_context = sdl2::init().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let renderer = opengl_renderer::GLRenderer::new(sdl_context);

    let bios_path = std::path::Path::new("./bios/bios");
    let bios = bios::BIOS::new(bios_path).unwrap();
    let gpu = gpu::GPU::new(renderer);
    let ram = memory::RAM::new();
    let bus = memory::Bus::new(bios, ram, gpu);
    let mut cpu = cpu::CPU::new(bus);

    info!("Starting emulation loop...");
    loop {
        for _ in 0..1_000_000 {
            cpu.exec_next_instruction();
        }

        for e in event_pump.poll_iter() {
            match e {
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => return,
                Event::Quit {..} => return,
                _ => (),
            }
        }
    }
}
