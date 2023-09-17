
use crate::{soc::{SoC, Isa, Exit}, stats::Stats, bus::{Bus, RAM_END, RAM_BASE}, mem::B32, exception::Exception, csr::Csr};

type IFOut = Option<u32>;
type IDOut = Option<u32>;
type EXAluOut = Option<(usize, u64)>;
type EXLdOut = Option<u32>;
type EXBrOut = Option<u64>;

pub struct Cv64e40p {
    regfile: [u64; 32],
    pc: u64,
    bus: Bus,
    csr: Csr,

    ifetch: IFOut,
    idecode: IDOut,
    ex: EXAluOut,
    branch_pc: EXBrOut,
    ld_ins1: EXLdOut,
    ld_ins2: EXLdOut,
}

impl Cv64e40p {
    pub fn new(bin: Vec<u8>) -> Self {
        let mut regfile = [0; 32];
        regfile[2] = RAM_END;
        Self {
            regfile,
            pc: RAM_BASE,
            bus: Bus::new(bin),
            csr: Csr::new(),
            ifetch: None,
            idecode: None,
            ex: None,
            branch_pc: None,
            ld_ins1: None,
            ld_ins2: None
        }
    }
}

impl Cv64e40p {
    fn ifetch(&mut self) -> Result<(), Exception> {
        if self.ifetch.is_none() {
            let ins = self.bus.load(self.pc, B32)
                .map(|i| Some(i as u32))?;
            self.ifetch = ins;
        }
        Ok(())
    }

    fn idecode(&mut self) -> Result<(), Exception> {
        if let None = self.idecode {
            self.idecode = self.ifetch;
        }
        self.ifetch = None;
        Ok(())
    }

    fn ex(&mut self, stats: &mut Stats) -> Result<(), Exception> {
        if let (Some(idecode), None, None) = (self.idecode, self.ex, self.ld_ins1) {
            println!("executing ins: {:#07b}", Self::opcode(idecode));
            self.regfile[0] = 0;
            if Self::is_jmp(idecode) {
                let (pc, rd) = self.jmp(idecode)?;
                self.branch_pc = Some(pc);
                self.ex = Some((Self::rd(idecode), rd));
                self.idecode = None;
            } else if Self::is_br(idecode) {
                let pc = self.br(idecode)?;
                self.branch_pc = pc;
                self.idecode = None;
            } else if Self::is_ld(idecode) {
                self.ld_ins1 = Some(idecode);
                self.idecode = None;
                stats.mem_cycles += 1;
            } else if Self::is_st(idecode) {
                self.st(idecode)?;
                self.idecode = None;
                stats.mem_cycles += 1;
            } else if Self::is_zicsr(idecode) {
                let (csr, rd, ncsr) = self.zicsr(idecode)?;
                self.csr.store(csr, ncsr);
                self.ex = Some((Self::rd(idecode), rd));
                self.idecode = None;
            } else {
                let src_regs = self.src_regs(idecode);
                let stall = self.ld_ins2
                    .map(|ld| src_regs.contains(&Self::rd(ld)))
                    .unwrap_or(false);
                if !stall {
                    let result = self.alu(idecode)?;
                    self.ex = Some((Self::rd(idecode), result));
                    self.idecode = None;
                    stats.exec_cycles += 1;
                } else {
                    stats.stall_cycles += 1;
                }
            }
        }
        Ok(())
    }

    fn wr(&mut self) -> Result<(), Exception> {
        if let Some((rd, x)) = self.ex {
            self.regfile[rd] = x;
            self.ex = None;
        }
        if let Some(ld_ins2) = self.ld_ins2 {
            self.regfile[Self::rd(ld_ins2)] = self.ld(ld_ins2)?;
            self.ld_ins2 = None;
        }
        if let (Some(ld_ins1), None) = (self.ld_ins1, self.ld_ins2) {
            self.ld_ins2 = Some(ld_ins1);
            self.ld_ins1 = None;
        }
        if let Some(pc) = self.branch_pc {
            self.pc = pc;
            // flush pipeline:
            self.idecode = None;
            self.ifetch = None;
            self.branch_pc = None;
        }
        Ok(())
    }
}

impl SoC for Cv64e40p {
    fn execute(&mut self) -> Result<(), Exit> {
        let mut stats = Stats::new();
        loop {
            stats.cycles += 1;

            match self.wr() {
                Ok(_) => {},
                Err(ex) => {
                    if ex.is_fatal() {
                        return Err(Exit::from_ex(stats, ex));
                    }
                },
            }

            match self.ex(&mut stats) {
                Ok(_) => {},
                Err(ex) => {
                    if ex.is_fatal() {
                        return Err(Exit::from_ex(stats, ex));
                    }
                },
            }

            match self.idecode() {
                Ok(_) => {},
                Err(ex) => {
                    if ex.is_fatal() {
                        return Err(Exit::from_ex(stats, ex));
                    }
                },
            }

            match self.ifetch() {
                Ok(_) => {},
                Err(ex) => {
                    if ex.is_fatal() {
                        return Err(Exit::from_ex(stats, ex));
                    }
                },
            }

            self.pc += 4;
        }
    }

    fn pc(&self) -> u64 {
        self.pc
    }

    fn regfile(&self) -> &[u64; 32] {
        &self.regfile
    }

    fn bus_mut(&mut self) -> &mut crate::bus::Bus {
        &mut self.bus
    }

    fn bus(&self) -> &crate::bus::Bus {
        &self.bus
    }

    fn csr(&self) -> &Csr {
        &self.csr
    }

    fn csr_mut(&mut self) -> &mut Csr {
        &mut self.csr
    }
}

impl Isa for Cv64e40p {}
