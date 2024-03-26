pub mod cmd;
use crate::cmd::{App, Command};

fn main() {
    let app: App = argh::from_env();

    match app.cmd {
        Command::Compile(compile) => compile.run(),
    }
}
