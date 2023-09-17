use std::{path::PathBuf, fs::File, io::{self, Read}, collections::HashMap};

use clap::Parser;
use cv64e40p::Cv64e40p;
use soc::SoC;

use crate::soc::Exit;

mod soc;
mod mem;
mod bus;
mod csr;
mod exception;
mod cv64e40p;
mod zeus64;
mod stats;

#[derive(clap::Parser)]
struct Args {
    path: PathBuf,
    #[arg(long, default_value="all")]
    soc: String
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    let mut file = File::open(args.path)?;
    let mut bin = Vec::new();
    file.read_to_end(&mut bin)?;

    let socs: HashMap<&str, _> = [
        ("cv32e40p", Box::new(|bin| { Cv64e40p::new(bin) }))
    ].into_iter().collect();


    if socs.contains_key(args.soc.as_str()) {
        let mut cpu = socs[args.soc.as_str()](bin);
        let exit = cpu.execute();
        match exit {
            Ok(_) => {},
            Err(Exit { stats, ex }) => {
                println!("Exited with exception {:?}", ex);
                println!("{}", stats);
            },
        }
        println!("{} [pc: {}] Register File:", args.soc.as_str(), cpu.pc());
        cpu.dump_registers();
    } else if args.soc == "all" {
        socs.values().for_each(|i| {
            let mut cpu = i(bin.clone());
            let exit = cpu.execute();
            match exit {
                Ok(_) => {},
                Err(Exit { stats, ex }) => {
                    println!("Exited with exception {:?}", ex);
                    println!("{}", stats);
                },
            }
        })
    } else {
        panic!("Unknown SoC type {}", args.soc)
    }
    Ok(())
}
