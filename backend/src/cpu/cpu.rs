use std::cell::RefMut;
use std::rc::Rc;

use super::instructions::{Instruction, Opcode, Timing};
use super::interrupt::InterruptController;
use super::registers::{Reg16, Reg8, Registers};
use crate::clock::{Clock, M_CYCLE};
use crate::cpu::instructions::Cycles;
use crate::memory::mmu::Mmu;
use crate::util::bit_operations::*;

pub struct Imm8;
pub struct Imm16;

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
        cpu.registers.write_u8(self, val)
    }
}

impl Dst<u16> for Reg16 {
    #[inline(always)]
    fn write(self, cpu: &mut Cpu, val: u16) {
        #[cfg(feature = "debug")]
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

impl Src<u8> for Imm8 {
    #[inline(always)]
    fn read(self, cpu: &mut Cpu) -> u8 {
        let value = cpu.fetch_u8();
        #[cfg(feature = "debug")]
        println!("Fetched value 0x{:02X} from immediate memory", value);
        value
    }
}

impl Src<u16> for Imm16 {
    #[inline(always)]
    fn read(self, cpu: &mut Cpu) -> u16 {
        let value = cpu.fetch_u16();
        #[cfg(feature = "debug")]
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

impl Src<u8> for Mem<Imm16> {
    #[inline(always)]
    fn read(self, cpu: &mut Cpu) -> u8 {
        let Mem(imm) = self;
        let addr = imm.read(cpu);
        cpu.mmu.read(addr)
    }
}

impl Dst<u8> for Mem<Reg16> {
    #[inline(always)]
    fn write(self, cpu: &mut Cpu, val: u8) {
        let Mem(reg) = self;
        let addr = reg.read(cpu);
        cpu.mmu.write(addr, val);
    }
}

impl Dst<u16> for Mem<Imm16> {
    #[inline(always)]
    fn write(self, cpu: &mut Cpu, val: u16) {
        let Mem(loc) = self;
        let addr = loc.read(cpu);
        let (msb, lsb) = word_to_bytes(val);
        cpu.mmu.write(addr, lsb);
        cpu.mmu.write(addr + 1, msb);
    }
}

impl Dst<u8> for Mem<Imm16> {
    #[inline(always)]
    fn write(self, cpu: &mut Cpu, value: u8) {
        let Mem(loc) = self;

        let addr = loc.read(cpu);
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

impl Src<u8> for DMem<Imm8> {
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

impl Dst<u8> for DMem<Imm8> {
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
    clock: Rc<Clock>,
    stopped: bool,
}

impl Cpu {
    pub fn new(mmu: Mmu, clock: Rc<Clock>) -> Cpu {
        Cpu {
            registers: Registers::new(),
            ime: true,
            halted: false,
            mmu,
            clock,
            stopped: false,
        }
    }

    /// Tick the cycles this unit of work needs on top of the ones its bus
    /// accesses already ticked, then start counting the next one.
    fn spend(&mut self, total_cycles: usize) -> usize {
        let elapsed = self.clock.elapsed();
        self.clock
            .tick((total_cycles as u32).saturating_sub(elapsed) as u16);
        self.clock.reset();
        elapsed.max(total_cycles as u32) as usize
    }

    pub fn execute_instruction(&mut self) -> usize {
        if self.halted || self.stopped {
            self.spend(M_CYCLE as usize);
            return M_CYCLE as usize;
        }

        let opcode = self.fetch_u8();
        let op = match opcode {
            0xCB => Opcode::Prefixed(self.fetch_u8()),
            _ => Opcode::Unprefixed(opcode),
        };
        let instruction = Instruction::from_opcode(op);

        #[cfg(feature = "debug")]
        {
            println!(
                "Executing {} at address 0x{:04X}",
                instruction.mnemonic,
                self.registers.pc - 1
            );
        }
        let timing = (instruction.execute)(self);

        let cycles = match &instruction.cycles {
            Cycles::Unconditional(cycles) => *cycles,
            Cycles::Conditional(condition_cycles) => match timing {
                Timing::Normal => condition_cycles.not_taken,
                Timing::Conditional => condition_cycles.taken,
            },
        };
        self.spend(cycles)
    }

    pub fn handle_interrupts(
        &mut self,
        interrupt_controller: RefMut<'_, InterruptController>,
    ) -> usize {
        if self.stopped {
            return 0;
        }
        // TODO: implement halt bug
        if self.halted {
            if let Some(_) = interrupt_controller.peek() {
                self.halted = false;
            }
        }
        if !self.ime {
            return 0;
        }
        let value = interrupt_controller.consume();
        let value = match value {
            Some(val) => val,
            None => return 0,
        };
        self.interrupt(value);
        self.halted = false;

        // interrupt handling always consumes exactly 20 cycles.
        // Share the 20 cycles base with potential bus cycles
        self.spend(20)
    }

    pub fn set_interrupts(&mut self, active: bool) {
        self.ime = active;
    }

    fn interrupt(&mut self, value: u8) {
        self.set_interrupts(false);
        self.clock.tick(M_CYCLE);
        self.push_u16(self.registers.pc);
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
        let lsb = self.fetch_u8();
        let msb = self.fetch_u8();
        bytes_to_word(msb, lsb)
    }

    #[inline(always)]
    pub fn push_u8(&mut self, value: u8) {
        let new_sp = self.registers.sp.wrapping_sub(1);
        self.registers.sp = new_sp;
        self.mmu.write(new_sp, value);
    }

    #[inline(always)]
    pub fn push_u16(&mut self, value: u16) {
        self.clock.tick(M_CYCLE);
        let (msb, lsb) = word_to_bytes(value);
        self.push_u8(msb);
        self.push_u8(lsb);
    }

    #[inline(always)]
    pub fn pop_u8(&mut self) -> u8 {
        let sp = self.registers.sp;
        self.registers.sp = sp.wrapping_add(1);
        self.mmu.read(sp)
    }

    #[inline(always)]
    pub fn pop_u16(&mut self) -> u16 {
        let lsb = self.pop_u8();
        let msb = self.pop_u8();
        bytes_to_word(msb, lsb)
    }

    pub fn stop(&mut self) {
        let pending = self.mmu.peek(0xFFFF) & self.mmu.peek(0xFF0F) & 0x1F;
        let joyp = self.mmu.peek(0xFF00) & 0x0F;

        if joyp != 0x0F {
            if pending == 0 {
                // STOP => 2 bytes
                let _ = self.fetch_u8();
                self.halted = true;
            }
            return;
        }

        // if KEY1 & SPEED_SWITCH {
        //     if pending == 0 {
        //         let _ = self.fetch_u8();
        //         self.halted = true;
        //     }
        //     if !self.ime {
        //         // Maybe
        //         return Err(StopGlitchError);
        //     }
        //     return;
        // }

        // If no pending speed switch and no JOYP currently pressed
        if pending == 0 {
            // STOP => 2 bytes
            let _ = self.fetch_u8();
        }
        // For both, enter STOP mode and reset DIV
        self.stopped = true;
        self.clock.stop();
    }

    pub fn resume(&mut self) {
        self.stopped = false
    }
}
