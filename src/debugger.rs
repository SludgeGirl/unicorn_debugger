use std::{
    fs,
    io::{self, BufRead, Write},
    process::exit,
};

use crate::engine::Engine;

pub struct Debugger<'a> {
    pub engine: Engine<'a>,
}

impl<'a> Debugger<'a> {
    pub fn new(engine: Engine<'a>) -> Self {
        Self { engine }
    }

    fn run(&mut self) {
        if self.engine.exited() {
            exit(0);
        }
        self.engine.start();
    }

    fn cont(&mut self) {
        if self.engine.exited() {
            exit(0);
        }

        self.engine.cont();
    }

    fn add_break(&mut self, cmd: &str) {
        let addr = cmd.split_whitespace().nth(1).unwrap();
        let addr = u64::from_str_radix(addr, 16).unwrap();
        println!("Adding break: {:016x}", addr);
        self.engine.add_break(addr);
    }

    fn run_command(&mut self, cmd: &str) {
        if cmd == "q" || cmd == "quit" || cmd == "exit" {
            exit(0);
        } else if cmd == "p" || cmd == "print" {
            println!("{:x}", self.engine.get_rsi());
        } else if cmd == "r" || cmd == "run" {
            self.run();
        } else if cmd == "c" || cmd == "continue" {
            self.cont();
        } else if cmd.starts_with("b ") || cmd.starts_with("break ") {
            self.add_break(cmd);
        } else {
            println!("Unknown command {cmd}");
        }
    }

    pub fn run_file(&mut self, path: &str) {
        let file_data = fs::read_to_string(path).unwrap();
        for line in file_data.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if line.starts_with('#') {
                continue;
            }

            self.run_command(line);
        }
    }

    pub fn repl(&mut self) {
        loop {
            print!("> ");
            io::stdout().flush().unwrap();
            let mut cmd = String::new();
            let _ = io::stdin().lock().read_line(&mut cmd).unwrap();
            self.run_command(cmd.trim());
        }
    }
}
