use std::vec;

use tabled::{builder::Builder, settings::Style};

use crate::{stats::Stats, mem::{B8, B16, B32, B64}, bus::Bus, exception::Exception, csr::Csr};

pub struct Exit {
    pub stats: Stats,
    pub ex: Exception
}

pub trait SoC {
    fn execute(&mut self) -> Result<(), Exit>;
    fn pc(&self) -> u64;
    fn regfile(&self) -> &[u64; 32];
    fn bus(&self) -> &Bus;
    fn bus_mut(&mut self) -> &mut Bus;
    fn csr(&self) -> &Csr;
    fn csr_mut(&mut self) -> &mut Csr;

    fn dump_registers(&self) {
        let mut builder = Builder::new();
        builder.set_header(["Register", "Decimal", "Hex", "Binary"]);
        self.regfile()
            .iter()
            .enumerate()
            .map(|(i, r)| [
                format!("x{}", i),
                format!("{}", r),
                format!("{:#01x}", r),
                format!("{:#01b}", r),
            ]).for_each(|line| {
                builder.push_record(line);
            });
        let table = builder.build()
            .with(Style::ascii_rounded())
            .to_string();
        println!("{}", table);
    }
}

pub trait Isa: SoC {
    fn opcode(ins: u32) -> u32 {
        ins & 0x7f
    }

    fn is_ld(ins: u32) -> bool {
        Self::opcode(ins) == 0x03
    }

    fn is_st(ins: u32) -> bool {
        Self::opcode(ins) == 0b0100011
    }

    fn is_br(ins: u32) -> bool {
        Self::opcode(ins) == 0b1100011
    }

    fn is_jmp(ins: u32) -> bool {
        Self::opcode(ins) == 0b1101111 || Self::opcode(ins) == 0b1100111
    }
    
    fn is_zicsr(ins: u32) -> bool {
        Self::opcode(ins) == 0b1110011
    }

    fn is_alu_op(ins: u32) -> bool {
        Self::opcode(ins) == 0b0010011 || Self::opcode(ins) == 0b0110011
    }

    fn rd(ins: u32) -> usize {
        ((ins >> 7) & 0b1_1111) as usize
    }

    fn rs1(ins: u32) -> usize {
        ((ins >> 15) & 0b1_1111) as usize
    }

    fn rs2(ins: u32) -> usize {
        ((ins >> 20) & 0b1_1111) as usize
    }

    fn funct3(ins: u32) -> u32 {
        (ins >> 12) & 0b111
    }

    fn funct7(ins: u32) -> u32 {
        ins >> 25
    }

    fn i_imm(ins: u32) -> i32 {
        (ins as i32) >> 20
    }

    fn s_imm(ins: u32) -> i32 {
        let lower = ((ins & 0b0000000_00000_00000_000_11111_0000000) > 7) as i32;
        let upper = ((ins & 0b1111111_00000_00000_000_00000_0000000) as i32) >> 20;
        lower | upper
    }

    fn u_imm(ins: u32) -> i32 {
        (ins as i32) >> 12
    }

    fn b_imm(ins: u32) -> i32 {
        let lower = (ins & 0b0000000_00000_00000_000_11110_0000000) >> 7;
        let upper = (ins & 0b0111111_00000_00000_000_00000_0000000) >> 20;
        let sign  =  ins & 0b1000000_00000_00000_000_00000_0000000;
        let sgnif = (ins & 0b0000000_00000_00000_000_00001_0000000) << 4;
        (lower | upper | sign | sgnif) as i32
    }

    fn j_imm(ins: u32) -> i32 {
        let lower = (ins & 0b0111111_11110_00000_000_00000_0000000) >> 20;
        let middl = (ins & 0b0000000_00001_00000_000_00000_0000000) >> 9;
        let upper =  ins & 0b0000000_00000_11111_111_00000_0000000;
        let sign  =  ins & 0b1000000_00000_00000_000_00000_0000000;
        (lower | upper | middl | sign) as i32
    }

    fn ireg(&self, reg: usize) -> i64 {
        self.regfile()[reg] as i64
    }

    fn ureg(&self, reg: usize) -> u64 {
        self.regfile()[reg]
    }

    fn ld(&self, ins: u32) -> Result<u64, Exception> {
        let imm = Self::i_imm(ins) as i64;
        let addr = self.ireg(Self::rs1(ins)).wrapping_add(imm) as u64;
        let funct3 = Self::funct3(ins);
        match funct3 {
            0x0 => {
                // lb
                println!("lb");
                Ok(self.bus().load(addr, B8)? as i8 as i64 as u64)
            }
            0x1 => {
                // lh
                println!("lh");
                Ok(self.bus().load(addr, B16)? as i16 as i64 as u64)
            }
            0x2 => {
                // lw
                println!("lw");
                Ok(self.bus().load(addr, B32)? as i32 as i64 as u64)
            }
            0x3 => {
                // ld
                println!("ld");
                self.bus().load(addr, B64)
            }
            0x4 => {
                // lbu
                println!("lbu");
                self.bus().load(addr, B8)
            }
            0x5 => {
                // lhu
                println!("lhu");
                self.bus().load(addr, B16)
            }
            0x6 => {
                // lwu
                println!("lwu");
                self.bus().load(addr, B32)
            }
            _ => Err(Exception::IllegalInstruction(ins as u64))
        }
    }

    fn st(&mut self, ins: u32) -> Result<(), Exception> {
        let imm = Self::s_imm(ins) as i64;
        let addr = self.ireg(Self::rs1(ins)).wrapping_add(imm) as u64;
        let funct3 = Self::funct3(ins);
        let value = self.ureg(Self::rs2(ins));
        println!("st {} {}", addr, value);
        match funct3 {
            0x0 => self.bus_mut().store(addr, B8, value),  // sb
            0x1 => self.bus_mut().store(addr, B16, value), // sh
            0x2 => self.bus_mut().store(addr, B32, value), // sw
            0x3 => self.bus_mut().store(addr, B64, value), // sd
            _ => Err(Exception::IllegalInstruction(ins as u64))
        }
    }

    fn src_regs(&self, ins: u32) -> Vec<usize> {
        match Self::opcode(ins) {
            0b1100111 => vec![Self::rs1(ins)], // jalr
            0b1100011 => vec![Self::rs1(ins), Self::rs2(ins)], // branch
            0b0000011 => vec![Self::rs1(ins)], // load
            0b0100011 => vec![Self::rs1(ins), Self::rs2(ins)], // store
            0b0010011 => vec![Self::rs1(ins)], // alu imm
            0b0110011 => vec![Self::rs1(ins), Self::rs2(ins)], // alu
            _ => vec![]
        }
    }

    fn jmp(&self, ins: u32) -> Result<(u64, u64), Exception> {
        match Self::opcode(ins) {
            0b1101111 => { // jal
                println!("jal");
                let next = ((self.pc() as i64) + Self::j_imm(ins) as i64) & 0xff_ff_ff_fe;
                let rd = self.pc() + 4;
                Ok((next as u64, rd))
            }
            0b1100111 => { // jalr
                println!("jalr");
                let next = (self.ireg(Self::rs1(ins)) + Self::i_imm(ins) as i64) & 0xff_ff_ff_fe;
                let rd = self.pc() + 4;
                Ok((next as u64, rd))
            }
            _ => Err(Exception::IllegalInstruction(ins as u64))
        }
    }

    fn br(&self, ins: u32) -> Result<Option<u64>, Exception> {
        match (Self::funct3(ins), Self::opcode(ins)) {
            (0b000, 0b1100011) => { // beq
                println!("beq");
                if self.ureg(Self::rs1(ins)) == self.ureg(Self::rs2(ins)) {
                    Ok(Some((self.pc() as i64 + Self::b_imm(ins) as i64) as u64))
                } else {
                    Ok(None)
                }
            }
            (0b001, 0b1100011) => { // bne
                println!("bne");
                if self.ureg(Self::rs1(ins)) != self.ureg(Self::rs2(ins)) {
                    Ok(Some((self.pc() as i64 + Self::b_imm(ins) as i64) as u64))
                } else {
                    Ok(None)
                }
            }
            (0b100, 0b1100011) => { // blt
                println!("blt");
                if self.ireg(Self::rs1(ins)) < self.ireg(Self::rs2(ins)) {
                    Ok(Some((self.pc() as i64 + Self::b_imm(ins) as i64) as u64))
                } else {
                    Ok(None)
                }
            }
            (0b101, 0b1100011) => { // bge
                println!("bge");
                if self.ireg(Self::rs1(ins)) >= self.ireg(Self::rs2(ins)) {
                    Ok(Some((self.pc() as i64 + Self::b_imm(ins) as i64) as u64))
                } else {
                    Ok(None)
                }
            }
            (0b110, 0b1100011) => { // bltu
                println!("bltu");
                if self.ureg(Self::rs1(ins)) < self.ureg(Self::rs2(ins)) {
                    Ok(Some((self.pc() as i64 + Self::b_imm(ins) as i64) as u64))
                } else {
                    Ok(None)
                }
            }
            (0b111, 0b1100011) => { // bgeu
                println!("bgeu");
                if self.ureg(Self::rs1(ins)) >= self.ureg(Self::rs2(ins)) {
                    Ok(Some((self.pc() as i64 + Self::b_imm(ins) as i64) as u64))
                } else {
                    Ok(None)
                }
            }
            _ => Err(Exception::IllegalInstruction(ins as u64))
        }
    }

    fn alu(&self, ins: u32) -> Result<u64, Exception> {
        match (Self::funct7(ins), Self::funct3(ins), Self::opcode(ins)) {
            (_, _, 0b0110111) => { // lui
                println!("lui");
                Ok(((Self::u_imm(ins) as i64) << 12) as u64)
            }
            (_, _, 0b0010111) => { // auipc
                println!("auipc");
                let val = (Self::u_imm(ins) as i64) << 12;
                Ok((self.pc() as i64).wrapping_add(val) as u64)
            }
            (_, 0b000, 0b0010011) => { // addi
                println!("addi {} {} {}", Self::rs1(ins), self.ireg(Self::rs1(ins)), Self::i_imm(ins) as i64);
                Ok(self.ireg(Self::rs1(ins)).wrapping_add(Self::i_imm(ins) as i64) as u64)
            },
            (_, 0b010, 0b0010011) => { // slti
                println!("slti");
                let cond = self.ireg(Self::rs1(ins)) < (Self::i_imm(ins) as i64);
                Ok(if cond { 1 } else { 0 })
            }
            (_, 0b011, 0b0010011) => { // sltiu
                println!("sltiu");
                let cond = self.ureg(Self::rs1(ins)) < (Self::i_imm(ins) as u64);
                Ok(if cond { 1 } else { 0 })
            }
            (_, 0b100, 0b0010011) => { // xori
                println!("xori");
                Ok(self.ureg(Self::rs1(ins)) ^ (Self::i_imm(ins) as u64))
            }
            (_, 0b110, 0b0010011) => { // ori
                println!("ori");
                Ok(self.ureg(Self::rs1(ins)) | (Self::i_imm(ins) as u64))
            }
            (_, 0b111, 0b0010011) => { // andi
                println!("andi");
                Ok(self.ureg(Self::rs1(ins)) & (Self::i_imm(ins) as u64))
            }
            (_, 0b001, 0b0010011) => { // slli
                println!("slli");
                Ok(self.ureg(Self::rs1(ins)) << (Self::i_imm(ins) as u64))
            }
            (0b0000000, 0b101, 0b0010011) => { // srli
                println!("srli");
                Ok(self.ureg(Self::rs1(ins)) >> (Self::i_imm(ins) as u64))
            }
            (0b1000000, 0b101, 0b0010011) => { // srai
                println!("srai");
                let shift = ((Self::i_imm(ins) as u32) << 1) >> 1;
                Ok((self.ireg(Self::rs1(ins)) >> (shift as i64)) as u64)
            }
            (0b0000000, 0b000, 0b0110011,) => { // add
                println!("add");
                Ok(self.ireg(Self::rs1(ins)).wrapping_add(self.ireg(Self::rs2(ins))) as u64)
            }
            (0b0100000, 0b000, 0b0110011) => { // sub
                println!("sub");
                Ok(self.ireg(Self::rs1(ins)).wrapping_sub(self.ireg(Self::rs2(ins))) as u64)
            }
            (0b0000000, 0b001, 0b0110011 ) => { // sll
                println!("sll");
                let shift = self.ureg(Self::rs2(ins)) & 0b1_1111;
                Ok(self.ureg(Self::rs1(ins)) << shift)
            }
            (0b0000000, 0b010, 0b0110011 ) => { // slt
                println!("slt");
                let cond = self.ireg(Self::rs1(ins)) < self.ireg(Self::rs2(ins));
                Ok(if cond { 1 } else { 0 })
            }
            (0b0000000, 0b011, 0b0110011 ) => { // sltu
                println!("sltu");
                let cond = self.ureg(Self::rs1(ins)) < self.ureg(Self::rs2(ins));
                Ok(if cond { 1 } else { 0 })
            }
            (0b0000000, 0b100, 0b0110011) => { // xori
                println!("xori");
                Ok(self.ureg(Self::rs1(ins)) ^ self.ureg(Self::rs2(ins)))
            }
            (0b0000000, 0b101, 0b0110011) => { // srl
                println!("srl");
                let shift = self.ureg(Self::rs2(ins)) & 0b1_1111;
                Ok(self.ureg(Self::rs1(ins)) >> shift)
            }
            (0b0100000, 0b101, 0b0110011) => { // sra
                println!("sra");
                let shift = self.ureg(Self::rs2(ins)) & 0b1_1111;
                Ok((self.ireg(Self::rs1(ins)) >> (shift as i64)) as u64)
            }
            (0b0000000, 0b110, 0b0110011) => { // or
                println!("or");
                Ok(self.ureg(Self::rs1(ins)) | self.ureg(Self::rs2(ins)))
            }
            (0b0000000, 0b111, 0b0110011) => { // and
                println!("and");
                Ok(self.ureg(Self::rs1(ins)) & self.ureg(Self::rs2(ins)))
            }
            _ => Err(Exception::IllegalInstruction(ins as u64))
        }
    }

    fn zicsr(&self, ins: u32) -> Result<(usize, u64, u64), Exception> {
        let funct3 = Self::funct3(ins);
        let csr = Self::i_imm(ins) as u32 as usize;
        let t = self.csr().load(csr);
        Ok((
            csr, // csr in question
            t, // rd
            match funct3 { // new csr value
            0b011 => { // csrrc
                Ok(t & !(self.ureg(Self::rs1(ins))))
            },
            0b111 => { // csrci
                Ok(t & !(Self::rs1(ins) as u64))
            }
            0b010 => { // csrrs
                Ok(t | self.ureg(Self::rs1(ins)))
            }
            0b110 => { // csrri
                Ok(t | (Self::rs1(ins) as u64))
            },
            0b001 => { // csrrw
                Ok(self.ureg(Self::rs1(ins)))
            },
            0b101 => { // csrwi
                Ok(Self::rs1(ins) as u64)
            }
            _ => Err(Exception::IllegalInstruction(ins as u64))
        }?))
    }
}

impl Exit {
    pub fn from_ex(stats: Stats, ex: Exception) -> Self {
        Self { stats, ex }
    }
}