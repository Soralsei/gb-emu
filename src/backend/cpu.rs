use super::mmu::Mmu;
use super::registers::{Registers, Reg8, Reg16};


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
        cpu.registers.write_u8(self, val)
    }
}

impl Dst<u16> for Reg16 {
    #[inline(always)]
    fn write(self, cpu: &mut Cpu, val: u16) {
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
        cpu.fetch_u8()
    }
}

impl Src<u16> for Imem16 {
    #[inline(always)]
    fn read(self, cpu: &mut Cpu) -> u16 {
        cpu.fetch_u16()
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
    fn write(self, cpu: &mut Cpu, val: u8) {
        let Mem(loc) = self;
        let addr = loc.read(cpu);
        let value = val as u8;
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

    pub fn set_interrupts(&mut self, active: bool) {
        self.ime = active;
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
        self.mmu.write(new_sp, value);
    }

    #[inline(always)]
    pub fn push16(&mut self, value: u16) {
        let lsb = (value & 0xFF) as u8;
        let msb = value as u8;
        self.push(lsb);
        self.push(msb);
    }

    #[inline(always)]
    pub fn pop(&mut self) -> u8 {
        let sp = self.registers.sp;
        self.registers.sp = sp.wrapping_add(1);
        self.mmu.read(sp)
    }

    #[inline(always)]
    pub fn pop16(&mut self) -> u16 {
        let msb: u16 = self.pop() as u16;
        let lsb: u16 = self.pop() as u16;
        (msb << 8) | lsb
    }

    #[inline(always)]
    pub fn stop(&mut self) {
        unimplemented!("CPU stop not implemented");
    }
}
