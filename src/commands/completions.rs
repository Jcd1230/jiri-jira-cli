use clap::CommandFactory;
use clap_complete::{generate, Shell};
use std::io;

pub fn run(shell: Shell) {
    let mut cmd = crate::Cli::command();
    generate(shell, &mut cmd, "jiri", &mut io::stdout());
}
