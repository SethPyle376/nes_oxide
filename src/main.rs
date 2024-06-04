mod cpu;

use std::io::Write;

pub use cpu::Bus;
pub use cpu::Cartridge;
pub use cpu::Cpu;

fn main() {
    let bus = Bus::new(Cartridge::load("./ROMS/nestest.nes").unwrap());
    let mut cpu = Cpu::new(bus);

    let mut trace_file = std::fs::OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open("trace.log")
        .unwrap();

    let mut inject = |cpu: &mut Cpu| {
        let trace = cpu.trace();
        trace_file.write_all(trace.as_bytes()).unwrap();
    };

    for i in 0..7000 {
        cpu.step(&mut inject);
    }
}
