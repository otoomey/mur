use std::fmt::Display;

use tabled::{builder::Builder, settings::Style};

use crate::{exception::Exception, bus::Bus, mem::{B8, B16, B32, B64}};

const RVABI: [&str; 32] = [
    "zero", "ra", "sp", "gp", "tp", "t0", "t1", "t2", 
    "s0", "s1", "a0", "a1", "a2", "a3", "a4", "a5", 
    "a6", "a7", "s2", "s3", "s4", "s5", "s6", "s7", 
    "s8", "s9", "s10", "s11", "t3", "t4", "t5", "t6",
];

pub trait Extension {
    fn id(ins: u32) -> Result<Self, Exception> where Self: Sized;
    fn ex(self, regs: &[u64; 32]) -> Self;
    fn wr(self, pc: u64, regs: &mut [u64; 32], bus: &mut Bus) -> Result<u64, Exception>;
    fn src_regs(&self) -> Vec<u64>;
    fn dst_reg(&self) -> Option<u64>;
    fn src_mem_addr(&self) -> Option<u64>;
    fn dst_mem_addr(&self) -> Option<u64>;
    fn is_ld(&self) -> bool;
    fn is_st(&self) -> bool;
    fn is_br(&self) -> bool;
    fn is_jmp(&self) -> bool;
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Rv32i {
    Lui { rd: u64, imm: u64 },
    Auipc { rd: u64, imm: u64 },
    Jal { rd: u64, imm: u64 },
    Jalr { rd: u64, rs1: u64, imm: u64 },
    Beq { rs1: u64, rs2: u64, imm: u64 },
    Bne { rs1: u64, rs2: u64, imm: u64 },
    Blt { rs1: u64, rs2: u64, imm: u64 },
    Bge { rs1: u64, rs2: u64, imm: u64 },
    Bltu { rs1: u64, rs2: u64, imm: u64 },
    Bgeu { rs1: u64, rs2: u64, imm: u64 },
    Lb { rd: u64, rs1: u64, imm: u64 },
    Lh { rd: u64, rs1: u64, imm: u64 },
    Lw { rd: u64, rs1: u64, imm: u64 },
    Lbu { rd: u64, rs1: u64, imm: u64 },
    Lhu { rd: u64, rs1: u64, imm: u64 },
    Sb { rs1: u64, rs2: u64, imm: u64 },
    Sh { rs1: u64, rs2: u64, imm: u64 },
    Sw { rs1: u64, rs2: u64, imm: u64 },
    Addi { rd: u64, rs1: u64, imm: u64 },
    Slti { rd: u64, rs1: u64, imm: u64 },
    Sltiu { rd: u64, rs1: u64, imm: u64 },
    Xori { rd: u64, rs1: u64, imm: u64 },
    Ori { rd: u64, rs1: u64, imm: u64 },
    Andi { rd: u64, rs1: u64, imm: u64 },
    Slli { rd: u64, rs1: u64, shamt: u32 },
    Srli { rd: u64, rs1: u64, shamt: u32 },
    Srai { rd: u64, rs1: u64, shamt: u32 },
    Add { rd: u64, rs1: u64, rs2: u64 },
    Sub { rd: u64, rs1: u64, rs2: u64 },
    Sll { rd: u64, rs1: u64, rs2: u64 },
    Slt { rd: u64, rs1: u64, rs2: u64 },
    Sltu { rd: u64, rs1: u64, rs2: u64 },
    Xor { rd: u64, rs1: u64, rs2: u64 },
    Srl { rd: u64, rs1: u64, rs2: u64 },
    Sra { rd: u64, rs1: u64, rs2: u64 },
    Or { rd: u64, rs1: u64, rs2: u64 },
    And { rd: u64, rs1: u64, rs2: u64 }
}

#[derive(Debug, PartialEq)]
pub enum Rv64i {
    Lwu { rd: u64, rs1: u64, imm: u64 },
    Ld { rd: u64, rs1: u64, imm: u64 },
    Sd { rs1: u64, rs2: u64, imm: u64 },
    Addiw { rd: u64, rs1: u64, imm: u64 },
    Slliw { rd: u64, rs1: u64, shamt: u32 },
    Srliw { rd: u64, rs1: u64, shamt: u32 },
    Sraiw { rd: u64, rs1: u64, shamt: u32 },
    Addw { rd: u64, rs1: u64, rs2: u64 },
    Subw { rd: u64, rs1: u64, rs2: u64 },
    Sllw { rd: u64, rs1: u64, rs2: u64 },
    Srlw { rd: u64, rs1: u64, rs2: u64 },
    Sraw { rd: u64, rs1: u64, rs2: u64 },
}

impl Extension for Rv32i {
    fn id(ins: u32) -> Result<Self, Exception> {
        let opcode = opcode(ins);
        let funct3 = funct3(ins);
        let funct7 = funct7(ins);

        let rd = rd(ins) as u64;
        let rs1 = rs1(ins) as u64;
        let rs2 = rs2(ins) as u64;

        let i_imm = i_imm(ins);
        let s_imm = s_imm(ins);
        let j_imm = j_imm(ins);
        let b_imm = b_imm(ins);
        let u_imm = u_imm(ins);

        match (funct7, funct3, opcode) {
            (_, _, 0b0110111) => Ok(Self::Lui { rd, imm: u_imm }),
            (_, _, 0b0010111) => Ok(Self::Auipc { rd, imm: u_imm }),
            (_, _, 0b1101111) => Ok(Self::Jal { rd, imm: j_imm }),
            (_, 0b000, 0b1100111) => Ok(Self::Jalr { rd, rs1, imm: i_imm }),
            (_, 0b000, 0b1100011) => Ok(Self::Beq { rs1, rs2, imm: b_imm }),
            (_, 0b001, 0b1100011) => Ok(Self::Bne { rs1, rs2, imm: b_imm }),
            (_, 0b100, 0b1100011) => Ok(Self::Blt { rs1, rs2, imm: b_imm }),
            (_, 0b101, 0b1100011) => Ok(Self::Bge { rs1, rs2, imm: b_imm }),
            (_, 0b110, 0b1100011) => Ok(Self::Bltu { rs1, rs2, imm: b_imm }),
            (_, 0b111, 0b1100011) => Ok(Self::Bgeu { rs1, rs2, imm: b_imm }),
            (_, 0b000, 0b0000011) => Ok(Self::Lb { rd, rs1, imm: i_imm }),
            (_, 0b001, 0b0000011) => Ok(Self::Lh { rd, rs1, imm: i_imm }),
            (_, 0b010, 0b0000011) => Ok(Self::Lw { rd, rs1, imm: i_imm }),
            (_, 0b100, 0b0000011) => Ok(Self::Lbu { rd, rs1, imm: i_imm }),
            (_, 0b101, 0b0000011) => Ok(Self::Lhu { rd, rs1, imm: i_imm }),
            (_, 0b000, 0b0100011) => Ok(Self::Sb { rs1, rs2, imm: s_imm }),
            (_, 0b001, 0b0100011) => Ok(Self::Sh { rs1, rs2, imm: s_imm }),
            (_, 0b010, 0b0100011) => Ok(Self::Sw { rs1, rs2, imm: s_imm }),
            (_, 0b000, 0b0010011) => Ok(Self::Addi { rd, rs1, imm: i_imm }),
            (_, 0b010, 0b0010011) => Ok(Self::Slti { rd, rs1, imm: i_imm }),
            (_, 0b011, 0b0010011) => Ok(Self::Sltiu { rd, rs1, imm: i_imm }),
            (_, 0b100, 0b0010011) => Ok(Self::Xori { rd, rs1, imm: i_imm }),
            (_, 0b110, 0b0010011) => Ok(Self::Ori { rd, rs1, imm: i_imm }),
            (_, 0b111, 0b0010011) => Ok(Self::Andi { rd, rs1, imm: i_imm }),
            (0b0000000, 0b001, 0b0010011) => Ok(Self::Slli { rd, rs1, shamt: (i_imm as u32) & 0xf }),
            (0b0000000, 0b101, 0b0010011) => Ok(Self::Srli { rd, rs1, shamt: (i_imm as u32) & 0xf }),
            (0b0100000, 0b101, 0b0010011) => Ok(Self::Srai { rd, rs1, shamt: (i_imm as u32) & 0xf }),
            (0b0000000, 0b000, 0b0110011) => Ok(Self::Add { rd, rs1, rs2 }),
            (0b0100000, 0b000, 0b0110011) => Ok(Self::Sub { rd, rs1, rs2 }),
            (0b0000000, 0b001, 0b0110011) => Ok(Self::Sll { rd, rs1, rs2 }),
            (0b0000000, 0b010, 0b0110011) => Ok(Self::Slt { rd, rs1, rs2 }),
            (0b0000000, 0b011, 0b0110011) => Ok(Self::Sltu { rd, rs1, rs2 }),
            (0b0000000, 0b100, 0b0110011) => Ok(Self::Xor { rd, rs1, rs2 }),
            (0b0000000, 0b101, 0b0110011) => Ok(Self::Srl { rd, rs1, rs2 }),
            (0b0100000, 0b101, 0b0110011) => Ok(Self::Sra { rd, rs1, rs2 }),
            (0b0000000, 0b110, 0b0110011) => Ok(Self::Or { rd, rs1, rs2 }),
            (0b0000000, 0b111, 0b0110011) => Ok(Self::And { rd, rs1, rs2 }),
            _ => Err(Exception::IllegalInstruction(ins as u64))
        }
    }

    fn ex(self, regs: &[u64; 32]) -> Self {
        match self {
            Rv32i::Lui { rd, imm } => Self::Lui { rd, imm },
            Rv32i::Auipc { rd, imm } => Self::Auipc { rd, imm },
            Rv32i::Jal { rd, imm } => Self::Jal { rd, imm },
            Rv32i::Jalr { rd, rs1, imm } => Self::Jalr { rd, rs1: regs[rs1 as usize], imm },
            Rv32i::Beq { rs1, rs2, imm } => Self::Beq { rs1: regs[rs1 as usize], rs2: regs[rs2 as usize], imm },
            Rv32i::Bne { rs1, rs2, imm } => Self::Bne { rs1: regs[rs1 as usize], rs2: regs[rs2 as usize], imm },
            Rv32i::Blt { rs1, rs2, imm } => Self::Blt { rs1: regs[rs1 as usize], rs2: regs[rs2 as usize], imm },
            Rv32i::Bge { rs1, rs2, imm } => Self::Bge { rs1: regs[rs1 as usize], rs2: regs[rs2 as usize], imm },
            Rv32i::Bltu { rs1, rs2, imm } => Self::Bltu { rs1: regs[rs1 as usize], rs2: regs[rs2 as usize], imm },
            Rv32i::Bgeu { rs1, rs2, imm } => Self::Bgeu { rs1: regs[rs1 as usize], rs2: regs[rs2 as usize], imm },
            Rv32i::Lb { rd, rs1, imm } => Self::Lb { rd, rs1: regs[rs1 as usize], imm },
            Rv32i::Lh { rd, rs1, imm } => Self::Lh { rd, rs1: regs[rs1 as usize], imm },
            Rv32i::Lw { rd, rs1, imm } => Self::Lw { rd, rs1: regs[rs1 as usize], imm },
            Rv32i::Lbu { rd, rs1, imm } => Self::Lbu { rd, rs1: regs[rs1 as usize], imm },
            Rv32i::Lhu { rd, rs1, imm } => Self::Lhu { rd, rs1: regs[rs1 as usize], imm },
            Rv32i::Sb { rs1, rs2, imm } => Self::Sb { rs1: regs[rs1 as usize], rs2: regs[rs2 as usize], imm },
            Rv32i::Sh { rs1, rs2, imm } => Self::Sh { rs1: regs[rs1 as usize], rs2: regs[rs2 as usize], imm },
            Rv32i::Sw { rs1, rs2, imm } => Self::Sw { rs1: regs[rs1 as usize], rs2: regs[rs2 as usize], imm },
            Rv32i::Addi { rd, rs1, imm } => Self::Addi { rd, rs1: regs[rs1 as usize], imm },
            Rv32i::Slti { rd, rs1, imm } => Self::Slti { rd, rs1: regs[rs1 as usize], imm },
            Rv32i::Sltiu { rd, rs1, imm } => Self::Sltiu { rd, rs1: regs[rs1 as usize], imm },
            Rv32i::Xori { rd, rs1, imm } => Self::Xori { rd, rs1: regs[rs1 as usize], imm },
            Rv32i::Ori { rd, rs1, imm } => Self::Ori { rd, rs1: regs[rs1 as usize], imm },
            Rv32i::Andi { rd, rs1, imm } => Self::Andi { rd, rs1: regs[rs1 as usize], imm },
            Rv32i::Slli { rd, rs1, shamt } => Self::Slli { rd, rs1: regs[rs1 as usize], shamt: shamt & 0x1f },
            Rv32i::Srli { rd, rs1, shamt } => Self::Srli { rd, rs1: regs[rs1 as usize], shamt: shamt & 0x1f },
            Rv32i::Srai { rd, rs1, shamt } => Self::Srai { rd, rs1: regs[rs1 as usize], shamt: shamt & 0x1f },
            Rv32i::Add { rd, rs1, rs2 } => Self::Add { rd, rs1: regs[rs1 as usize], rs2: regs[rs2 as usize] },
            Rv32i::Sub { rd, rs1, rs2 } => Self::Sub { rd, rs1: regs[rs1 as usize], rs2: regs[rs2 as usize] },
            Rv32i::Sll { rd, rs1, rs2 } => Self::Sll { rd, rs1: regs[rs1 as usize], rs2: regs[rs2 as usize] },
            Rv32i::Slt { rd, rs1, rs2 } => Self::Slt { rd, rs1: regs[rs1 as usize], rs2: regs[rs2 as usize] },
            Rv32i::Sltu { rd, rs1, rs2 } => Self::Sltu { rd, rs1: regs[rs1 as usize], rs2: regs[rs2 as usize] },
            Rv32i::Xor { rd, rs1, rs2 } => Self::Xor { rd, rs1: regs[rs1 as usize], rs2: regs[rs2 as usize] },
            Rv32i::Srl { rd, rs1, rs2 } => Self::Srl { rd, rs1: regs[rs1 as usize], rs2: regs[rs2 as usize] },
            Rv32i::Sra { rd, rs1, rs2 } => Self::Sra { rd, rs1: regs[rs1 as usize], rs2: regs[rs2 as usize] },
            Rv32i::Or { rd, rs1, rs2 } => Self::Or { rd, rs1: regs[rs1 as usize], rs2: regs[rs2 as usize] },
            Rv32i::And { rd, rs1, rs2 } => Self::And { rd, rs1: regs[rs1 as usize], rs2: regs[rs2 as usize] },
        }
    }

    fn wr(self, pc: u64, regs: &mut [u64; 32], bus: &mut Bus) -> Result<u64, Exception> {
        match self {
            Rv32i::Lui { rd, imm } => {
                regs[rd as usize] = imm;
                Ok(pc.wrapping_add(4))
            },
            Rv32i::Auipc { rd, imm } => {
                regs[rd as usize] = pc.wrapping_add(imm);
                Ok(pc.wrapping_add(4))
            },
            Rv32i::Jal { rd, imm } => {
                regs[rd as usize] = pc.wrapping_add(4);
                Ok(pc.wrapping_add(imm) as u64)
            },
            Rv32i::Jalr { rd, rs1, imm } => {
                regs[rd as usize] = pc.wrapping_add(4);
                Ok((rs1.wrapping_add(imm) as u64) & !1)
            },
            Rv32i::Beq { rs1, rs2, imm } => {
                Ok(if rs1 == rs2 { pc.wrapping_add(imm) as u64 } else { pc.wrapping_add(4) })
            },
            Rv32i::Bne { rs1, rs2, imm } => {
                Ok(if rs1 != rs2 { pc.wrapping_add(imm ) as u64 } else { pc.wrapping_add(4) })
            },
            Rv32i::Blt { rs1, rs2, imm } => {
                Ok(if (rs1 as i64) < (rs2 as i64) { (pc as i64).wrapping_add(imm as i64) as u64 } else { pc.wrapping_add(4) })
            },
            Rv32i::Bge { rs1, rs2, imm } => {
                Ok(if (rs1 as i64) >= (rs2 as i64) { (pc as i64).wrapping_add(imm as i64) as u64 } else { pc.wrapping_add(4) })
            },
            Rv32i::Bltu { rs1, rs2, imm } => {
                Ok(if rs1 < rs2 { pc.wrapping_add(imm) as u64 } else { pc.wrapping_add(4) })
            },
            Rv32i::Bgeu { rs1, rs2, imm } => {
                Ok(if rs1 >= rs2 { pc.wrapping_add(imm) as u64 } else { pc.wrapping_add(4)})
            },
            Rv32i::Lb { rd, rs1, imm } => {
                let addr = rs1.wrapping_add(imm);
                regs[rd as usize] = bus.load(addr as u64, B8)? as i8 as i64 as u64;
                Ok(pc.wrapping_add(4))
            },
            Rv32i::Lh { rd, rs1, imm } => {
                let addr = rs1.wrapping_add(imm);
                regs[rd as usize] = bus.load(addr as u64, B16)? as i16 as i64 as u64;
                Ok(pc.wrapping_add(4))
            },
            Rv32i::Lw { rd, rs1, imm } => {
                let addr = rs1.wrapping_add(imm);
                regs[rd as usize] = bus.load(addr as u64, B32)? as i32 as i64 as u64;
                Ok(pc.wrapping_add(4))
            },
            Rv32i::Lbu { rd, rs1, imm } => {
                let addr = rs1.wrapping_add(imm);
                regs[rd as usize] = bus.load(addr as u64, B8)?;
                Ok(pc.wrapping_add(4))
            },
            Rv32i::Lhu { rd, rs1, imm } => {
                let addr = rs1.wrapping_add(imm);
                regs[rd as usize] = bus.load(addr as u64, B16)?;
                Ok(pc.wrapping_add(4))
            },
            Rv32i::Sb { rs1, rs2, imm } => {
                let addr = rs1.wrapping_add(imm);
                bus.store(addr as u64, B8, rs2 & 0xff)?;
                Ok(pc.wrapping_add(4))
            },
            Rv32i::Sh { rs1, rs2, imm } => {
                let addr = rs1.wrapping_add(imm);
                bus.store(addr as u64, B16, rs2 & 0xffff)?;
                Ok(pc.wrapping_add(4))
            },
            Rv32i::Sw { rs1, rs2, imm } => {
                let addr = rs1.wrapping_add(imm);
                bus.store(addr as u64, B32, rs2 & 0xffffffff)?;
                Ok(pc.wrapping_add(4))
            },
            Rv32i::Addi { rd, rs1, imm } => {
                regs[rd as usize] = rs1.wrapping_add(imm);
                Ok(pc.wrapping_add(4))
            },
            Rv32i::Slti { rd, rs1, imm } => {
                regs[rd as usize] = if (rs1 as i64) < (imm as i64) { 1 } else { 0 };
                Ok(pc.wrapping_add(4))
            },
            Rv32i::Sltiu { rd, rs1, imm } => {
                regs[rd as usize] = if rs1 < imm { 1 } else { 0 };
                Ok(pc.wrapping_add(4))
            },
            Rv32i::Xori { rd, rs1, imm } => {
                regs[rd as usize] = rs1 ^ imm;
                Ok(pc.wrapping_add(4))
            },
            Rv32i::Ori { rd, rs1, imm } => {
                regs[rd as usize] = rs1 | imm;
                Ok(pc.wrapping_add(4))
            },
            Rv32i::Andi { rd, rs1, imm } => {
                regs[rd as usize] = rs1 & imm;
                Ok(pc.wrapping_add(4))
            },
            Rv32i::Slli { rd, rs1, shamt } => {
                regs[rd as usize] = rs1.wrapping_shl(shamt);
                Ok(pc.wrapping_add(4))
            },
            Rv32i::Srli { rd, rs1, shamt } => {
                regs[rd as usize] = rs1.wrapping_shr(shamt);
                Ok(pc.wrapping_add(4))
            },
            Rv32i::Srai { rd, rs1, shamt } => {
                regs[rd as usize] = ((rs1 as i64).wrapping_shr(shamt)) as u64;
                Ok(pc.wrapping_add(4))
            },
            Rv32i::Add { rd, rs1, rs2 } => {
                regs[rd as usize] = rs1.wrapping_add(rs2);
                Ok(pc.wrapping_add(4))
            },
            Rv32i::Sub { rd, rs1, rs2 } => {
                regs[rd as usize] = rs1.wrapping_sub(rs2);
                Ok(pc.wrapping_add(4))
            },
            Rv32i::Sll { rd, rs1, rs2 } => {
                regs[rd as usize] = rs1.wrapping_shl(rs2 as u32);
                Ok(pc.wrapping_add(4))
            },
            Rv32i::Slt { rd, rs1, rs2 } => {
                regs[rd as usize] = if (rs1 as i64) < (rs2 as i64) { 1 } else { 0 };
                Ok(pc.wrapping_add(4))
            },
            Rv32i::Sltu { rd, rs1, rs2 } => {
                regs[rd as usize] = if rs1 < rs2 { 1 } else { 0 };
                Ok(pc.wrapping_add(4))
            },
            Rv32i::Xor { rd, rs1, rs2 } => {
                regs[rd as usize] = rs1 ^ rs2;
                Ok(pc.wrapping_add(4))
            },
            Rv32i::Srl { rd, rs1, rs2 } => {
                regs[rd as usize] = rs1.wrapping_shr(rs2 as u32);
                Ok(pc.wrapping_add(4))
            },
            Rv32i::Sra { rd, rs1, rs2 } => {
                regs[rd as usize] = ((rs1 as i64).wrapping_shr(rs2 as u32)) as u64;
                Ok(pc.wrapping_add(4))
            },
            Rv32i::Or { rd, rs1, rs2 } => {
                regs[rd as usize] = rs1 | rs2;
                Ok(pc.wrapping_add(4))
            },
            Rv32i::And { rd, rs1, rs2 } => {
                regs[rd as usize] = rs1 & rs2;
                Ok(pc.wrapping_add(4))
            },
        }
    }

    fn src_regs(&self) -> Vec<u64> {
        match self {
            Rv32i::Lui { .. } => vec![],
            Rv32i::Auipc { .. } => vec![],
            Rv32i::Jal { .. } => vec![],
            Rv32i::Jalr { rs1, .. } => vec![*rs1],
            Rv32i::Beq { rs1, rs2, .. } => vec![*rs1, *rs2],
            Rv32i::Bne { rs1, rs2, .. } => vec![*rs1, *rs2],
            Rv32i::Blt { rs1, rs2, .. } => vec![*rs1, *rs2],
            Rv32i::Bge { rs1, rs2, .. } => vec![*rs1, *rs2],
            Rv32i::Bltu { rs1, rs2, .. } => vec![*rs1, *rs2],
            Rv32i::Bgeu { rs1, rs2, .. } => vec![*rs1, *rs2],
            Rv32i::Lb { rs1, .. } => vec![*rs1],
            Rv32i::Lh { rs1, .. } => vec![*rs1],
            Rv32i::Lw { rs1, .. } => vec![*rs1],
            Rv32i::Lbu { rs1, .. } => vec![*rs1],
            Rv32i::Lhu { rs1, .. } => vec![*rs1],
            Rv32i::Sb { rs1, rs2, .. } => vec![*rs1, *rs2],
            Rv32i::Sh { rs1, rs2, .. } => vec![*rs1, *rs2],
            Rv32i::Sw { rs1, rs2, .. } => vec![*rs1, *rs2],
            Rv32i::Addi { rs1, .. } => vec![*rs1],
            Rv32i::Slti { rs1, .. } => vec![*rs1],
            Rv32i::Sltiu { rs1, .. } => vec![*rs1],
            Rv32i::Xori { rs1, .. } => vec![*rs1],
            Rv32i::Ori { rs1, .. } => vec![*rs1],
            Rv32i::Andi { rs1, .. } => vec![*rs1],
            Rv32i::Slli { rs1, .. } => vec![*rs1],
            Rv32i::Srli { rs1, .. } => vec![*rs1],
            Rv32i::Srai { rs1, .. } => vec![*rs1],
            Rv32i::Add { rs1, rs2, .. } => vec![*rs1, *rs2],
            Rv32i::Sub { rs1, rs2, .. } => vec![*rs1, *rs2],
            Rv32i::Sll { rs1, rs2, .. } => vec![*rs1, *rs2],
            Rv32i::Slt { rs1, rs2, .. } => vec![*rs1, *rs2],
            Rv32i::Sltu { rs1, rs2, .. } => vec![*rs1, *rs2],
            Rv32i::Xor { rs1, rs2, .. } => vec![*rs1, *rs2],
            Rv32i::Srl { rs1, rs2, .. } => vec![*rs1, *rs2],
            Rv32i::Sra { rs1, rs2, .. } => vec![*rs1, *rs2],
            Rv32i::Or { rs1, rs2, .. } => vec![*rs1, *rs2],
            Rv32i::And { rs1, rs2, .. } => vec![*rs1, *rs2],
        }
    }

    fn dst_reg(&self) -> Option<u64> {
        match self {
            Rv32i::Lui { rd, .. } => Some(*rd),
            Rv32i::Auipc { rd, .. } => Some(*rd),
            Rv32i::Jal { rd, .. } => Some(*rd),
            Rv32i::Jalr { rd, .. } => Some(*rd),
            Rv32i::Beq { .. } => None,
            Rv32i::Bne { .. } => None,
            Rv32i::Blt { .. } => None,
            Rv32i::Bge { .. } => None,
            Rv32i::Bltu { .. } => None,
            Rv32i::Bgeu { .. } => None,
            Rv32i::Lb { rd, .. } => Some(*rd),
            Rv32i::Lh { rd, .. } => Some(*rd),
            Rv32i::Lw { rd, .. } => Some(*rd),
            Rv32i::Lbu { rd, .. } => Some(*rd),
            Rv32i::Lhu { rd, .. } => Some(*rd),
            Rv32i::Sb { .. } => None,
            Rv32i::Sh { .. } => None,
            Rv32i::Sw { .. } => None,
            Rv32i::Addi { rd, .. } => Some(*rd),
            Rv32i::Slti { rd, .. } => Some(*rd),
            Rv32i::Sltiu { rd, .. } => Some(*rd),
            Rv32i::Xori { rd, .. } => Some(*rd),
            Rv32i::Ori { rd, .. } => Some(*rd),
            Rv32i::Andi { rd, .. } => Some(*rd),
            Rv32i::Slli { rd, .. } => Some(*rd),
            Rv32i::Srli { rd, .. } => Some(*rd),
            Rv32i::Srai { rd, .. } => Some(*rd),
            Rv32i::Add { rd, .. } => Some(*rd),
            Rv32i::Sub { rd, .. } => Some(*rd),
            Rv32i::Sll { rd, .. } => Some(*rd),
            Rv32i::Slt { rd, .. } => Some(*rd),
            Rv32i::Sltu { rd, .. } => Some(*rd),
            Rv32i::Xor { rd, .. } => Some(*rd),
            Rv32i::Srl { rd, .. } => Some(*rd),
            Rv32i::Sra { rd, .. } => Some(*rd),
            Rv32i::Or { rd, .. } => Some(*rd),
            Rv32i::And { rd, .. } => Some(*rd),
        }
    }

    fn src_mem_addr(&self) -> Option<u64> {
        match self {
            Rv32i::Lb { rs1, imm, .. } => {
                Some(rs1.wrapping_add(*imm))
            },
            Rv32i::Lh { rs1, imm, .. } => {
                Some(rs1.wrapping_add(*imm))
            },
            Rv32i::Lw { rs1, imm, .. } => {
                Some(rs1.wrapping_add(*imm))
            },
            Rv32i::Lbu { rs1, imm, .. } => {
                Some(rs1.wrapping_add(*imm))
            },
            Rv32i::Lhu { rs1, imm, .. } => {
                Some(rs1.wrapping_add(*imm))
            },
            _ => None
        }
    }

    fn dst_mem_addr(&self) -> Option<u64> {
        match self {
            Rv32i::Sb { rs1, imm, .. } => {
                Some(rs1.wrapping_add(*imm))
            },
            Rv32i::Sh { rs1, imm, .. } => {
                Some(rs1.wrapping_add(*imm))
            },
            Rv32i::Sw { rs1, imm, .. } => {
                Some(rs1.wrapping_add(*imm))
            },
            _ => None
        }
    }

    fn is_ld(&self) -> bool {
        match self {
            Rv32i::Lb { .. } | 
            Rv32i::Lh { .. } |
            Rv32i::Lw { .. } |
            Rv32i::Lbu { .. } |
            Rv32i::Lhu { .. } => true,
            _ => false
        }
    }

    fn is_st(&self) -> bool {
        match self {
            Rv32i::Sb { .. } |
            Rv32i::Sh { .. } |
            Rv32i::Sw { .. } => { true },
            _ => false
        }
    }

    fn is_br(&self) -> bool {
        match self {
            Rv32i::Beq { .. } |
            Rv32i::Bne { .. } |
            Rv32i::Blt { .. } |
            Rv32i::Bge { .. } |
            Rv32i::Bltu { .. } |
            Rv32i::Bgeu { .. } => true,
            _ => false
        }
    }

    fn is_jmp(&self) -> bool {
        match self {
            Rv32i::Jal { .. } |
            Rv32i::Jalr { .. } => true,
            _ => false
        }
    }
}

impl Extension for Rv64i {
    fn id(ins: u32) -> Result<Self, Exception> {
        let opcode = opcode(ins);
        let funct3 = funct3(ins);
        let funct7 = funct7(ins);

        let rd = rd(ins) as u64;
        let rs1 = rs1(ins) as u64;
        let rs2 = rs2(ins) as u64;

        let i_imm = i_imm(ins);
        let s_imm = s_imm(ins);

        match (funct7, funct3, opcode) {
            (_, 0b110, 0b0000011) => Ok(Self::Lwu { rd, rs1, imm: i_imm }),
            (_, 0b011, 0b0000011) => Ok(Self::Ld { rd, rs1, imm: i_imm }),
            (_, 0b011, 0b0100011) => Ok(Self::Sd { rs1, rs2, imm: s_imm }),
            (_, 0b000, 0b0011011) => Ok(Self::Addiw { rd, rs1, imm: i_imm }),
            (0b0000000, 0b001, 0b0011011) => Ok(Self::Slliw { rd, rs1, shamt: (i_imm as u32) & 0xf }),
            (0b0000000, 0b101, 0b0011011) => Ok(Self::Srliw { rd, rs1, shamt: (i_imm as u32) & 0xf }),
            (0b0100000, 0b101, 0b0011011) => Ok(Self::Sraiw { rd, rs1, shamt: (i_imm as u32) & 0xf }),
            (0b0000000, 0b000, 0b0111011) => Ok(Self::Addw { rd, rs1, rs2 }),
            (0b0100000, 0b000, 0b0111011) => Ok(Self::Subw { rd, rs1, rs2 }),
            (0b0000000, 0b001, 0b0111011) => Ok(Self::Sllw { rd, rs1, rs2 }),
            (0b0000000, 0b101, 0b0111011) => Ok(Self::Srlw { rd, rs1, rs2 }),
            (0b0100000, 0b101, 0b0111011) => Ok(Self::Sraw { rd, rs1, rs2 }),
            _ => Err(Exception::IllegalInstruction(ins as u64))
        }
    }

    fn ex(self, regs: &[u64; 32]) -> Self {
        match self {
            Rv64i::Lwu { rd, rs1, imm } => Self::Lwu { rd, rs1: regs[rs1 as usize], imm },
            Rv64i::Ld { rd, rs1, imm } => Self::Ld { rd, rs1: regs[rs1 as usize], imm },
            Rv64i::Sd { rs1, rs2, imm } => Self::Sd { rs1: regs[rs1 as usize], rs2: regs[rs2 as usize], imm },
            Rv64i::Addiw { rd, rs1, imm } => Self::Addiw { rd, rs1: regs[rs1 as usize], imm },
            Rv64i::Slliw { rd, rs1, shamt } => Self::Slliw { rd, rs1: regs[rs1 as usize], shamt: shamt & 0x3f },
            Rv64i::Srliw { rd, rs1, shamt } => Self::Srliw { rd, rs1: regs[rs1 as usize], shamt: shamt & 0x3f },
            Rv64i::Sraiw { rd, rs1, shamt } => Self::Sraiw { rd, rs1: regs[rs1 as usize], shamt: shamt & 0x3f },
            Rv64i::Addw { rd, rs1, rs2 } => Self::Addw { rd, rs1: regs[rs1 as usize], rs2: regs[rs2 as usize] },
            Rv64i::Subw { rd, rs1, rs2 } => Self::Subw { rd, rs1: regs[rs1 as usize], rs2: regs[rs2 as usize] },
            Rv64i::Sllw { rd, rs1, rs2 } => Self::Sllw { rd, rs1: regs[rs1 as usize], rs2: regs[rs2 as usize] },
            Rv64i::Srlw { rd, rs1, rs2 } => Self::Srlw { rd, rs1: regs[rs1 as usize], rs2: regs[rs2 as usize] },
            Rv64i::Sraw { rd, rs1, rs2 } => Self::Sraw { rd, rs1: regs[rs1 as usize], rs2: regs[rs2 as usize] },
        }
    }

    fn wr(self, pc: u64, regs: &mut [u64; 32], bus: &mut Bus) -> Result<u64, Exception> {
        match self {
            Rv64i::Lwu { rd, rs1, imm } => {
                let addr = rs1.wrapping_add(imm);
                regs[rd as usize] = bus.load(addr, B64)?;
                Ok(pc.wrapping_add(4))
            },
            Rv64i::Ld { rd, rs1, imm } => {
                let addr = rs1.wrapping_add(imm);
                regs[rd as usize] = bus.load(addr as u64, B64)?;
                Ok(pc.wrapping_add(4))
            },
            Rv64i::Sd { rs1, rs2, imm } => {
                let addr = rs1.wrapping_add(imm);
                bus.store(addr as u64, B64, rs2)?;
                Ok(pc.wrapping_add(4))
            },
            Rv64i::Addiw { rd, rs1, imm } => {
                regs[rd as usize] = rs1.wrapping_add(imm) as i32 as i64 as u64;
                Ok(pc.wrapping_add(4))
            },
            Rv64i::Slliw { rd, rs1, shamt } => {
                regs[rd as usize] = rs1.wrapping_shl(shamt) as i32 as i64 as u64;
                Ok(pc.wrapping_add(4))
            },
            Rv64i::Srliw { rd, rs1, shamt } => {
                regs[rd as usize] = (rs1 as u32).wrapping_shr(shamt) as i32 as i64 as u64;
                Ok(pc.wrapping_add(4))
            },
            Rv64i::Sraiw { rd, rs1, shamt } => {
                regs[rd as usize] = ((rs1 as i32).wrapping_shr(shamt)) as i64 as u64;
                Ok(pc.wrapping_add(4))
            },
            Rv64i::Addw { rd, rs1, rs2 } => {
                regs[rd as usize] = rs1.wrapping_add(rs2) as i32 as u64;
                Ok(pc.wrapping_add(4))
            },
            Rv64i::Subw { rd, rs1, rs2 } => {
                regs[rd as usize] = rs1.wrapping_sub(rs2) as i32 as u64;
                Ok(pc.wrapping_add(4))
            },
            Rv64i::Sllw { rd, rs1, rs2 } => {
                regs[rd as usize] = (rs1 as u32).wrapping_shl(rs2 as u32) as i32 as u64;
                Ok(pc.wrapping_add(4))
            },
            Rv64i::Srlw { rd, rs1, rs2 } => {
                regs[rd as usize] = (rs1 as u32).wrapping_shr(rs2 as u32) as i32 as u64;
                Ok(pc.wrapping_add(4))
            },
            Rv64i::Sraw { rd, rs1, rs2 } => {
                regs[rd as usize] = (rs1 as i32).wrapping_shr(rs2 as u32) as u64;
                Ok(pc.wrapping_add(4))
            },
        }
    }

    fn src_regs(&self) -> Vec<u64> {
        match self {
            Rv64i::Lwu { rs1, .. } => vec![*rs1],
            Rv64i::Ld { rs1, .. } => vec![*rs1],
            Rv64i::Sd { rs1, rs2, .. } => vec![*rs1, *rs2],
            Rv64i::Addiw { rs1, .. } => vec![*rs1],
            Rv64i::Slliw { rs1, .. } => vec![*rs1],
            Rv64i::Srliw { rs1, .. } => vec![*rs1],
            Rv64i::Sraiw { rs1, .. } => vec![*rs1],
            Rv64i::Addw { rs1, rs2, .. } => vec![*rs1, *rs2],
            Rv64i::Subw { rs1, rs2, .. } => vec![*rs1, *rs2],
            Rv64i::Sllw { rs1, rs2, .. } => vec![*rs1, *rs2],
            Rv64i::Srlw { rs1, rs2, .. } => vec![*rs1, *rs2],
            Rv64i::Sraw { rs1, rs2, .. } => vec![*rs1, *rs2],
        }
    }

    fn dst_reg(&self) -> Option<u64> {
        match self {
            Rv64i::Lwu { rd, .. } => Some(*rd),
            Rv64i::Ld { rd, .. } => Some(*rd),
            Rv64i::Sd { .. } => None,
            Rv64i::Addiw { rd, .. } => Some(*rd),
            Rv64i::Slliw { rd, .. } => Some(*rd),
            Rv64i::Srliw { rd, .. } => Some(*rd),
            Rv64i::Sraiw { rd, .. } => Some(*rd),
            Rv64i::Addw { rd, .. } => Some(*rd),
            Rv64i::Subw { rd, .. } => Some(*rd),
            Rv64i::Sllw { rd, .. } => Some(*rd),
            Rv64i::Srlw { rd, .. } => Some(*rd),
            Rv64i::Sraw { rd, .. } => Some(*rd),
        }
    }

    fn src_mem_addr(&self) -> Option<u64> {
        match self {
            Rv64i::Lwu { rs1, imm, .. } => {
                Some(rs1.wrapping_add(*imm))
            },
            Rv64i::Ld { rs1, imm, .. } => {
                Some(rs1.wrapping_add(*imm))
            },
            _ => None
        }
    }

    fn dst_mem_addr(&self) -> Option<u64> {
        match self {
            Rv64i::Sd { rs1, imm, .. } => {
                Some(rs1.wrapping_add(*imm))
            },
            _ => None
        }
    }

    fn is_ld(&self) -> bool {
        match self {
            Rv64i::Lwu { .. } |
            Rv64i::Ld { .. } => true,
            _ => false
        }
    }

    fn is_st(&self) -> bool {
        match self {
            Rv64i::Sd { .. } => true,
            _ => false
        }
    }

    fn is_br(&self) -> bool {
        false
    }

    fn is_jmp(&self) -> bool {
        false
    }
}

impl Display for Rv32i {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Rv32i::Lui { rd, imm } => write!(f, "lui rd={}, imm={}", rd, imm),
            Rv32i::Auipc { rd, imm } => write!(f, "auipc rd={}, imm={}", rd, imm),
            Rv32i::Jal { rd, imm } => write!(f, "jal rd={}, offset={}", rd, imm),
            Rv32i::Jalr { rd, rs1, imm } => write!(f, "jalr rd={}, offset(rs1)={}({})", rd, imm, rs1),
            Rv32i::Beq { rs1, rs2, imm } => write!(f, "beq rs1={}, rs2={}, offset={}", rs1, rs2, imm),
            Rv32i::Bne { rs1, rs2, imm } => write!(f, "bne rs1={}, rs2={}, offset={}", rs1, rs2, imm),
            Rv32i::Blt { rs1, rs2, imm } => write!(f, "blt rs1={}, rs2={}, offset={}", rs1, rs2, imm),
            Rv32i::Bge { rs1, rs2, imm } => write!(f, "bge rs1={}, rs2={}, offset={}", rs1, rs2, imm),
            Rv32i::Bltu { rs1, rs2, imm } => write!(f, "bltu rs1={}, rs2={}, offset={}", rs1, rs2, imm),
            Rv32i::Bgeu { rs1, rs2, imm } => write!(f, "bgeu rs1={}, rs2={}, offset={}", rs1, rs2, imm),
            Rv32i::Lb { rd, rs1, imm } => write!(f, "lb rd={}, offset(rs1)={}({})", rd, imm, rs1),
            Rv32i::Lh { rd, rs1, imm } => write!(f, "lh rd={}, offset(rs1)={}({})", rd, imm, rs1),
            Rv32i::Lw { rd, rs1, imm } => write!(f, "lw rd={}, offset(rs1)={}({})", rd, imm, rs1),
            Rv32i::Lbu { rd, rs1, imm } => write!(f, "lbu rd={}, offset(rs1)={}({})", rd, imm, rs1),
            Rv32i::Lhu { rd, rs1, imm } => write!(f, "lhu rd={}, offset(rs1)={}({})", rd, imm, rs1),
            Rv32i::Sb { rs1, rs2, imm } => write!(f, "sb rs2={}, offset(rs1)={}({})", rs2, imm, rs1),
            Rv32i::Sh { rs1, rs2, imm } => write!(f, "sh rs2={}, offset(rs1)={}({})", rs2, imm, rs1),
            Rv32i::Sw { rs1, rs2, imm } => write!(f, "sw rs2={}, offset(rs1)={}({})", rs2, imm, rs1),
            Rv32i::Addi { rd, rs1, imm } => write!(f, "addi rd={}, rs1={}, imm={}", rd, rs1, imm),
            Rv32i::Slti { rd, rs1, imm } => write!(f, "slti rd={}, rs1={}, imm={}", rd, rs1, imm),
            Rv32i::Sltiu { rd, rs1, imm } => write!(f, "sltiu rd={}, rs1={}, imm={}", rd, rs1, imm),
            Rv32i::Xori { rd, rs1, imm } => write!(f, "xori rd={}, rs1={}, imm={}", rd, rs1, imm),
            Rv32i::Ori { rd, rs1, imm } => write!(f, "ori rd={}, rs1={}, imm={}", rd, rs1, imm),
            Rv32i::Andi { rd, rs1, imm } => write!(f, "andi rd={}, rs1={}, imm={}", rd, rs1, imm),
            Rv32i::Slli { rd, rs1, shamt } => write!(f, "slli rd={}, rs1={}, shamt={}", rd, rs1, shamt),
            Rv32i::Srli { rd, rs1, shamt } => write!(f, "srli rd={}, rs1={}, shamt={}", rd, rs1, shamt),
            Rv32i::Srai { rd, rs1, shamt } => write!(f, "srai rd={}, rs1={}, shamt={}", rd, rs1, shamt),
            Rv32i::Add { rd, rs1, rs2 } => write!(f, "add rd={}, rs1={}, rs2={}", rd, rs1, rs2),
            Rv32i::Sub { rd, rs1, rs2 } => write!(f, "sub rd={}, rs1={}, rs2={}", rd, rs1, rs2),
            Rv32i::Sll { rd, rs1, rs2 } => write!(f, "sll rd={}, rs1={}, rs2={}", rd, rs1, rs2),
            Rv32i::Slt { rd, rs1, rs2 } => write!(f, "slt rd={}, rs1={}, rs2={}", rd, rs1, rs2),
            Rv32i::Sltu { rd, rs1, rs2 } => write!(f, "sltu rd={}, rs1={}, rs2={}", rd, rs1, rs2),
            Rv32i::Xor { rd, rs1, rs2 } => write!(f, "xor rd={}, rs1={}, rs2={}", rd, rs1, rs2),
            Rv32i::Srl { rd, rs1, rs2 } => write!(f, "srl rd={}, rs1={}, rs2={}", rd, rs1, rs2),
            Rv32i::Sra { rd, rs1, rs2 } => write!(f, "sra rd={}, rs1={}, rs2={}", rd, rs1, rs2),
            Rv32i::Or { rd, rs1, rs2 } => write!(f, "or rd={}, rs1={}, rs2={}", rd, rs1, rs2),
            Rv32i::And { rd, rs1, rs2 } => write!(f, "and rd={}, rs1={}, rs2={}", rd, rs1, rs2),
        }
    }
}

impl Display for Rv64i {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Rv64i::Lwu { rd, rs1, imm } => write!(f, "lwu rd={}, offset(rs1)={}({})", rd, imm, rs1),
            Rv64i::Ld { rd, rs1, imm } => write!(f, "ld rd={}, offset(rs1)={}({})", rd, imm, rs1),
            Rv64i::Sd { rs1, rs2, imm } => write!(f, "sd rs2={}, offset(rs1)={}({})", rs2, imm, rs1),
            Rv64i::Addiw { rd, rs1, imm } => write!(f, "add rd={}, rs1={}, imm={}", rd, rs1, imm),
            Rv64i::Slliw { rd, rs1, shamt } => write!(f, "add rd={}, rs1={}, shamt={}", rd, rs1, shamt),
            Rv64i::Srliw { rd, rs1, shamt } => write!(f, "add rd={}, rs1={}, shamt={}", rd, rs1, shamt),
            Rv64i::Sraiw { rd, rs1, shamt } => write!(f, "add rd={}, rs1={}, shamt={}", rd, rs1, shamt),
            Rv64i::Addw { rd, rs1, rs2 } => write!(f, "addw rd={}, rs1={}, rs2={}", rd, rs1, rs2),
            Rv64i::Subw { rd, rs1, rs2 } => write!(f, "subw rd={}, rs1={}, rs2={}", rd, rs1, rs2),
            Rv64i::Sllw { rd, rs1, rs2 } => write!(f, "sllw rd={}, rs1={}, rs2={}", rd, rs1, rs2),
            Rv64i::Srlw { rd, rs1, rs2 } => write!(f, "srlw rd={}, rs1={}, rs2={}", rd, rs1, rs2),
            Rv64i::Sraw { rd, rs1, rs2 } => write!(f, "sraw rd={}, rs1={}, rs2={}", rd, rs1, rs2),
        }
    }
}

pub fn opcode(ins: u32) -> u32 {
    ins & 0x7f
}

pub fn rd(ins: u32) -> usize {
    ((ins >> 7) & 0b1_1111) as usize
}

pub fn rs1(ins: u32) -> usize {
    ((ins >> 15) & 0b1_1111) as usize
}

pub fn rs2(ins: u32) -> usize {
    ((ins >> 20) & 0b1_1111) as usize
}

pub fn funct3(ins: u32) -> u32 {
    (ins >> 12) & 0b111
}

pub fn funct7(ins: u32) -> u32 {
    ins >> 25
}

pub fn i_imm(ins: u32) -> u64 {
    ((((ins & 0xfff00000) as i32) as i64) >> 20) as u64
}

pub fn s_imm(ins: u32) -> u64 {
    (((ins & 0xfe000000) as i32 as i64 >> 20) as u64) | ((ins as u64 >> 7) & 0x1f)
}

pub fn u_imm(ins: u32) -> u64 {
    (ins & 0xfffff000) as i32 as i64 as u64
}

pub fn b_imm(ins: u32) -> u64 {
    (((ins & 0x80000000) as i32 as i64 >> 19) as u64)
        | ((ins as u64 & 0x80) << 4) // imm[11]
        | ((ins as u64 >> 20) & 0x7e0) // imm[10:5]
        | ((ins as u64 >> 7) & 0x1e)// imm[4:1]
}

pub fn j_imm(ins: u32) -> u64 {
    (((ins & 0x80000000) as i32 as i64 >> 11) as u64) // imm[20]
        | (ins as u64 & 0xff000) // imm[19:12]
        | ((ins as u64 >> 9) & 0x800) // imm[11]
        | ((ins as u64 >> 20) & 0x7fe)
}

pub fn print_register_table(regs: &[u64; 32]) {
    let mut builder = Builder::new();
        builder.set_header(["Register", "Decimal", "Hex"]);
        regs
            .iter()
            .enumerate()
            .map(|(i, r)| [
                format!("{}", RVABI[i]),
                format!("{}", r),
                format!("{:#01x}", r),
                //format!("{:#01b}", r),
            ]).for_each(|line| {
                builder.push_record(line);
            });
        let table = builder.build()
            .with(Style::ascii_rounded())
            .to_string();
        println!("{}", table);
}

#[cfg(test)]
mod tests {
    use std::{process::Command, fs::File, io::{Write, Read}};
    use crate::{isa::{Rv32i, Extension}, bus::Bus};

    type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

    #[allow(dead_code)]
    fn clang_compile_c(c_src: &str) -> Result<()> {
        let cc = "clang";
        let out = Command::new(cc).arg("-S")
            .arg(c_src)
            .arg("-nostdlib")
            .arg("-march=rv64i")
            .arg("-mabi=lp64")
            .arg("--target=riscv64")
            .arg("-mno-relax")
            .output()?;
        if out.status.success() {
            Ok(())
        } else {
            let err = String::from_utf8_lossy(&out.stderr);
            Err(format!("C compilation failed: {}", err).into())
        }
    }

    fn clang_compile_asm(asm_path: &str, ex_path: &str) -> Result<()> {
        let cc = "clang";
        let out = Command::new(cc).arg("-Wl,-Ttext=0x0")
            .arg("-nostdlib")
            .arg("-march=rv64i")
            .arg("-mabi=lp64")
            .arg("--target=riscv64")
            .arg("-mno-relax")
            .arg("-o")
            .arg(ex_path)
            .arg(asm_path)
            .output()?;
        if out.status.success() {
            Ok(())
        } else {
            let err = String::from_utf8_lossy(&out.stderr);
            Err(format!("ASM compilation failed: {}", err).into())
        }
    } 

    fn llvm_copy_obj(ex_path: &str, bin_path: &str) -> Result<()> {
        let objcopy = "llvm-objcopy";
        let out = Command::new(objcopy).arg("-O")
            .arg("binary")
            .arg(ex_path)
            .arg(bin_path)
            .output()?;
        if out.status.success() {
            Ok(())
        } else {
            let err = String::from_utf8_lossy(&out.stderr);
            Err(format!("LLVM copy obj failed: {}", err).into())
        }
    }

    fn asm(name: &str, code: &str) -> Result<Vec<u8>> {
        let asm_path = "./target/test/".to_string() + name + ".s";
        let ex_path = "./target/test/".to_string() + name;
        let bin_path = "./target/test/".to_string() + name + ".bin";
        std::fs::create_dir_all("./target/test/")?;
        let mut asm_file = File::create(&asm_path)?;
        asm_file.write(&code.as_bytes())?;
        clang_compile_asm(&asm_path, &ex_path)?;
        llvm_copy_obj(&ex_path, &bin_path)?;
        let mut file_bin = File::open(bin_path)?;
        let mut code = Vec::new();
        file_bin.read_to_end(&mut code)?;
        Ok(code)
    }

    fn if32(bin: &[u8], i: usize) -> Option<u32> {
        assert!(bin.len() >= (i * 4) + 4);
        bin.iter().skip(i * 4).enumerate()
            .map(|(i, x)| (*x as u32) << (i * 8))
            .reduce(|a, b| a | b)
    }

    #[test]
    fn addi() {
        let addi = asm("addi", "addi x31, x0, 42");
        assert!(addi.is_ok(), "Failed to compile: {}", addi.err().unwrap());
        let ins = if32(&addi.unwrap(), 0);
        assert!(ins.is_some(), "Failed to find instruction at index {}", 0);
        let t = Rv32i::id(ins.unwrap());
        assert!(t.is_ok(), "Failed to parse instruction: {:?}", t.err().unwrap());
        assert_eq!(t.as_ref().unwrap(), &Rv32i::Addi { rd: 31, rs1: 0, imm: 42 });
        let mut regs = [0_u64; 32];
        regs[31] = 5;
        let t = t.unwrap().ex(&regs);
        assert_eq!(&t, &Rv32i::Addi { rd: 31, rs1: 0, imm: 42 });
        let res = t.wr(0, &mut regs, &mut Bus::new(vec![]));
        assert!(res.is_ok(), "Execution failed: {:?}", res.err().unwrap());
        let res = res.unwrap();
        assert_eq!(res, 4);
        assert_eq!(regs[31], 42);
    }
}