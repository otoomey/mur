use crate::soc::{Neumann, SoC, Isa};

pub struct Zeus64 {
    soc: Neumann
}

impl Zeus64 {
    pub fn new(bin: Vec<u8>) -> Self {
        Self {
            soc: Neumann::new(bin, 0x8000_0000)
        }
    }
}

impl Isa for Zeus64 {}

impl SoC for Zeus64 {
    fn execute(&mut self) -> crate::stats::Stats {
        todo!()
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