use std::process::{Command, Output};

use color_eyre::{eyre::eyre, Help, Report, Result, SectionExt};

mod private {
    pub trait Sealed {}
    impl<T> Sealed for T {}
}

pub trait CommandExt: private::Sealed {
    fn run(&mut self) -> Result<()>;
    fn read(&mut self) -> Result<String>;
}

impl CommandExt for Command {
    fn run(&mut self) -> Result<()> {
        let Output {
            status,
            stderr,
            stdout,
        } = self.output()?;

        if status.success() {
            return Ok(());
        }

        Err(wrap_command_error(&stdout, &stderr))
    }

    fn read(&mut self) -> Result<String> {
        let Output {
            status,
            stderr,
            stdout,
        } = self.output()?;

        if status.success() {
            return Ok(String::from_utf8_lossy(&stdout).into());
        }

        Err(wrap_command_error(&stdout, &stderr))
    }
}

pub fn wrap_command_error(stdout: &[u8], stderr: &[u8]) -> Report {
    let stdout = String::from_utf8_lossy(stdout);
    let stderr = String::from_utf8_lossy(stderr);

    eyre!("Command execution failure")
        .with_section(|| stdout.trim().to_string().header("Stdout"))
        .with_section(|| stderr.trim().to_string().header("Stderr"))
}
