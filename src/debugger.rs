use std::{
    fs,
    io::{self, BufRead, Write},
    process::exit,
};

use crate::engine::Engine;

enum Command {
    Quit,
    Print,
    Run,
    Next,
    Continue,
    Logon,
    Logoff,
    Break(String),
}

struct Ast {
    commands: Vec<Command>,
}

impl Ast {
    fn new(file: &str) -> Self {
        let mut commands = Vec::new();

        for (idx, line) in file.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if line == "q" || line == "quit" || line == "exit" {
                commands.push(Command::Quit);
            } else if line == "p" || line == "print" {
                commands.push(Command::Print);
            } else if line == "r" || line == "run" {
                commands.push(Command::Run);
            } else if line == "n" || line == "next" {
                commands.push(Command::Next);
            } else if line == "c" || line == "continue" {
                commands.push(Command::Continue);
            } else if line == "logon" {
                commands.push(Command::Logon);
            } else if line == "logoff" {
                commands.push(Command::Logoff);
            } else if line.starts_with("b ") || line.starts_with("break ") {
                commands.push(Command::Break(line.into()));
            } else {
                panic!("Unknown command {line} on line {}", idx + 1);
            }
        }

        Self { commands }
    }
}

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

    fn next(&mut self) {
        if self.engine.exited() {
            exit(0);
        }

        self.engine.step();
    }

    fn add_break(&mut self, cmd: &str) {
        let addr = cmd.split_whitespace().nth(1).unwrap();
        let addr = if let Some(addrs) = addr.split_once(':') {
            let segment = u64::from_str_radix(addrs.0, 16).unwrap();
            let offset = u64::from_str_radix(addrs.1, 16).unwrap();
            segment * 16 + offset
        } else {
            u64::from_str_radix(addr, 16).unwrap()
        };

        self.engine.add_break(addr);
    }

    fn run_ast(&mut self, ast: &Ast) {
        for command in &ast.commands {
            match command {
                Command::Quit => exit(0),
                Command::Print => println!("{}", self.engine.read_cpu()),
                Command::Run => self.run(),
                Command::Next => self.next(),
                Command::Continue => self.cont(),
                Command::Logon => self.engine.set_verbose(true),
                Command::Logoff => self.engine.set_verbose(false),
                Command::Break(cmd) => self.add_break(cmd),
            }
        }
    }

    pub fn run_file(&mut self, path: &str) {
        let file_data = fs::read_to_string(path).unwrap();
        let ast = Ast::new(&file_data);
        self.run_ast(&ast);
    }

    pub fn repl(&mut self) {
        loop {
            print!("> ");
            io::stdout().flush().unwrap();
            let mut cmd = String::new();
            let _ = io::stdin().lock().read_line(&mut cmd).unwrap();
            let ast = Ast::new(&cmd);
            self.run_ast(&ast);
        }
    }
}
