use crate::{mem::{Mem, Bits}, exception::Exception};

pub const RAM_BASE: u64 = 0x8000_0000;
pub const RAM_SIZE: u64 = 1024 * 1024 * 128;
pub const RAM_END: u64 = RAM_SIZE + RAM_BASE - 1;

pub struct Bus {
    pub mem: Mem
}

impl Bus {
    pub fn new(program: Vec<u8>) -> Bus {
        let mut mem = vec![0; RAM_SIZE as usize];
        mem.splice(..program.len(), program.into_iter());
        Self { mem: Mem::new(mem) }
    }

    pub fn load(&self, addr: u64, bits: Bits) -> Result<u64, Exception> {
        match addr {
            RAM_BASE..=RAM_END => Ok(self.mem.load(addr - RAM_BASE, bits)),
            _ => Err(Exception::LoadAccessFault(addr))
        }
    }

    pub fn store(&mut self, addr: u64, bits: Bits, value: u64) -> Result<(), Exception> {
        match addr {
            RAM_BASE..=RAM_END => Ok(self.mem.store(addr - RAM_BASE, bits, value)),
            _ => Err(Exception::StoreAMOAccessFault(addr))
        }
    }
}