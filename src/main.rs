// #![allow(dead_code)]
mod bios;
mod cpu;
mod glrenderer;
mod gpu;
mod memory;
mod renderer;
mod utils;

#[macro_use]
extern crate log;
extern crate env_logger;
extern crate gl;
extern crate sdl2;

use std::os;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

fn main() {
    env_logger::Builder::from_default_env()
        .format_timestamp(None)
        .init();

    let sdl_context = sdl2::init().unwrap();
    async fn watch_events(mut event_pump: sdl2::EventPump) {
        loop {
            for e in event_pump.poll_iter() {
                match e {
                    Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => std::process::exit(0),
                    Event::Quit { .. } => std::process::exit(0),
                    _ => (),
                }
            }
        }
    }
    let _ = watch_events(sdl_context.event_pump().unwrap());

    let renderer = glrenderer::GLRenderer::new(sdl_context);

    let bios_path = std::path::Path::new("./bios/bios");
    let bios = bios::BIOS::new(bios_path).unwrap();

    let gpu = gpu::GPU::new(renderer);
    let ram = memory::RAM::new();
    let bus = memory::Bus::new(bios, ram, gpu);
    let mut cpu = cpu::CPU::new(bus);

    info!("Starting emulation loop...");
    loop {
        cpu.exec_next_instruction();
    }
}
