use crate::{bus::{Bus, RAM_END, RAM_BASE}, stats::Stats, mem::B64, isa::{Rv32i, Extension, Rv64i}, exception::Exception};

pub struct DartSoC {
    pub regs: [u64; 32],
    pub pc: u64,
    pub bus: Bus,
    pub stats: Stats
}

type Result = std::result::Result<(), Exception>;

impl DartSoC {
    pub fn new(bin: Vec<u8>) -> Self {
        let mut regs = [0_u64; 32];
        regs[2] = RAM_END;
        let pc = RAM_BASE;
        let bus = Bus::new(bin);
        let stats = Stats::new();
        Self { regs, pc, bus, stats }
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

    pub fn datapath<O: Extension>(&mut self, i: O) -> Result {
        let ins_ex = i.ex(&self.regs);
        if ins_ex.is_ld() || ins_ex.is_st() {
            self.stats.mem_ops += 1;
        } else {
            self.stats.alu_ops += 1;
        }
        self.regs[0] = 0;
        self.pc = ins_ex.wr(self.pc, &mut self.regs, &mut self.bus)?;
        self.regs[0] = 0;
        Ok(())
    }

    pub fn execute(&mut self) -> Exception {
        loop {
            self.stats.cycles += 1;
            match self.pipeline() {
                Ok(_) => {},
                Err(ex) => if ex.is_fatal() {
                    return ex
                },
            }
        }
    }
}