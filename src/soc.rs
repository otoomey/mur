use tabled::{builder::Builder, settings::Style};

use crate::stats::Stats;

pub struct Neumann {
    pub regfile: [u64; 32],
    pub pc: u64,
    pub mem: Mem
}

pub struct Mem {
    mem: Vec<u8>,
    offset: u64
}

impl Neumann {
    pub fn new(mem: Vec<u8>, mem_offset: u64) -> Self {
        Neumann { 
            regfile: [0; 32], 
            pc: 0, 
            mem: Mem::new(mem, mem_offset)
        }
    }
}

pub trait Isa {
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
}

pub trait SoC: Isa {
    fn execute(&mut self) -> Stats;
    fn pc(&self) -> u64;
    fn regfile(&self) -> &[u64; 32];
    fn mem_mut(&mut self) -> &mut Mem;
    fn mem(&self) -> &Mem;

    fn ireg(&self, reg: usize) -> i64 {
        self.regfile()[reg] as i64
    }

    fn ureg(&self, reg: usize) -> u64 {
        self.regfile()[reg]
    }

    fn ld(&self, ins: u32) -> u64 {
        let imm = Self::i_imm(ins) as i64;
        let addr = self.ireg(Self::rs1(ins)).wrapping_add(imm) as u64;
        let funct3 = Self::funct3(ins);
        match funct3 {
            0x0 => {
                // lb
                println!("lb");
                self.mem().ld8(addr) as i8 as i64 as u64
            }
            0x1 => {
                // lh
                println!("lh");
                self.mem().ld16(addr) as i16 as i64 as u64
            }
            0x2 => {
                // lw
                println!("lw");
                self.mem().ld32(addr) as i32 as i64 as u64
            }
            0x3 => {
                // ld
                println!("ld");
                self.mem().ld64(addr)
            }
            0x4 => {
                // lbu
                println!("lbu");
                self.mem().ld8(addr) as u64
            }
            0x5 => {
                // lhu
                println!("lhu");
                self.mem().ld16(addr) as u64
            }
            0x6 => {
                // lwu
                println!("lwu");
                self.mem().ld32(addr) as u64
            }
            _ => panic!("Unknown ld op {}", funct3)
        }
    }

    fn st(&mut self, ins: u32) {
        let imm = Self::s_imm(ins) as i64;
        let addr = self.ireg(Self::rs1(ins)).wrapping_add(imm) as u64;
        let funct3 = Self::funct3(ins);
        let value = self.ureg(Self::rs2(ins));
        println!("st");
        match funct3 {
            0x0 => self.mem_mut().st8(addr, value as u8),  // sb
            0x1 => self.mem_mut().st16(addr, value as u16), // sh
            0x2 => self.mem_mut().st32(addr, value as u32), // sw
            0x3 => self.mem_mut().st64(addr, value), // sd
            _ => panic!("Unknown st op {}", funct3)
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
            _ => panic!("Unknown instruction {}", Self::opcode(ins))
        }
    }

    fn jmp(&self, ins: u32) -> (u64, u64) {
        match Self::opcode(ins) {
            0b1101111 => { // jal
                println!("jal");
                let next = ((self.pc() as i64) + Self::j_imm(ins) as i64) & 0xff_ff_ff_fe;
                let rd = self.pc() + 4;
                (next as u64, rd)
            }
            0b1100111 => { // jalr
                println!("jalr");
                let next = (self.ireg(Self::rs1(ins)) + Self::i_imm(ins) as i64) & 0xff_ff_ff_fe;
                let rd = self.pc() + 4;
                (next as u64, rd)
            }
            _ => panic!("Not a valid jump opcode {}", Self::opcode(ins))
        }
    }

    fn br(&self, ins: u32) -> Option<u64> {
        match (Self::funct3(ins), Self::opcode(ins)) {
            (0b000, 0b1100011) => { // beq
                println!("beq");
                if self.ureg(Self::rs1(ins)) == self.ureg(Self::rs2(ins)) {
                    Some((self.pc() as i64 + Self::b_imm(ins) as i64) as u64)
                } else {
                    None
                }
            }
            (0b001, 0b1100011) => { // bne
                println!("bne");
                if self.ureg(Self::rs1(ins)) != self.ureg(Self::rs2(ins)) {
                    Some((self.pc() as i64 + Self::b_imm(ins) as i64) as u64)
                } else {
                    None
                }
            }
            (0b100, 0b1100011) => { // blt
                println!("blt");
                if self.ireg(Self::rs1(ins)) < self.ireg(Self::rs2(ins)) {
                    Some((self.pc() as i64 + Self::b_imm(ins) as i64) as u64)
                } else {
                    None
                }
            }
            (0b101, 0b1100011) => { // bge
                println!("bge");
                if self.ireg(Self::rs1(ins)) >= self.ireg(Self::rs2(ins)) {
                    Some((self.pc() as i64 + Self::b_imm(ins) as i64) as u64)
                } else {
                    None
                }
            }
            (0b110, 0b1100011) => { // bltu
                println!("bltu");
                if self.ureg(Self::rs1(ins)) < self.ureg(Self::rs2(ins)) {
                    Some((self.pc() as i64 + Self::b_imm(ins) as i64) as u64)
                } else {
                    None
                }
            }
            (0b111, 0b1100011) => { // bgeu
                println!("bgeu");
                if self.ureg(Self::rs1(ins)) >= self.ureg(Self::rs2(ins)) {
                    Some((self.pc() as i64 + Self::b_imm(ins) as i64) as u64)
                } else {
                    None
                }
            }
            _ => panic!("Unknown instruction {}", ins)
        }
    }

    fn alu(&self, ins: u32) -> u64 {
        match (Self::funct7(ins), Self::funct3(ins), Self::opcode(ins)) {
            (_, _, 0b0110111) => { // lui
                println!("lui");
                ((Self::u_imm(ins) as i64) << 12) as u64
            }
            (_, _, 0b0010111) => { // auipc
                println!("auipc");
                let val = (Self::u_imm(ins) as i64) << 12;
                (self.pc() as i64).wrapping_add(val) as u64
            }
            (_, 0b000, 0b0010011) => { // addi
                println!("addi");
                self.ireg(Self::rs1(ins)).wrapping_add(Self::i_imm(ins) as i64) as u64
            },
            (_, 0b010, 0b0010011) => { // slti
                println!("slti");
                let cond = self.ireg(Self::rs1(ins)) < (Self::i_imm(ins) as i64);
                if cond { 1 } else { 0 }
            }
            (_, 0b011, 0b0010011) => { // sltiu
                println!("sltiu");
                let cond = self.ureg(Self::rs1(ins)) < (Self::i_imm(ins) as u64);
                if cond { 1 } else { 0 }
            }
            (_, 0b100, 0b0010011) => { // xori
                println!("xori");
                self.ureg(Self::rs1(ins)) ^ (Self::i_imm(ins) as u64)
            }
            (_, 0b110, 0b0010011) => { // ori
                println!("ori");
                self.ureg(Self::rs1(ins)) | (Self::i_imm(ins) as u64)
            }
            (_, 0b111, 0b0010011) => { // andi
                println!("andi");
                self.ureg(Self::rs1(ins)) & (Self::i_imm(ins) as u64)
            }
            (_, 0b001, 0b0010011) => { // slli
                println!("slli");
                self.ureg(Self::rs1(ins)) << (Self::i_imm(ins) as u64)
            }
            (0b0000000, 0b101, 0b0010011) => { // srli
                println!("srli");
                self.ureg(Self::rs1(ins)) >> (Self::i_imm(ins) as u64)
            }
            (0b1000000, 0b101, 0b0010011) => { // srai
                println!("srai");
                let shift = ((Self::i_imm(ins) as u32) << 1) >> 1;
                (self.ireg(Self::rs1(ins)) >> (shift as i64)) as u64
            }
            (0b0000000, 0b000, 0b0110011,) => { // add
                println!("add");
                self.ireg(Self::rs1(ins)).wrapping_add(self.ireg(Self::rs2(ins))) as u64
            }
            (0b0100000, 0b000, 0b0110011) => { // sub
                println!("sub");
                self.ireg(Self::rs1(ins)).wrapping_sub(self.ireg(Self::rs2(ins))) as u64
            }
            (0b0000000, 0b001, 0b0110011 ) => { // sll
                println!("sll");
                let shift = self.ureg(Self::rs2(ins)) & 0b1_1111;
                self.ureg(Self::rs1(ins)) << shift
            }
            (0b0000000, 0b010, 0b0110011 ) => { // slt
                println!("slt");
                let cond = self.ireg(Self::rs1(ins)) < self.ireg(Self::rs2(ins));
                if cond { 1 } else { 0 }
            }
            (0b0000000, 0b011, 0b0110011 ) => { // sltu
                println!("sltu");
                let cond = self.ureg(Self::rs1(ins)) < self.ureg(Self::rs2(ins));
                if cond { 1 } else { 0 }
            }
            (0b0000000, 0b100, 0b0110011) => { // xori
                println!("xori");
                self.ureg(Self::rs1(ins)) ^ self.ureg(Self::rs2(ins))
            }
            (0b0000000, 0b101, 0b0110011) => { // srl
                println!("srl");
                let shift = self.ureg(Self::rs2(ins)) & 0b1_1111;
                self.ureg(Self::rs1(ins)) >> shift
            }
            (0b0100000, 0b101, 0b0110011) => { // sra
                println!("sra");
                let shift = self.ureg(Self::rs2(ins)) & 0b1_1111;
                (self.ireg(Self::rs1(ins)) >> (shift as i64)) as u64
            }
            (0b0000000, 0b110, 0b0110011) => { // or
                println!("or");
                self.ureg(Self::rs1(ins)) | self.ureg(Self::rs2(ins))
            }
            (0b0000000, 0b111, 0b0110011) => { // and
                println!("and");
                self.ureg(Self::rs1(ins)) & self.ureg(Self::rs2(ins))
            }
            _ => panic!("Unknown instruction {:#032b}", ins)
        }
    }

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
        println!("Cv32e40p [pc: {}] Register File:", self.pc());
        println!("{}", table);
    }
}

impl Mem {
    pub fn new(mem: Vec<u8>, offset: u64) -> Self {
        Self { mem, offset }
    }

    pub fn if32(&self, pc: u64) -> u32 {
        let index = pc as usize;
        if index + 3 >= self.mem.len() {
            return 0b0000000_00000_00000_000_00000_0010011;
        }
        (self.mem[index] as u32)
            | ((self.mem[index + 1] as u32) << 8)
            | ((self.mem[index + 2] as u32) << 16)
            | ((self.mem[index + 3] as u32) << 24)
    }

    pub fn ld8(&self, addr: u64) -> u8 {
        let index = (addr - self.offset) as usize;
        self.mem[index]
    }

    pub fn ld16(&self, addr: u64) -> u16 {
        let index = (addr - self.offset) as usize;
        return (self.mem[index] as u16)
            | ((self.mem[index + 1] as u16) << 8);
    }

    pub fn ld32(&self, addr: u64) -> u32 {
        let index = (addr - self.offset) as usize;
        return (self.mem[index] as u32)
            | ((self.mem[index + 1] as u32) << 8)
            | ((self.mem[index + 2] as u32) << 16)
            | ((self.mem[index + 3] as u32) << 24);
    }

    pub fn ld64(&self, addr: u64) -> u64 {
        let index = (addr - self.offset) as usize;
        return (self.mem[index] as u64)
            | ((self.mem[index + 1] as u64) << 8)
            | ((self.mem[index + 2] as u64) << 16)
            | ((self.mem[index + 3] as u64) << 24)
            | ((self.mem[index + 4] as u64) << 32)
            | ((self.mem[index + 5] as u64) << 40)
            | ((self.mem[index + 6] as u64) << 48)
            | ((self.mem[index + 7] as u64) << 56);
    }

    pub fn st8(&mut self, addr: u64, value: u8) {
        let index = (addr - self.offset) as usize;
        self.mem[index] = value;
    }

    pub fn st16(&mut self, addr: u64, value: u16) {
        let index = (addr - self.offset) as usize;
        self.mem[index] = (value & 0xff) as u8;
        self.mem[index + 1] = ((value >> 8) & 0xff) as u8;
    }

    pub fn st32(&mut self, addr: u64, value: u32) {
        let index = (addr - self.offset) as usize;
        self.mem[index] = (value & 0xff) as u8;
        self.mem[index + 1] = ((value >> 8) & 0xff) as u8;
        self.mem[index + 2] = ((value >> 16) & 0xff) as u8;
        self.mem[index + 3] = ((value >> 24) & 0xff) as u8;
    }

    pub fn st64(&mut self, addr: u64, value: u64) {
        let index = (addr - self.offset) as usize;
        self.mem[index] = (value & 0xff) as u8;
        self.mem[index + 1] = ((value >> 8) & 0xff) as u8;
        self.mem[index + 2] = ((value >> 16) & 0xff) as u8;
        self.mem[index + 3] = ((value >> 24) & 0xff) as u8;
        self.mem[index + 4] = ((value >> 32) & 0xff) as u8;
        self.mem[index + 5] = ((value >> 40) & 0xff) as u8;
        self.mem[index + 6] = ((value >> 48) & 0xff) as u8;
        self.mem[index + 7] = ((value >> 56) & 0xff) as u8;
    }

    pub fn size(&self) -> usize {
        self.mem.len()
    }
}