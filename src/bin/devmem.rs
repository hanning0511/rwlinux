use rwlinux::{
    devmem::Devmem,
    matrix::{init_terminal, reset_terminal, start, Matrix, Result},
};

fn main() -> Result<()> {
    let mut terminal = init_terminal()?;
    let mut devmem: Matrix<Devmem> = Matrix::new("/dev/mem");
    let res = start(&mut terminal, &mut devmem);
    reset_terminal()?;
    if let Err(err) = res {
        println!("{:?}", err);
    }
    Ok(())
}
