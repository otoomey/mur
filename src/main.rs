use std::{path::PathBuf, fs::File, io::Read};

use clap::Parser;
use dart::DartSoC;

use crate::{isa::print_register_table, zeus::ZeusSoC, kronos::KronosSoC, atlas::AtlasSoC};

mod mem;
mod bus;
mod isa;
mod exception;
mod dart;
mod zeus;
mod kronos;
mod atlas;
mod stats;

#[derive(clap::Parser)]
struct Args {
    path: PathBuf,
    #[arg(long, default_value="all")]
    soc: String
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let mut file = File::open(args.path)?;
    let mut bin = Vec::new();
    file.read_to_end(&mut bin)?;

    match args.soc.as_str() {
        "dart" => {
            let mut cpu = DartSoC::new(bin);
            let ex = cpu.execute();
            println!("Dart exited with exception {:?}", ex);
            print_register_table(&cpu.regs);
            println!("{}", cpu.stats);
            Ok(())
        },
        "zeus" => {
            let mut cpu = ZeusSoC::new(bin);
            let ex = cpu.execute();
            println!("Zeus exited with exception {:?}", ex);
            print_register_table(&cpu.regs);
            println!("{}", cpu.stats);
            Ok(())
        },
        "kronos" => {
            let mut cpu = KronosSoC::new(bin);
            let ex = cpu.execute();
            println!("Kronos exited with exception {:?}", ex);
            print_register_table(&cpu.regs);
            println!("{}", cpu.stats);
            Ok(())
        },
        "atlas" => {
            let mut cpu = AtlasSoC::new(bin);
            let ex = cpu.execute();
            println!("Atlas exited with exception {:?}", ex);
            print_register_table(&cpu.regs);
            println!("{}", cpu.stats);
            Ok(())
        },
        _ => Err(format!("Unknown SoC type {}", args.soc).into())
    }
}
