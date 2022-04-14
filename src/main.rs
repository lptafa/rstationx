// #![allow(dead_code)]
mod bios;
mod cpu;
mod gpu;
mod memory;
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
    let gpu = gpu::GPU::new();
    let ram = memory::RAM::new();
    let bus = memory::Bus::new(bios, ram, gpu);
    let mut cpu = cpu::CPU::new(bus);

    info!("Starting emulation loop...");
    loop {
        cpu.exec_next_instruction();
    }
}
