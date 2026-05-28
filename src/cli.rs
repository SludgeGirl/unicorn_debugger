use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    /// Run a debug script
    #[arg(short = 'f', long)]
    pub debug_file: Option<String>,

    /// Debug program in debug repl
    #[arg(short, long)]
    pub debug: bool,

    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Path to executable MsDos EXE
    pub program_path: String,

    pub args: Vec<String>,
}

impl CliArgs {
    pub fn debug_mode(&self) -> bool {
        self.debug || self.debug_file.is_some()
    }
}
