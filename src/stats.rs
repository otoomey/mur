use std::fmt::Display;

use tabled::{builder::Builder, settings::Style};

pub struct Stats {
    pub cycles: usize,
    pub stall_cycles: usize,
    pub exec_cycles: usize,
    pub mem_cycles: usize
}

impl Stats {
    pub fn new() -> Self {
        Self {
            cycles: 0,
            stall_cycles: 0,
            exec_cycles: 0,
            mem_cycles: 0,
        }
    }
}

impl Display for Stats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut table = Builder::new();
        table.set_header(["Stat", "Value"]);
        table.push_record(["Cycles", &format!("{}", self.cycles)]);
        table.push_record(["Stall Cycles", &format!("{}", self.stall_cycles)]);
        table.push_record(["Exec Cycles", &format!("{}", self.exec_cycles)]);
        table.push_record(["Mem Cycles", &format!("{}", self.mem_cycles)]);
        let table = table.build()
            .with(Style::ascii_rounded())
            .to_string();
        writeln!(f, "{}", table)?;
        Ok(())
    }
}
