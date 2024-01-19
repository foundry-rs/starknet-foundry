use std::io::{Error, Write};

pub trait PrintOutput: Clone + Default {
    fn print_stdout(stdout: &[u8]) -> Result<(), Error>;
    fn print_stderr(stderr: &[u8]) -> Result<(), Error>;
}

#[derive(Clone, Default)]
pub struct PassByPrint;

impl PrintOutput for PassByPrint {
    fn print_stdout(stdout: &[u8]) -> Result<(), Error> {
        std::io::stdout().write_all(stdout)
    }
    fn print_stderr(stderr: &[u8]) -> Result<(), Error> {
        std::io::stderr().write_all(stderr)
    }
}
