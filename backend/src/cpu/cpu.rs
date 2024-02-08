use std::cell::RefMut;
use std::io::stdin;

use super::instructions::{Instruction, Opcode, Timing, NOP};
use super::interrupt::{self, InterruptController};
use super::registers::{Reg16, Reg8, Registers};
use crate::memory::mmu::Mmu;

pub struct Imem8;
pub struct Imem16;

#[derive(Copy, Clone)]
pub struct Mem<T: Src<u16>>(pub T);
#[derive(Copy, Clone)]
pub struct DMem<T: Src<u8>>(pub T);

pub trait Src<T> {
    fn read(self, cpu: &mut Cpu) -> T;
}

pub trait Dst<T> {
    fn write(self, cpu: &mut Cpu, val: T);
}

impl Dst<u8> for Reg8 {
    #[inline(always)]
    fn write(self, cpu: &mut Cpu, val: u8) {
        
        // if let Reg8::B = self {
        //     panic!("Writing {:02X} to register B", val);
        // }
        cpu.registers.write_u8(self, val)
    }
}

impl Dst<u16> for Reg16 {
    #[inline(always)]
    fn write(self, cpu: &mut Cpu, val: u16) {
        #[cfg(feature="debug")]
        if let Reg16::SP = self {
            println!("Writing 0x{:04X} to SP", val);
        }
        cpu.registers.write_u16(self, val)
    }
}

impl Src<u8> for Reg8 {
    #[inline(always)]
    fn read(self, cpu: &mut Cpu) -> u8 {
        cpu.registers.read_u8(self)
    }
}

impl Src<u16> for Reg16 {
    #[inline(always)]
    fn read(self, cpu: &mut Cpu) -> u16 {
        cpu.registers.read_u16(self)
    }
}

impl Src<u8> for Imem8 {
    #[inline(always)]
    fn read(self, cpu: &mut Cpu) -> u8 {
        let value = cpu.fetch_u8();
        #[cfg(feature="debug")]
        println!("Fetched value 0x{:02X} from immediate memory", value);
        value
    }
}

impl Src<u16> for Imem16 {
    #[inline(always)]
    fn read(self, cpu: &mut Cpu) -> u16 {
        let value = cpu.fetch_u16();
        #[cfg(feature="debug")]
        println!("Fetched value 0x{:04X} from immediate memory", value);
        value
    }
}

impl Src<u8> for Mem<Reg16> {
    #[inline(always)]
    fn read(self, cpu: &mut Cpu) -> u8 {
        let Mem(reg) = self;
        let addr = reg.read(cpu);
        cpu.mmu.read(addr)
    }
}

impl Src<u8> for Mem<Imem16> {
    #[inline(always)]
    fn read(self, cpu: &mut Cpu) -> u8 {
        let Mem(imm) = self;
        let addr = imm.read(cpu);
        // println!("Fetching value from address 0x{:04X}", addr);
        cpu.mmu.read(addr)
    }
}

impl Dst<u8> for Mem<Reg16> {
    #[inline(always)]
    fn write(self, cpu: &mut Cpu, val: u8) {
        let Mem(reg) = self;
        let addr = reg.read(cpu);
        // if let Reg16::HL = reg {
        //     eprintln!("Writing {:02X} to 0x{:04X}", val, addr);
        // }
        cpu.mmu.write(addr, val);
    }
}

impl Dst<u16> for Mem<Imem16> {
    #[inline(always)]
    fn write(self, cpu: &mut Cpu, val: u16) {
        let Mem(loc) = self;
        let addr = loc.read(cpu);
        let lsb = val as u8;
        let msb = (val >> 8) as u8;
        cpu.mmu.write(addr, lsb);
        cpu.mmu.write(addr + 1, msb);
    }
}

impl Dst<u8> for Mem<Imem16> {
    #[inline(always)]
    fn write(self, cpu: &mut Cpu, value: u8) {
        let Mem(loc) = self;

        let addr = loc.read(cpu);
        if addr == 0xDEF8 || addr == 0xDEF9{
            println!("Writing value 0x{:02X} to 0x{:04X}, {}", value, addr, cpu.registers);
            stdin().read_line(&mut String::with_capacity(1)).unwrap();
        }
        cpu.mmu.write(addr, value);
    }
}

impl Src<u8> for DMem<Reg8> {
    #[inline(always)]
    fn read(self, cpu: &mut Cpu) -> u8 {
        let DMem(reg) = self;
        let addr = reg.read(cpu) as u16;
        cpu.mmu.read(0xFF00 + addr)
    }
}

impl Src<u8> for DMem<Imem8> {
    #[inline(always)]
    fn read(self, cpu: &mut Cpu) -> u8 {
        let DMem(imm) = self;
        let addr = imm.read(cpu) as u16;
        cpu.mmu.read(0xFF00 + addr)
    }
}

impl Dst<u8> for DMem<Reg8> {
    #[inline(always)]
    fn write(self, cpu: &mut Cpu, value: u8) {
        let DMem(reg) = self;
        let addr = reg.read(cpu) as u16;
        cpu.mmu.write(0xFF00 + addr, value);
    }
}

impl Dst<u8> for DMem<Imem8> {
    #[inline(always)]
    fn write(self, cpu: &mut Cpu, value: u8) {
        let DMem(imm) = self;
        let addr = imm.read(cpu) as u16;
        cpu.mmu.write(0xFF00 + addr, value);
    }
}

#[allow(unused)]
pub struct Cpu {
    pub registers: Registers,
    ime: bool,
    pub halted: bool,
    mmu: Mmu,
}

impl Cpu {
    pub fn new(mmu: Mmu) -> Cpu {
        Cpu {
            registers: Registers::new(),
            ime: true,
            halted: false,
            mmu,
        }
    }

    pub fn execute_instruction(&mut self) -> u8 {
        if self.halted {
            return 4;
        }

        let opcode = self.fetch_u8();
        let op = match opcode {
            0xCB => Opcode::Prefixed(self.fetch_u8()),
            _ => Opcode::Unprefixed(opcode),
        };
        let instruction= match Instruction::get_instruction(op) {
            Some(instruction) => instruction,
            None => {
                eprintln!(
                    "Unknown opcode 0x{:04X} at address 0x{:04X}",
                    opcode,
                    self.registers.pc - 1
                );
                &NOP
            }
        };

        #[cfg(feature="debug")]
        {
            println!("Executing {} at address 0x{:04X}", instruction.mnemonic, self.registers.pc - 1);
        }
        let timing = (instruction.execute)(self);

        match timing {
            Timing::Normal => instruction.c_cycles,
            Timing::Conditionnal => match instruction.conditional_c_cycles {
                Some(cycles) => cycles,
                None => instruction.c_cycles,
            },
        }
    }

    pub fn handle_interrupts(&mut self, interrupt_controller: RefMut<'_, InterruptController>) -> u8 {
        // TODO: implement halt bug
        if self.halted {
            if let Some(_) = interrupt_controller.peek() {
                self.halted = false;
            }
            if !self.ime {
                
                return 0;
            }
        }
        let value = interrupt_controller.consume();
        let value = match value {
            Some(val) => val,
            None => return 0,
        };
        self.interrupt(value);
        self.halted = false;
        20
    }

    pub fn set_interrupts(&mut self, active: bool) {
        self.ime = active;
    }

    #[allow(unused)]
    fn interrupt(&mut self, value: u8) {
        self.set_interrupts(false);
        self.push16(self.registers.pc);
        self.registers.pc = value as u16;
    }

    #[inline(always)]
    pub fn fetch_u8(&mut self) -> u8 {
        let pc = self.registers.pc;
        self.registers.pc = pc.wrapping_add(1);
        self.mmu.read(pc)
    }

    #[inline(always)]
    pub fn fetch_u16(&mut self) -> u16 {
        let lsb = self.fetch_u8() as u16;
        let msb = self.fetch_u8() as u16;
        (msb << 8) | lsb
    }

    #[inline(always)]
    pub fn push(&mut self, value: u8) {
        let new_sp = self.registers.sp.wrapping_sub(1);
        self.registers.sp = new_sp;
        self.mmu.write(new_sp, value);
    }

    #[inline(always)]
    pub fn push16(&mut self, value: u16) {
        let lsb = (value & 0xFF) as u8;
        let msb = (value >> 8) as u8;
        // println!("pushing bytes {:02X} and {:02X} to stack pointer at {:04X}", lsb, msb, self.registers.sp);
        self.push(msb);
        self.push(lsb);
        // println!("{}", self.registers);
    }

    #[inline(always)]
    pub fn pop(&mut self) -> u8 {
        let sp = self.registers.sp;
        self.registers.sp = sp.wrapping_add(1);
        self.mmu.read(sp)
    }

    #[inline(always)]
    pub fn pop16(&mut self) -> u16 {
        // println!("SP before : 0x{:04X}", self.registers.sp);
        let lsb: u16 = self.pop() as u16;
        let msb: u16 = self.pop() as u16;
        // println!("popped bytes {:02X} and {:02X} from stack", lsb, msb);
        (msb << 8) | lsb
    }

    #[inline(always)]
    pub fn stop(&mut self) {
        eprintln!("CPU stop not implemented");
    }
}
