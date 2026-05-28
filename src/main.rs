use clap::Parser;

use crate::{debugger::Debugger, engine::Engine, program::Program};

mod cli;
mod debugger;
mod engine;
mod program;

fn main() {
    let args = cli::CliArgs::parse();
    let program = Program::new(&args.program_path, 0x1000);
    let mut engine = Engine::new(program, args.args.join(" ").as_bytes());
    engine.set_verbose(args.verbose);

    if args.debug_mode() {
        let mut debug = Debugger::new(engine);
        if let Some(file) = &args.debug_file {
            debug.run_file(file);
        } else {
            debug.repl();
        }
    } else {
        engine.start();
    }
}
