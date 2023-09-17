

pub struct Mem {
    mem: Vec<u8>
}

pub struct Bits {
    size: u64
}

pub const B8: Bits = Bits {size: 1 };
pub const B16: Bits = Bits {size: 2 };
pub const B32: Bits = Bits {size: 4 };
pub const B64: Bits = Bits {size: 8 };

impl Mem {
    pub fn new(mem: Vec<u8>) -> Self {
        Self { mem }
    }

    pub fn load(&self, addr: u64, bits: Bits) -> u64 {
        (0..bits.size)
            .map(|i| (self.mem[(addr + i) as usize] as u64) << (i * 8))
            .reduce(|a, b| a | b)
            .unwrap_or(0)
    }

    pub fn store(&mut self, addr: u64, bits: Bits, value: u64) {
        (0..bits.size).for_each(|i| {
            let offset = 8 * i as usize;
            self.mem[(addr + i) as usize] = ((value >> offset) & 0xff) as u8;
        })
    }
}