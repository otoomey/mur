use std::fmt::Display;

use crate::{bus::{Bus, RAM_END, RAM_BASE}, stats::Stats, mem::B64, isa::{Rv32i, Extension, Rv64i}, exception::Exception};

/*
An out-of-order, infinite-fetch, infinite-issue single-stage processor
*/

struct HistItem {
    src_regs: Vec<u64>,
    dst_reg: Option<u64>,
    blocking: bool
}

pub struct KronosSoC {
    pub regs: [u64; 32],
    pub pc: u64,
    pub bus: Bus,
    pub stats: Stats,
    hist: Vec<HistItem>
}

type Result = std::result::Result<(), Exception>;

impl KronosSoC {
    pub fn new(bin: Vec<u8>) -> Self {
        let mut regs = [0_u64; 32];
        regs[2] = RAM_END;
        let pc = RAM_BASE;
        let bus = Bus::new(bin);
        let stats = Stats::new();
        let hist = Vec::new();
        Self { regs, pc, bus, stats, hist }
    }

    pub fn pipeline(&mut self) -> Result {
        let ins = self.bus.load(self.pc, B64)? as u32;
        if let Ok(ins) = Rv32i::id(ins) {
            self.datapath(ins)
        } else if let Ok(ins) = Rv64i::id(ins) {
            self.datapath(ins)
        } else {
            Err(Exception::IllegalInstruction(ins as u64))
        }
    }

    pub fn datapath<O: Extension + Display>(&mut self, i: O) -> Result {
        let record = HistItem { 
            src_regs: i.src_regs(), 
            dst_reg: i.dst_reg(), 
            blocking: i.is_ld() || i.is_st()
        };
        let ins_ex = i.ex(&self.regs);
        if ins_ex.is_ld() || ins_ex.is_st() {
            self.stats.mem_ops += 1;
        } else {
            self.stats.alu_ops += 1;
        }
        self.regs[0] = 0;
        self.pc = ins_ex.wr(self.pc, &mut self.regs, &mut self.bus)?;
        self.regs[0] = 0;
        self.hist.push(record);
        Ok(())
    }

    fn intersect<'a, T: PartialEq>(a: &'a [T], b: &'a [T]) -> Vec<&'a T> {
        a.iter()
            .filter(|item| b.contains(&item))
            .collect()
    }

    fn calc_stats(&mut self) {
        let mut cycles = 0;
        let mut stalls = 0;
        // 1. starting from the top of the hist:
        // 2. an instruction is executed if all src regs are available
        // 3. the instructions's dst regs are then added to the occupied list
        // 4. the instruction is removed from the history
        // 5. if we encounter the end of the list or a branch, we stop
        // 6. increment cycles and go to 1
        let mut executed = vec![false; self.hist.len()];
        'cycle: loop {
            cycles += 1;
            let mut occupied_regs = Vec::new();
            let iter = executed.iter_mut().enumerate()
                .filter(|(_, done)| !**done);
            for (i, done) in iter {
                if Self::intersect(&self.hist[i].src_regs, &occupied_regs).is_empty() {
                    // we can execute this op
                    *done = true;
                }
                if let Some(dst) = self.hist[i].dst_reg {
                    occupied_regs.push(dst);
                }
                if self.hist[i].blocking {
                    stalls += 1;
                    continue 'cycle;
                }
            }
            if executed.iter().all(|e| *e) {
                self.stats.cycles = cycles;
                self.stats.stalls = stalls;
                break;
            }
        }
    }

    pub fn execute(&mut self) -> Exception {
        loop {
            // execute instruction, add dst registers to dependents
            // don't execute beyond branch
            match self.pipeline() {
                Ok(_) => {},
                Err(ex) => if ex.is_fatal() {
                    self.calc_stats();
                    return ex
                },
            }
        }
    }
}