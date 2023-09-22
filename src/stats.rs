use std::fmt::Display;

use tabled::{builder::Builder, settings::Style};

#[derive(Copy, Clone)]
pub struct Stats {
    pub cycles: usize,
    pub stalls: usize,
    pub alu_ops: usize,
    pub mem_ops: usize
}

impl Stats {
    pub fn new() -> Self {
        Self {
            cycles: 0,
            stalls: 0,
            alu_ops: 0,
            mem_ops: 0,
        }
    }
}

impl Display for Stats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut table = Builder::new();
        table.set_header(["Stat", "Value"]);
        table.push_record(["Cycles", &format!("{}", self.cycles)]);
        table.push_record(["Stalls", &format!("{}", self.stalls)]);
        table.push_record(["ALU ops", &format!("{}", self.alu_ops)]);
        table.push_record(["Mem ops", &format!("{}", self.mem_ops)]);
        let table = table.build()
            .with(Style::ascii_rounded())
            .to_string();
        writeln!(f, "{}", table)?;
        Ok(())
    }
}
