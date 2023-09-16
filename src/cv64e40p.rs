use tabled::{builder::Builder, settings::Style};

use crate::{soc::{Neumann, SoC, Isa}, stats::Stats};

pub struct Cv64e40p {
    soc: Neumann,
    ifetch: Option<u32>,
    idecode: Option<u32>,
    ex: Option<(usize, u64)>,
    pc: Option<u64>,
    ld_ins1: Option<u32>,
    ld_ins2: Option<u32>,
}

impl Cv64e40p {
    pub fn new(bin: Vec<u8>) -> Self {
        Self {
            soc: Neumann::new(bin, 0x8000_0000),
            ifetch: None,
            idecode: None,
            ex: None,
            pc: None,
            ld_ins1: None,
            ld_ins2: None
        }
    }
}

impl Isa for Cv64e40p {}

impl SoC for Cv64e40p {
    fn execute(&mut self) -> Stats {
        let mut stats = Stats::new();
        while self.soc.pc < self.soc.mem.size() as u64 + 16 {
            if let Some((rd, x)) = self.ex {
                self.soc.regfile[rd] = x;
                self.ex = None;
            }
            if let Some(ld_ins2) = self.ld_ins2 {
                self.soc.regfile[Self::rd(ld_ins2)] = self.ld(ld_ins2);
                self.ld_ins2 = None;
            }
            if let (Some(ld_ins1), None) = (self.ld_ins1, self.ld_ins2) {
                self.ld_ins2 = Some(ld_ins1);
                self.ld_ins1 = None;
            }
            if let Some(pc) = self.pc {
                self.soc.pc = pc;
                // flush pipeline:
                self.idecode = None;
                self.ifetch = None;
                self.pc = None;
            }
            if let (Some(idecode), None, None) = (self.idecode, self.ex, self.ld_ins1) {
                if Self::is_jmp(idecode) {
                    let (pc, rd) = self.jmp(idecode);
                    self.pc = Some(pc);
                    self.ex = Some((Self::rd(idecode), rd));
                    self.idecode = None;
                } else if Self::is_br(idecode) {
                    let pc = self.br(idecode);
                    self.pc = pc;
                    self.idecode = None;
                } else if Self::is_ld(idecode) {
                    self.ld_ins1 = Some(idecode);
                    self.idecode = None;
                    stats.mem_cycles += 1;
                } else if Self::is_st(idecode) {
                    self.st(idecode);
                    self.idecode = None;
                    stats.mem_cycles += 1;
                } else {
                    let stall = self.ld_ins2
                        .map(|ld| self.src_regs(idecode).contains(&Self::rd(ld)))
                        .unwrap_or(false);
                    if !stall {
                        let result = self.alu(idecode);
                        self.ex = Some((Self::rd(idecode), result));
                        self.idecode = None;
                        stats.exec_cycles += 1;
                    } else {
                        stats.stall_cycles += 1;
                    }
                }
            }
            if let (Some(ifetch), None) = (self.ifetch, self.idecode) {
                self.idecode = Some(ifetch);
                self.ifetch = None;
            }
            if self.ifetch.is_none() {
                self.ifetch = Some(self.soc.mem.if32(self.soc.pc as u64));
            }
            self.soc.pc += 4;
            stats.cycles += 1;
        }
        stats
    }

    fn pc(&self) -> u64 {
        self.soc.pc
    }

    fn regfile(&self) -> &[u64; 32] {
        &self.soc.regfile
    }

    fn mem_mut(&mut self) -> &mut crate::soc::Mem {
        &mut self.soc.mem
    }

    fn mem(&self) -> &crate::soc::Mem {
        &self.soc.mem
    }
}
