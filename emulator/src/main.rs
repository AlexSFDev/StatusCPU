mod cpu;
mod csr;
mod executable;
mod instruction;
mod memory;
mod mmio;

#[cfg(test)]
mod tests;

use cpu::Cpu;
use executable::Executable;
use std::env;

fn main() {
    let path = env::args()
        .nth(1)
        .unwrap_or_else(|| "program.status".to_string());

    let executable = Executable::load(&path).unwrap_or_else(|e| panic!("{}", e));

    let mut cpu = Cpu::new(1024 * 1024);

    cpu.load_program(&executable.code);
    cpu.load_data(executable.code.len() as u32, &executable.data);
    cpu.set_pc(executable.entry);

    cpu.run(10_000_000);

    println!();
    println!("StatusCPU v1");
    println!("PC     = 0x{:08X}", cpu.pc());
    println!("FLAGS  = 0x{:08X}", cpu.flags());

    for i in 0..16 {
        println!("R{:02}    = 0x{:08X}", i, cpu.registers()[i]);
    }

    let csr = cpu.csr();

    println!("CAUSE  = 0x{:08X}", csr.cause);
    println!("EPC    = 0x{:08X}", csr.epc);
    println!("MODE   = {}", csr.mode);
    println!("CYCLE  = {}", csr.cycle);
    println!("INSTRET = {}", csr.instret);
}
