// #![allow(dead_code)]
mod bios;
mod cpu;
mod gpu;
mod memory;
mod utils;
mod renderer;

#[macro_use]
extern crate log;
extern crate env_logger;
extern crate glium;

use crate::renderer::gl_renderer::GLRenderer;
use glium::glutin;

fn main() {
    env_logger::Builder::from_default_env()
        .format_timestamp(None)
        .init();

    let bios_path = std::path::Path::new("./bios/bios");
    let bios = bios::BIOS::new(bios_path).unwrap();

    let event_loop = glutin::event_loop::EventLoop::new();
    let renderer = GLRenderer::new(&event_loop);
    let gpu = gpu::GPU::new(renderer);
    let ram = memory::RAM::new();
    let bus = memory::Bus::new(bios, ram, gpu);
    let mut cpu = cpu::CPU::new(bus);

    info!("Starting emulation loop...");
    event_loop.run(move |_event, _, _control_flow| {
        for _ in 0..1_000_000 {
            cpu.exec_next_instruction();
        }
    })
}
