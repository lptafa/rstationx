// #![allow(dead_code)]
mod bios;
mod bus;
mod cpu;
mod dma;
mod gpu;
mod instruction;
mod map;
mod ram;
mod utils;

#[macro_use]
extern crate log;
extern crate env_logger;

fn main() {
    env_logger::Builder::from_default_env()
        .format_timestamp(None)
        .init();

    let bios_path = std::path::Path::new("./bios/bios");
    let bios = bios::BIOS::new(bios_path).unwrap();
    let ram = ram::RAM::new();
    let gpu = gpu::GPU::new();
    let bus = bus::Bus::new(bios, ram, gpu);
    let mut cpu = cpu::CPU::new(bus);

    info!("Starting emulation loop...");
    loop {
        cpu.exec_next_instruction();
    }
}
