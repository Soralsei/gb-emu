use backend::memory::mmu::Mmu;
use backend::cpu::cpu::Cpu;

fn main() {
    let cpu = Cpu::new(Mmu::new());
    println!("Hello, world! {:#?}", cpu.registers);
}
