use crate::csr::*;
use crate::instruction::*;
use crate::memory::Memory;
use crate::mmio::*;

pub const FLAG_ZERO: u32 = 1;
pub const FLAG_NEGATIVE: u32 = 2;
pub const FLAG_CARRY: u32 = 4;
pub const FLAG_OVERFLOW: u32 = 8;

pub const CAUSE_ILLEGAL: u32 = 2;
pub const CAUSE_ALIGN: u32 = 4;
pub const CAUSE_BREAK: u32 = 3;
pub const CAUSE_DIVZERO: u32 = 5;
pub const CAUSE_PRIVILEGE: u32 = 6;
pub const CAUSE_TIMER: u32 = 0x8000_0001;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fault {
    Halt,
    Trap(u32),
}

pub struct Cpu {
    registers: [u32; 16],
    pc: u32,
    flags: u32,
    memory: Memory,
    csr: Csr,
    mmio: Mmio,
    reservation: Option<u32>,
    fault: Option<Fault>,
    waiting: bool,
}

impl Cpu {
    pub fn new(memory_size: usize) -> Self {
        let mut registers = [0; 16];
        registers[15] = (memory_size as u32) & !3;

        Self {
            registers,
            pc: 0xffff_0000,
            flags: 0,
            memory: Memory::new(memory_size),
            csr: Csr::new(),
            mmio: Mmio::new(),
            reservation: None,
            fault: None,
            waiting: false,
        }
    }

    pub fn load_program(&mut self, program: &[u8]) {
        self.pc = 0;
        self.registers = [0; 16];
        self.registers[15] = (self.memory.len() as u32) & !3;
        self.flags = 0;
        self.csr = Csr::new();
        self.fault = None;
        self.waiting = false;
        self.reservation = None;
        let _ = self.memory.load(0, program);
    }

    pub fn load_data(&mut self, address: u32, data: &[u8]) {
        let _ = self.memory.load(address, data);
    }

    pub fn set_pc(&mut self, pc: u32) {
        self.pc = pc;
    }

    pub fn step(&mut self) {
        if self.fault.is_some() {
            return;
        }

        self.mmio.tick(1);
        self.csr.cycle = self.csr.cycle.wrapping_add(1);

        if self.waiting {
            if self.interrupt_pending() {
                self.waiting = false;
            } else {
                return;
            }
        }

        if self.interrupt_pending() {
            self.take_trap(CAUSE_TIMER);
            return;
        }

        let bytes = self.fetch_bytes();

        let instruction = match Instruction::decode(&bytes) {
            Ok(value) => value,
            Err(_) => {
                self.take_trap(CAUSE_ILLEGAL);
                return;
            }
        };

        let next = self.pc.wrapping_add(instruction.size());
        self.pc = next;

        self.execute(instruction);
        self.csr.instret = self.csr.instret.wrapping_add(1);
        self.registers[0] = 0;
    }

    pub fn run(&mut self, max_cycles: u64) {
        for _ in 0..max_cycles {
            if self.fault.is_some() {
                break;
            }
            self.step();
        }
    }

    fn fetch_bytes(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(7);

        for i in 0..7 {
            if let Ok(value) = self.memory.read_u8(self.pc.wrapping_add(i)) {
                result.push(value);
            } else {
                break;
            }
        }

        result
    }

    fn execute(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::Halt => {
                self.fault = Some(Fault::Halt);
            }
            Instruction::Nop => {}
            Instruction::Wfi => {
                self.waiting = true;
            }
            Instruction::Eret | Instruction::Iret => {
                self.pc = self.csr.epc;
            }
            Instruction::Break => self.take_trap(CAUSE_BREAK),

            Instruction::AddI(d, a, v) => self.binary_flags(d, self.reg(a).wrapping_add(v)),
            Instruction::SubI(d, a, v) => self.binary_flags(d, self.reg(a).wrapping_sub(v)),
            Instruction::AndI(d, a, v) => self.write_flags(d, self.reg(a) & v),
            Instruction::OrI(d, a, v) => self.write_flags(d, self.reg(a) | v),
            Instruction::XorI(d, a, v) => self.write_flags(d, self.reg(a) ^ v),
            Instruction::MulI(d, a, v) => self.write_flags(d, self.reg(a).wrapping_mul(v)),
            Instruction::SltI(d, a, v) => {
                self.write_flags(d, ((self.reg(a) as i32) < (v as i32)) as u32)
            }
            Instruction::SltIU(d, a, v) => self.write_flags(d, (self.reg(a) < v) as u32),

            Instruction::Add(d, a, b) => {
                let x = self.reg(a);
                let y = self.reg(b);
                let (r, c) = x.overflowing_add(y);
                self.write_register(d, r);
                self.flags = 0;
                if r == 0 {
                    self.flags |= FLAG_ZERO;
                }
                if (r & 0x8000_0000) != 0 {
                    self.flags |= FLAG_NEGATIVE;
                }
                if c {
                    self.flags |= FLAG_CARRY;
                }
                if ((x ^ r) & (y ^ r) & 0x8000_0000) != 0 {
                    self.flags |= FLAG_OVERFLOW;
                }
            }

            Instruction::Sub(d, a, b) => {
                let x = self.reg(a);
                let y = self.reg(b);
                let (r, c) = x.overflowing_sub(y);
                self.write_register(d, r);
                self.flags = 0;
                if r == 0 {
                    self.flags |= FLAG_ZERO;
                }
                if (r & 0x8000_0000) != 0 {
                    self.flags |= FLAG_NEGATIVE;
                }
                if c {
                    self.flags |= FLAG_CARRY;
                }
                if ((x ^ y) & (x ^ r) & 0x8000_0000) != 0 {
                    self.flags |= FLAG_OVERFLOW;
                }
            }

            Instruction::Mul(d, a, b) => self.write_flags(d, self.reg(a).wrapping_mul(self.reg(b))),

            Instruction::Div(d, a, b) => {
                let x = self.reg(a) as i32;
                let y = self.reg(b) as i32;
                if y == 0 {
                    self.take_trap(CAUSE_DIVZERO);
                } else {
                    self.write_flags(d, x.wrapping_div(y) as u32);
                }
            }

            Instruction::DivU(d, a, b) => {
                let y = self.reg(b);
                if y == 0 {
                    self.take_trap(CAUSE_DIVZERO);
                } else {
                    self.write_flags(d, self.reg(a) / y);
                }
            }

            Instruction::Rem(d, a, b) => {
                let y = self.reg(b) as i32;
                if y == 0 {
                    self.take_trap(CAUSE_DIVZERO);
                } else {
                    self.write_flags(d, (self.reg(a) as i32).wrapping_rem(y) as u32);
                }
            }

            Instruction::RemU(d, a, b) => {
                let y = self.reg(b);
                if y == 0 {
                    self.take_trap(CAUSE_DIVZERO);
                } else {
                    self.write_flags(d, self.reg(a) % y);
                }
            }

            Instruction::Cmp(a, b) => {
                let x = self.reg(a);
                let y = self.reg(b);
                let (r, c) = x.overflowing_sub(y);
                self.flags = 0;
                if r == 0 {
                    self.flags |= FLAG_ZERO;
                }
                if (r & 0x8000_0000) != 0 {
                    self.flags |= FLAG_NEGATIVE;
                }
                if c {
                    self.flags |= FLAG_CARRY;
                }
                if ((x ^ y) & (x ^ r) & 0x8000_0000) != 0 {
                    self.flags |= FLAG_OVERFLOW;
                }
            }

            Instruction::Min(d, a, b) => {
                self.write_flags(d, (self.reg(a) as i32).min(self.reg(b) as i32) as u32)
            }
            Instruction::Max(d, a, b) => {
                self.write_flags(d, (self.reg(a) as i32).max(self.reg(b) as i32) as u32)
            }

            Instruction::And(d, a, b) => self.write_flags(d, self.reg(a) & self.reg(b)),
            Instruction::Or(d, a, b) => self.write_flags(d, self.reg(a) | self.reg(b)),
            Instruction::Xor(d, a, b) => self.write_flags(d, self.reg(a) ^ self.reg(b)),
            Instruction::Not(d, a) => self.write_flags(d, !self.reg(a)),
            Instruction::Clz(d, a) => self.write_flags(d, self.reg(a).leading_zeros()),
            Instruction::Ctz(d, a) => self.write_flags(d, self.reg(a).trailing_zeros()),
            Instruction::Popcnt(d, a) => self.write_flags(d, self.reg(a).count_ones()),

            Instruction::Shl(d, a, b) => {
                self.write_flags(d, self.reg(a).wrapping_shl(self.reg(b) & 31))
            }
            Instruction::Shr(d, a, b) => {
                self.write_flags(d, self.reg(a).wrapping_shr(self.reg(b) & 31))
            }
            Instruction::Sar(d, a, b) => {
                self.write_flags(d, ((self.reg(a) as i32) >> (self.reg(b) & 31)) as u32)
            }
            Instruction::Rol(d, a, b) => {
                self.write_flags(d, self.reg(a).rotate_left(self.reg(b) & 31))
            }
            Instruction::Ror(d, a, b) => {
                self.write_flags(d, self.reg(a).rotate_right(self.reg(b) & 31))
            }

            Instruction::Lb(d, b, o) => self.load(d, self.reg(b).wrapping_add(o), 1, true),
            Instruction::Lbu(d, b, o) => self.load(d, self.reg(b).wrapping_add(o), 1, false),
            Instruction::Lh(d, b, o) => self.load(d, self.reg(b).wrapping_add(o), 2, true),
            Instruction::Lhu(d, b, o) => self.load(d, self.reg(b).wrapping_add(o), 2, false),
            Instruction::Lw(d, b, o) => self.load(d, self.reg(b).wrapping_add(o), 4, false),

            Instruction::Sb(s, b, o) => self.store(self.reg(b).wrapping_add(o), self.reg(s), 1),
            Instruction::Sh(s, b, o) => self.store(self.reg(b).wrapping_add(o), self.reg(s), 2),
            Instruction::Sw(s, b, o) => self.store(self.reg(b).wrapping_add(o), self.reg(s), 4),

            Instruction::Lrw(d, b) => {
                let address = self.reg(b);
                match self.read_word(address) {
                    Some(v) => {
                        self.reservation = Some(address);
                        self.write_flags(d, v);
                    }
                    None => self.take_trap(CAUSE_ALIGN),
                }
            }

            Instruction::Scw(d, s) => {
                let address = self.reg(s);
                let success = self.reservation == Some(address);
                self.reservation = None;
                self.write_flags(d, success as u32);
            }

            Instruction::Jmp(a) => {
                self.pc = a;
            }
            Instruction::Jz(a) => {
                if (self.flags & FLAG_ZERO) != 0 {
                    self.pc = a;
                }
            }
            Instruction::Jnz(a) => {
                if (self.flags & FLAG_ZERO) == 0 {
                    self.pc = a;
                }
            }
            Instruction::Jc(a) => {
                if (self.flags & FLAG_CARRY) != 0 {
                    self.pc = a;
                }
            }
            Instruction::Jn(a) => {
                if (self.flags & FLAG_NEGATIVE) != 0 {
                    self.pc = a;
                }
            }
            Instruction::Jo(a) => {
                if (self.flags & FLAG_OVERFLOW) != 0 {
                    self.pc = a;
                }
            }
            Instruction::Jge(a) => {
                if self.signed_ge() {
                    self.pc = a;
                }
            }
            Instruction::Jl(a) => {
                if !self.signed_ge() {
                    self.pc = a;
                }
            }
            Instruction::Jgt(a) => {
                if !self.zero() && self.signed_ge() {
                    self.pc = a;
                }
            }
            Instruction::Jle(a) => {
                if self.zero() || !self.signed_ge() {
                    self.pc = a;
                }
            }
            Instruction::Jcs(a) => {
                if self.reg(1) >= self.reg(2) {
                    self.pc = a;
                }
            }
            Instruction::Jcc(a) => {
                if self.reg(1) < self.reg(2) {
                    self.pc = a;
                }
            }

            Instruction::Push(r) => {
                let sp = self.reg(15).wrapping_sub(4);
                self.registers[15] = sp;
                self.store(sp, self.reg(r), 4);
            }

            Instruction::Pop(r) => {
                let sp = self.reg(15);
                self.load(r, sp, 4, false);
                self.registers[15] = sp.wrapping_add(4);
            }

            Instruction::Call(a) => {
                let ret = self.pc;
                let sp = self.reg(15).wrapping_sub(4);
                self.registers[15] = sp;
                self.store(sp, ret, 4);
                self.pc = a;
            }

            Instruction::Ret => {
                let sp = self.reg(15);
                if let Some(v) = self.read_word(sp) {
                    self.registers[15] = sp.wrapping_add(4);
                    self.pc = v;
                } else {
                    self.take_trap(CAUSE_ALIGN);
                }
            }

            Instruction::Csrr(d, c) => self.write_register(d, self.csr.read(c)),
            Instruction::Csrw(r, c) => self.csr_write(c, self.reg(r)),
            Instruction::Csrs(r, c) => self.csr_write(c, self.csr.read(c) | self.reg(r)),
            Instruction::Csrc(r, c) => self.csr_write(c, self.csr.read(c) & !self.reg(r)),

            Instruction::Mret | Instruction::Sret => {
                self.pc = self.csr.epc;
            }

            Instruction::SetMode(r) => {
                if self.csr.mode == MODE_MACHINE {
                    self.csr.mode = self.reg(r) & 2;
                } else {
                    self.take_trap(CAUSE_PRIVILEGE);
                }
            }

            Instruction::Int(c) => self.take_trap(c),
        }
    }

    fn reg(&self, r: u8) -> u32 {
        if r == 0 {
            0
        } else {
            self.registers[r as usize]
        }
    }

    fn write_register(&mut self, r: u8, v: u32) {
        if r != 0 {
            self.registers[r as usize] = v;
        }
        self.registers[0] = 0;
    }

    fn write_flags(&mut self, r: u8, v: u32) {
        self.write_register(r, v);
        self.flags &= !(FLAG_ZERO | FLAG_NEGATIVE);
        if v == 0 {
            self.flags |= FLAG_ZERO;
        }
        if (v & 0x8000_0000) != 0 {
            self.flags |= FLAG_NEGATIVE;
        }
    }

    fn binary_flags(&mut self, r: u8, v: u32) {
        self.write_flags(r, v);
    }

    fn zero(&self) -> bool {
        (self.flags & FLAG_ZERO) != 0
    }

    fn signed_ge(&self) -> bool {
        let n = (self.flags & FLAG_NEGATIVE) != 0;
        let v = (self.flags & FLAG_OVERFLOW) != 0;
        n == v
    }

    fn read_word(&mut self, address: u32) -> Option<u32> {
        if (address & 3) != 0 {
            return None;
        }

        if let Some(value) = self.mmio.read(address) {
            return Some(value);
        }

        self.memory.read_u32(address).ok()
    }

    fn load(&mut self, d: u8, address: u32, size: u8, signed: bool) {
        if address >= UART_BASE {
            if let Some(v) = self.mmio.read(address) {
                self.write_flags(d, v);
            } else {
                self.take_trap(CAUSE_ALIGN);
            }
            return;
        }

        let result = match size {
            1 => self.memory.read_u8(address).map(|v| {
                if signed {
                    v as i8 as i32 as u32
                } else {
                    v as u32
                }
            }),
            2 => self.memory.read_u16(address).map(|v| {
                if signed {
                    v as i16 as i32 as u32
                } else {
                    v as u32
                }
            }),
            _ => self.memory.read_u32(address),
        };

        match result {
            Ok(v) => self.write_flags(d, v),
            Err(_) => self.take_trap(CAUSE_ALIGN),
        }
    }

    fn store(&mut self, address: u32, value: u32, size: u8) {
        if address >= UART_BASE {
            if !self.mmio.write(address, value) {
                self.take_trap(CAUSE_ALIGN);
            }
            return;
        }

        let result = match size {
            1 => self.memory.write_u8(address, value as u8),
            2 => self.memory.write_u16(address, value as u16),
            _ => self.memory.write_u32(address, value),
        };

        if result.is_err() {
            self.take_trap(CAUSE_ALIGN);
        }
    }

    fn csr_write(&mut self, csr: u16, value: u32) {
        if self.csr.mode != MODE_MACHINE
            && matches!(
                csr,
                CSR_TVEC | CSR_CAUSE | CSR_EPC | CSR_IE | CSR_IP | CSR_MODE
            )
        {
            self.take_trap(CAUSE_PRIVILEGE);
            return;
        }
        self.csr.write(csr, value);
    }

    fn interrupt_pending(&self) -> bool {
        (self.csr.status & STATUS_INTERRUPT_ENABLE) != 0 && (self.csr.ip & self.csr.ie) != 0
    }

    fn take_trap(&mut self, cause: u32) {
        self.csr.epc = self.pc;
        self.csr.cause = cause;
        self.csr.status &= !STATUS_INTERRUPT_ENABLE;
        self.csr.mode = MODE_MACHINE;
        self.pc = self.csr.tvec;
    }

    pub fn registers(&self) -> &[u32; 16] {
        &self.registers
    }
    pub fn pc(&self) -> u32 {
        self.pc
    }
    pub fn flags(&self) -> u32 {
        self.flags
    }
    pub fn csr(&self) -> Csr {
        self.csr
    }
    pub fn fault(&self) -> Option<Fault> {
        self.fault
    }
}
