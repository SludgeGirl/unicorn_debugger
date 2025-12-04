use std::{
    fs,
    io::{self, BufRead, Write},
    num::ParseIntError,
    process::exit,
};

use crate::engine::{Engine, FarPointer};

#[derive(Debug)]
enum Command {
    Quit,
    Print(String),
    Run,
    Next,
    Continue,
    Logon,
    Logoff,
    Break(String),
    WhileBreak { addr: u64, commands: Vec<Command> },
}

#[derive(Debug)]
enum ParseVal {
    Comment,
    BlockEnd,
    Command(Command),
}

#[derive(Debug)]
struct Ast {
    commands: Vec<Command>,
}

impl Ast {
    fn new(file: &str) -> Self {
        let mut commands = Vec::new();

        let mut idx = 0;
        let lines: Vec<&str> = file.lines().collect();
        while let Some((value, next_idx)) = Self::parse_command(idx, &lines, false) {
            if let ParseVal::Command(command) = value {
                commands.push(command);
            }
            idx = next_idx;
        }

        Self { commands }
    }

    fn parse_command(idx: usize, lines: &[&str], in_block: bool) -> Option<(ParseVal, usize)> {
        if idx >= lines.len() {
            return None;
        }

        let line = lines[idx];
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            return Some((ParseVal::Comment, idx + 1));
        }

        if in_block && line == "}" {
            return Some((ParseVal::BlockEnd, idx + 1));
        }

        let (command, size) = if line == "q" || line == "quit" || line == "exit" {
            (Command::Quit, 1)
        } else if line == "p" || line == "print" {
            (Command::Print("".into()), 1)
        } else if line.starts_with("p ") || line.starts_with("print ") {
            (Command::Print(line.into()), 1)
        } else if line == "r" || line == "run" {
            (Command::Run, 1)
        } else if line == "n" || line == "next" {
            (Command::Next, 1)
        } else if line == "c" || line == "continue" {
            (Command::Continue, 1)
        } else if line == "logon" {
            (Command::Logon, 1)
        } else if line == "logoff" {
            (Command::Logoff, 1)
        } else if line.starts_with("b ") || line.starts_with("break ") {
            (Command::Break(line.into()), 1)
        } else if line.starts_with("while") {
            Self::parse_while(idx, lines)
        } else {
            panic!("Unknown command {line} on line {}", idx + 1);
        };

        Some((ParseVal::Command(command), idx + size))
    }

    fn parse_while(idx: usize, lines: &[&str]) -> (Command, usize) {
        let mut idx = idx;
        let line_num = idx + 1;

        let line = lines[idx].trim();
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            panic!("line {line_num}: while statement requires 4 parts");
        }

        if parts[1] != "break" {
            panic!("line {line_num}: only 'break' is supported after while command");
        }

        let addr = if let Ok(addr) = Self::parse_addr(&parts[2]) {
            addr
        } else {
            panic!(
                "line {line_num}: cannot parse addr '{}' after break",
                parts[2]
            );
        };

        if parts[3] != "{" {
            panic!("line {line_num}: expected '{{' after address")
        };

        // move to the next line and start parsin the commands
        idx += 1;
        let mut end_found = false;
        let mut commands = Vec::new();
        while let Some((value, next_idx)) = Self::parse_command(idx, &lines, true) {
            idx = next_idx;
            match value {
                ParseVal::BlockEnd => {
                    end_found = true;
                    break;
                }
                ParseVal::Command(command) => commands.push(command),
                _ => {}
            }
        }

        if !end_found {
            panic!("expected closing '}}' after a while command ")
        }

        (Command::WhileBreak { addr, commands }, idx)
    }

    fn parse_addr(addr: &str) -> Result<u64, ParseIntError> {
        if let Some(addrs) = addr.split_once(':') {
            let segment = u64::from_str_radix(addrs.0, 16)?;
            let offset = u64::from_str_radix(addrs.1, 16)?;
            Ok(segment * 16 + offset)
        } else {
            u64::from_str_radix(addr, 16)
        }
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

    fn print(&self, cmd: &str) {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        let cpu = self.engine.read_cpu();
        if parts.len() == 1 {
            println!("{cpu}");
            return;
        }

        let (at, addr) = if let Ok(addr) = Ast::parse_addr(&parts[1]) {
            (parts[1].into(), addr)
        } else if let Some((reg1, reg2)) = parts[1].split_once(':') {
            let segment = cpu.register(reg1);
            let offset = cpu.register(reg2);
            let fp = FarPointer::from_segment_offset(segment, offset);
            (format!("{}[{segment}:{offset}]", parts[1]), fp.address())
        } else {
            panic!("at the disco")
        };

        println!("Data(u16) at {at}: {:x}", self.engine.read_mem(addr));
    }

    fn run_commands(&mut self, commands: &[Command]) {
        for command in commands {
            match command {
                Command::Quit => exit(0),
                Command::Print(cmd) => self.print(cmd),
                Command::Run => self.run(),
                Command::Next => self.next(),
                Command::Continue => self.cont(),
                Command::Logon => self.engine.set_verbose(true),
                Command::Logoff => self.engine.set_verbose(false),
                Command::Break(cmd) => self.add_break(cmd),
                Command::WhileBreak { addr, commands } => {
                    self.engine.add_while_break(*addr);
                    loop {
                        self.cont();
                        let ip = FarPointer::read_engine(self.engine.engine());
                        if ip.address() != *addr {
                            break;
                        }

                        self.run_commands(commands);
                    }
                }
            }
        }
    }

    fn run_ast(&mut self, ast: &Ast) {
        self.run_commands(&ast.commands);
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
