use std::{path::PathBuf, fs::File, io::{self, Read}, collections::HashMap};

use clap::Parser;
use cv64e40p::Cv64e40p;
use soc::SoC;

mod soc;
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
        let stats = cpu.execute();
        cpu.dump_registers();
        println!("{}", stats);
    } else if args.soc == "all" {
        socs.values().for_each(|i| {
            let mut cpu = i(bin.clone());
            cpu.execute();
        })
    } else {
        panic!("Unknown SoC type {}", args.soc)
    }
    Ok(())
}
