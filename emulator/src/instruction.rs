#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeError {
    InvalidOpcode(u8),
    Truncated,
    InvalidRegister(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Instruction {
    Halt,
    Nop,
    Wfi,
    Eret,
    Break,

    AddI(u8, u8, u32),
    SubI(u8, u8, u32),
    AndI(u8, u8, u32),
    OrI(u8, u8, u32),
    XorI(u8, u8, u32),
    MulI(u8, u8, u32),
    SltI(u8, u8, u32),
    SltIU(u8, u8, u32),

    Add(u8, u8, u8),
    Sub(u8, u8, u8),
    Mul(u8, u8, u8),
    Div(u8, u8, u8),
    DivU(u8, u8, u8),
    Rem(u8, u8, u8),
    RemU(u8, u8, u8),
    Cmp(u8, u8),
    Min(u8, u8, u8),
    Max(u8, u8, u8),

    And(u8, u8, u8),
    Or(u8, u8, u8),
    Xor(u8, u8, u8),
    Not(u8, u8),
    Clz(u8, u8),
    Ctz(u8, u8),
    Popcnt(u8, u8),

    Shl(u8, u8, u8),
    Shr(u8, u8, u8),
    Sar(u8, u8, u8),
    Rol(u8, u8, u8),
    Ror(u8, u8, u8),

    Lb(u8, u8, u32),
    Lbu(u8, u8, u32),
    Lh(u8, u8, u32),
    Lhu(u8, u8, u32),
    Lw(u8, u8, u32),
    Sb(u8, u8, u32),
    Sh(u8, u8, u32),
    Sw(u8, u8, u32),
    Lrw(u8, u8),
    Scw(u8, u8),

    Jmp(u32),
    Jz(u32),
    Jnz(u32),
    Jc(u32),
    Jn(u32),
    Jo(u32),
    Jge(u32),
    Jl(u32),
    Jgt(u32),
    Jle(u32),
    Jcs(u32),
    Jcc(u32),

    Push(u8),
    Pop(u8),
    Call(u32),
    Ret,

    Csrr(u8, u16),
    Csrw(u8, u16),
    Csrs(u8, u16),
    Csrc(u8, u16),

    Mret,
    Sret,
    SetMode(u8),

    Int(u32),
    Iret,
}

impl Instruction {
    pub fn decode(bytes: &[u8]) -> Result<Self, DecodeError> {
        let op = *bytes.first().ok_or(DecodeError::Truncated)?;

        let r = |index: usize| -> Result<u8, DecodeError> {
            let value = *bytes.get(index).ok_or(DecodeError::Truncated)?;
            if value > 15 {
                Err(DecodeError::InvalidRegister(value))
            } else {
                Ok(value)
            }
        };

        let u32_at = |index: usize| -> Result<u32, DecodeError> {
            if bytes.len() < index + 4 {
                return Err(DecodeError::Truncated);
            }
            Ok(u32::from_le_bytes([
                bytes[index],
                bytes[index + 1],
                bytes[index + 2],
                bytes[index + 3],
            ]))
        };

        let u16_at = |index: usize| -> Result<u16, DecodeError> {
            if bytes.len() < index + 2 {
                return Err(DecodeError::Truncated);
            }
            Ok(u16::from_le_bytes([bytes[index], bytes[index + 1]]))
        };

        let i3 = |f: fn(u8, u8, u8) -> Instruction| -> Result<Instruction, DecodeError> {
            Ok(f(r(1)?, r(2)?, r(3)?))
        };

        match op {
            0x00 => Ok(Self::Halt),
            0x01 => Ok(Self::Nop),
            0x02 => Ok(Self::Wfi),
            0x03 => Ok(Self::Eret),
            0x04 => Ok(Self::Break),

            0x10 => Ok(Self::AddI(r(1)?, r(2)?, u32_at(3)?)),
            0x11 => Ok(Self::SubI(r(1)?, r(2)?, u32_at(3)?)),
            0x12 => Ok(Self::AndI(r(1)?, r(2)?, u32_at(3)?)),
            0x13 => Ok(Self::OrI(r(1)?, r(2)?, u32_at(3)?)),
            0x14 => Ok(Self::XorI(r(1)?, r(2)?, u32_at(3)?)),
            0x15 => Ok(Self::MulI(r(1)?, r(2)?, u32_at(3)?)),
            0x16 => Ok(Self::SltI(r(1)?, r(2)?, u32_at(3)?)),
            0x17 => Ok(Self::SltIU(r(1)?, r(2)?, u32_at(3)?)),

            0x20 => i3(Self::Add),
            0x21 => i3(Self::Sub),
            0x22 => i3(Self::Mul),
            0x23 => i3(Self::Div),
            0x24 => i3(Self::DivU),
            0x25 => i3(Self::Rem),
            0x26 => i3(Self::RemU),
            0x27 => Ok(Self::Cmp(r(1)?, r(2)?)),
            0x28 => i3(Self::Min),
            0x29 => i3(Self::Max),

            0x30 => i3(Self::And),
            0x31 => i3(Self::Or),
            0x32 => i3(Self::Xor),
            0x33 => Ok(Self::Not(r(1)?, r(2)?)),
            0x34 => Ok(Self::Clz(r(1)?, r(2)?)),
            0x35 => Ok(Self::Ctz(r(1)?, r(2)?)),
            0x36 => Ok(Self::Popcnt(r(1)?, r(2)?)),

            0x40 => i3(Self::Shl),
            0x41 => i3(Self::Shr),
            0x42 => i3(Self::Sar),
            0x43 => i3(Self::Rol),
            0x44 => i3(Self::Ror),

            0x50 => Ok(Self::Lb(r(1)?, r(2)?, u32_at(3)?)),
            0x51 => Ok(Self::Lbu(r(1)?, r(2)?, u32_at(3)?)),
            0x52 => Ok(Self::Lh(r(1)?, r(2)?, u32_at(3)?)),
            0x53 => Ok(Self::Lhu(r(1)?, r(2)?, u32_at(3)?)),
            0x54 => Ok(Self::Lw(r(1)?, r(2)?, u32_at(3)?)),
            0x55 => Ok(Self::Sb(r(1)?, r(2)?, u32_at(3)?)),
            0x56 => Ok(Self::Sh(r(1)?, r(2)?, u32_at(3)?)),
            0x57 => Ok(Self::Sw(r(1)?, r(2)?, u32_at(3)?)),
            0x58 => Ok(Self::Lrw(r(1)?, r(2)?)),
            0x59 => Ok(Self::Scw(r(1)?, r(2)?)),

            0x60 => Ok(Self::Jmp(u32_at(1)?)),
            0x61 => Ok(Self::Jz(u32_at(1)?)),
            0x62 => Ok(Self::Jnz(u32_at(1)?)),
            0x63 => Ok(Self::Jc(u32_at(1)?)),
            0x64 => Ok(Self::Jn(u32_at(1)?)),
            0x65 => Ok(Self::Jo(u32_at(1)?)),
            0x66 => Ok(Self::Jge(u32_at(1)?)),
            0x67 => Ok(Self::Jl(u32_at(1)?)),
            0x68 => Ok(Self::Jgt(u32_at(1)?)),
            0x69 => Ok(Self::Jle(u32_at(1)?)),
            0x6a => Ok(Self::Jcs(u32_at(1)?)),
            0x6b => Ok(Self::Jcc(u32_at(1)?)),

            0x70 => Ok(Self::Push(r(1)?)),
            0x71 => Ok(Self::Pop(r(1)?)),
            0x72 => Ok(Self::Call(u32_at(1)?)),
            0x73 => Ok(Self::Ret),

            0x90 => Ok(Self::Csrr(r(1)?, u16_at(2)?)),
            0x91 => Ok(Self::Csrw(r(1)?, u16_at(2)?)),
            0x92 => Ok(Self::Csrs(r(1)?, u16_at(2)?)),
            0x93 => Ok(Self::Csrc(r(1)?, u16_at(2)?)),

            0xa0 => Ok(Self::Mret),
            0xa1 => Ok(Self::Sret),
            0xa2 => Ok(Self::SetMode(r(1)?)),

            0xb0 => Ok(Self::Int(u32_at(1)?)),
            0xb1 => Ok(Self::Iret),

            _ => Err(DecodeError::InvalidOpcode(op)),
        }
    }

    pub fn size(&self) -> u32 {
        match self {
            Self::Halt
            | Self::Nop
            | Self::Wfi
            | Self::Eret
            | Self::Break
            | Self::Ret
            | Self::Mret
            | Self::Sret
            | Self::Iret => 1,

            Self::AddI(_, _, _)
            | Self::SubI(_, _, _)
            | Self::AndI(_, _, _)
            | Self::OrI(_, _, _)
            | Self::XorI(_, _, _)
            | Self::MulI(_, _, _)
            | Self::SltI(_, _, _)
            | Self::SltIU(_, _, _) => 7,

            Self::Add(_, _, _)
            | Self::Sub(_, _, _)
            | Self::Mul(_, _, _)
            | Self::Div(_, _, _)
            | Self::DivU(_, _, _)
            | Self::Rem(_, _, _)
            | Self::RemU(_, _, _)
            | Self::Min(_, _, _)
            | Self::Max(_, _, _)
            | Self::And(_, _, _)
            | Self::Or(_, _, _)
            | Self::Xor(_, _, _)
            | Self::Shl(_, _, _)
            | Self::Shr(_, _, _)
            | Self::Sar(_, _, _)
            | Self::Rol(_, _, _)
            | Self::Ror(_, _, _) => 4,

            Self::Cmp(_, _)
            | Self::Not(_, _)
            | Self::Clz(_, _)
            | Self::Ctz(_, _)
            | Self::Popcnt(_, _)
            | Self::Lrw(_, _)
            | Self::Scw(_, _)
            | Self::Push(_)
            | Self::Pop(_)
            | Self::SetMode(_) => 3,

            Self::Lb(_, _, _)
            | Self::Lbu(_, _, _)
            | Self::Lh(_, _, _)
            | Self::Lhu(_, _, _)
            | Self::Lw(_, _, _)
            | Self::Sb(_, _, _)
            | Self::Sh(_, _, _)
            | Self::Sw(_, _, _) => 7,

            Self::Jmp(_)
            | Self::Jz(_)
            | Self::Jnz(_)
            | Self::Jc(_)
            | Self::Jn(_)
            | Self::Jo(_)
            | Self::Jge(_)
            | Self::Jl(_)
            | Self::Jgt(_)
            | Self::Jle(_)
            | Self::Jcs(_)
            | Self::Jcc(_)
            | Self::Call(_)
            | Self::Int(_) => 5,

            Self::Csrr(_, _) | Self::Csrw(_, _) | Self::Csrs(_, _) | Self::Csrc(_, _) => 4,
        }
    }
}
