use clap::{Parser, Subcommand};
use rwlinux::{
    devmem::Devmem,
    matrix::{init_terminal, reset_terminal, start, Matrix, Result},
};

#[derive(Parser)]
#[clap(
    author("Han Ning <ning.han@intel.com>"),
    version("0.1"),
    about("Read Write on Linux")
)]
pub struct RwApp {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Access physical memory via /dev/mem node
    Devmem,
}

pub fn run() -> Result<()> {
    let app = RwApp::parse();

    match app.command {
        Command::Devmem => {
            let mut terminal = init_terminal()?;
            let mut dm: Matrix<Devmem> = Matrix::new("/dev/mem");
            let res = start(&mut terminal, &mut dm);
            reset_terminal()?;
            if let Err(err) = res {
                println!("{:?}", err);
            }
            Ok(())
        }
    }
}

fn main() {
    run().unwrap();
}
