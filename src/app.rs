use super::devmem;
use clap::{Parser, Subcommand};
use std::error::Error;

#[derive(Parser)]
#[clap(
    name("RW-Linux"),
    author("Han Ning <ning.han@intel.com>"),
    version("0.1"),
    about("Read Write on Linux")
)]
pub struct App {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Access physical memory through /dev/mem
    Devmem,
    /// Access I/O space
    Io,
}

pub fn run() -> Result<(), Box<dyn Error>> {
    let app = App::parse();

    match app.command {
        Commands::Devmem => devmem::run(),
        Commands::Io => todo!(),
    }
}
