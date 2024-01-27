mod backend;
use crate::backend::cpu::Cpu;
use crate::backend::mmu::Mmu;

fn main() {
    println!("Hello, world!");
    let mmu: Mmu = Mmu::new();
    println!("test : {}", mmu.read(0x100));
    let _cpu = Cpu::new(mmu);
}
