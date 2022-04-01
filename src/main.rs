// #![allow(dead_code)]
mod bios;
mod bus;
mod cpu;
mod instruction;
mod range;

fn main() {
    let bios_path = std::path::Path::new("./bios/bios");
    let bios = bios::BIOS::new(bios_path).unwrap();
    let bus = bus::Bus::new(bios);
    let mut cpu = cpu::CPU::new(bus);
    loop {
        cpu.exec_next_instruction();
    }
}
