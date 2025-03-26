use std::fmt::Display;

use crate::imm::{I5, U3, U4};
use crate::reg::Reg;

#[derive(Debug, PartialEq)]
pub enum Op {
    BR(I5),
    BRZ(I5),
    ADDI(Reg, U3),
    SUBI(Reg, U3),
    SR0(U4),
    SRH0(U4),
    CLR(Reg),
    MOV(Reg, Reg),
    MOVA(Reg),
    MOVR(Reg),
    MOVRHS(Reg),
    PAUSE,
}

impl Op {
    pub fn to_string(&self) -> String {
        match self {
            Op::BR(imm) => format!("BR {}", imm.get()),
            Op::BRZ(imm) => format!("BRZ {}", imm.get()),
            Op::ADDI(reg, imm) => format!("ADDI {reg}, {}", imm.get()),
            Op::SUBI(reg, imm) => format!("SUBI {reg}, {}", imm.get()),
            Op::SR0(imm) => format!("SR0 {}", imm.get()),
            Op::SRH0(imm) => format!("SRH0 {}", imm.get()),
            Op::CLR(reg) => format!("CLR {reg}"),
            Op::MOV(regd, regs) => format!("MOV {regd}, {regs}"),
            Op::MOVA(reg) => format!("MOVA {reg}"),
            Op::MOVR(reg) => format!("MOVR {reg}"),
            Op::MOVRHS(reg) => format!("MOVRHS {reg}"),
            Op::PAUSE => format!("PAUSE"),
        }
    }

    pub fn to_binary(&self) -> u8 {
        match self {
            Op::BR(imm) => 0x80 | (imm.get() & 0x1f) as u8,
            Op::BRZ(imm) => 0xA0 | (imm.get() & 0x1f) as u8,
            Op::ADDI(reg, imm) => 0x00 | (imm.get() << 2) | *reg as u8,
            Op::SUBI(reg, imm) => 0x20 | (imm.get() << 2) | *reg as u8,
            Op::SR0(imm) => 0x40 | imm.get(),
            Op::SRH0(imm) => 0x50 | imm.get(),
            Op::CLR(reg) => 0x60 | *reg as u8,
            Op::MOV(regd, regs) => 0x70 | (*regd as u8) << 2 | *regs as u8,
            Op::MOVA(reg) => 0xc0 | *reg as u8,
            Op::MOVR(reg) => 0xc4 | *reg as u8,
            Op::MOVRHS(reg) => 0xc8 | *reg as u8,
            Op::PAUSE => 0xff,
        }
    }
}

#[derive(Debug)]
pub struct InvalidOpcode(u8);

impl Display for InvalidOpcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid opcode {:08b}.", self.0)
    }
}

impl TryFrom<u8> for Op {
    type Error = InvalidOpcode;

    fn try_from(opcode: u8) -> Result<Self, Self::Error> {
        // unwrapping is safe since the bit mask limits the value
        fn to_reg(val: u8) -> Reg {
            val.try_into().unwrap()
        }
        fn to_i5(val: u8) -> I5 {
            let sign_extend: i8 = (val & 0x1f | if val & 0x10 != 0 { 0xe0 } else { 0x00 }) as i8;
            sign_extend.try_into().unwrap()
        }
        fn to_u3(val: u8) -> U3 {
            val.try_into().unwrap()
        }
        fn to_u4(val: u8) -> U4 {
            val.try_into().unwrap()
        }

        match opcode {
            x if x >> 5 == 0b100 => Ok(Op::BR(to_i5(opcode & 0x1f))),
            x if x >> 5 == 0b101 => Ok(Op::BRZ(to_i5(opcode & 0x1f))),
            x if x >> 5 == 0b000 => {
                Ok(Op::ADDI(to_reg(opcode & 0x03), to_u3((opcode >> 2) & 0x07)))
            }
            x if x >> 5 == 0b001 => {
                Ok(Op::SUBI(to_reg(opcode & 0x03), to_u3((opcode >> 2) & 0x07)))
            }
            x if x >> 4 == 0b0100 => Ok(Op::SR0(to_u4(opcode & 0x0f))),
            x if x >> 4 == 0b0101 => Ok(Op::SRH0(to_u4(opcode & 0x0f))),
            x if x >> 2 == 0b011000 => Ok(Op::CLR(to_reg(opcode & 0x03))),
            x if x >> 4 == 0b0111 => {
                Ok(Op::MOV(to_reg((opcode >> 2) & 0x03), to_reg(opcode & 0x03)))
            }
            x if x >> 2 == 0b110000 => Ok(Op::MOVA(to_reg(opcode & 0x03))),
            x if x >> 2 == 0b110001 => Ok(Op::MOVR(to_reg(opcode & 0x03))),
            x if x >> 2 == 0b110010 => Ok(Op::MOVRHS(to_reg(opcode & 0x03))),
            0xff => Ok(Op::PAUSE),
            _ => Err(InvalidOpcode(opcode)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary() {
        let data: Vec<(Op, u8)> = vec![
            // Flow
            (Op::BR(I5::new(-16).unwrap()), 0b100_10000),
            (Op::BR(I5::new(15).unwrap()), 0b100_01111),
            (Op::BR(I5::new(-5).unwrap()), 0b100_11011),
            (Op::BR(I5::new(3).unwrap()), 0b100_00011),
            (Op::BRZ(I5::new(14).unwrap()), 0b101_01110),
            (Op::PAUSE, 0b11111111),
            // ALU
            (Op::ADDI(Reg::R0, U3::new(0).unwrap()), 0b000_000_00),
            (Op::ADDI(Reg::R1, U3::new(2).unwrap()), 0b000_010_01),
            (Op::ADDI(Reg::R2, U3::new(5).unwrap()), 0b000_101_10),
            (Op::ADDI(Reg::R3, U3::new(7).unwrap()), 0b000_111_11),
            (Op::SUBI(Reg::R0, U3::new(1).unwrap()), 0b001_001_00),
            (Op::SUBI(Reg::R1, U3::new(3).unwrap()), 0b001_011_01),
            (Op::SUBI(Reg::R2, U3::new(4).unwrap()), 0b001_100_10),
            (Op::SUBI(Reg::R3, U3::new(6).unwrap()), 0b001_110_11),
            (Op::SR0(U4::new(0).unwrap()), 0b0100_0000),
            (Op::SR0(U4::new(5).unwrap()), 0b0100_0101),
            (Op::SR0(U4::new(10).unwrap()), 0b0100_1010),
            (Op::SR0(U4::new(15).unwrap()), 0b0100_1111),
            (Op::SRH0(U4::new(1).unwrap()), 0b0101_0001),
            (Op::SRH0(U4::new(6).unwrap()), 0b0101_0110),
            (Op::SRH0(U4::new(11).unwrap()), 0b0101_1011),
            (Op::SRH0(U4::new(14).unwrap()), 0b0101_1110),
            // Memory
            (Op::MOV(Reg::R1, Reg::R2), 0b0111_01_10),
            (Op::CLR(Reg::R0), 0b011000_00),
            (Op::CLR(Reg::R1), 0b011000_01),
            (Op::CLR(Reg::R2), 0b011000_10),
            (Op::CLR(Reg::R3), 0b011000_11),
            (Op::MOV(Reg::R0, Reg::R2), 0b0111_00_10),
            (Op::MOV(Reg::R3, Reg::R1), 0b0111_11_01),
            // Motor
            (Op::MOVA(Reg::R0), 0b110000_00),
            (Op::MOVA(Reg::R1), 0b110000_01),
            (Op::MOVA(Reg::R2), 0b110000_10),
            (Op::MOVA(Reg::R3), 0b110000_11),
            (Op::MOVR(Reg::R0), 0b110001_00),
            (Op::MOVR(Reg::R1), 0b110001_01),
            (Op::MOVR(Reg::R2), 0b110001_10),
            (Op::MOVR(Reg::R3), 0b110001_11),
            (Op::MOVRHS(Reg::R0), 0b110010_00),
            (Op::MOVRHS(Reg::R1), 0b110010_01),
            (Op::MOVRHS(Reg::R2), 0b110010_10),
            (Op::MOVRHS(Reg::R3), 0b110010_11),
        ];
        for (op, code) in data {
            assert_eq!(op, code.try_into().unwrap(), "Failed {code:08b} to ASM",);
            assert_eq!(
                op.to_binary(),
                code,
                "Failed \"{}\" to binary",
                op.to_string()
            );
        }
    }
}
