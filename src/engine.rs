use crate::program::Program;
use std::{collections::HashMap, rc::Rc};
use unicorn_engine::{Arch, Mode, Prot, RegisterX86, Unicorn};

#[derive(Debug, Clone, Copy)]
struct EngineBreak {
    addr: u64,
    /// Is currently being interrupted
    intr: bool,
}

impl EngineBreak {
    fn new(addr: u64) -> Self {
        Self { addr, intr: false }
    }
}

struct EngineData {
    program: Rc<Program>,
    /// address -> break data
    breaks: HashMap<u64, EngineBreak>,
    exited: bool,
}

impl EngineData {
    fn new(program: Program) -> Self {
        Self {
            program: Rc::new(program),
            breaks: HashMap::new(),
            exited: false,
        }
    }

    fn start(&self) -> u64 {
        self.program.start()
    }

    fn add_break(&mut self, ebreak: EngineBreak) {
        self.breaks.insert(ebreak.addr, ebreak);
    }

    fn get_break(&self, addr: u64) -> Option<&EngineBreak> {
        self.breaks.get(&addr)
    }

    fn get_break_mut(&mut self, addr: u64) -> Option<&mut EngineBreak> {
        self.breaks.get_mut(&addr)
    }
}

pub struct Engine<'a> {
    engine: Unicorn<'a, EngineData>,
}

impl<'a> Engine<'a> {
    #[allow(dead_code)]
    fn clear_cache(&mut self) {
        // we need to invalidate the cache to make sure the code changes are applied
        // https://github.com/unicorn-engine/unicorn/wiki/FAQ#editing-an-instruction-doesnt-take-effecthooks-added-during-emulation-are-not-called
        self.engine.ctl_remove_cache(0, 8 * 1024 * 1024).unwrap();
    }

    pub fn new(program: Program) -> Self {
        let data = EngineData::new(program);
        let mut engine = Unicorn::new_with_data(Arch::X86, Mode::MODE_64, data).unwrap();
        engine.mem_map(0, 8 * 1024 * 1024, Prot::ALL).unwrap();
        let program = engine.get_data().program.clone();

        // for section in program.sections() {
        //     if let Some(data) = program.section_data(section) {
        //         engine.mem_write(section.sh_addr, data).unwrap();
        //     }
        // }

        engine
            .add_insn_sys_hook(
                unicorn_engine::X86Insn::SYSCALL,
                program.start(),
                0,
                |emu| {
                    let syscall = emu.reg_read(RegisterX86::RAX).unwrap();

                    if syscall == 1 {
                        let fd = emu.reg_read(RegisterX86::RDI).unwrap();
                        let data_ptr = emu.reg_read(RegisterX86::RSI).unwrap();
                        let data_len = emu.reg_read(RegisterX86::RDX).unwrap();
                        let data_from_mem =
                            emu.mem_read_as_vec(data_ptr, data_len as usize).unwrap();

                        if fd == 1 {
                            print!("{}", String::from_utf8(data_from_mem).unwrap())
                        } else if fd == 2 {
                            eprint!("{}", String::from_utf8(data_from_mem).unwrap())
                        } else {
                            println!("cannot write to fd '{fd}'");
                            emu.emu_stop().unwrap();
                        }
                    } else if syscall == 60 {
                        emu.emu_stop().unwrap();
                        println!("exit captured, stopping emulation");
                        emu.get_data_mut().exited = true;
                    } else {
                        println!("unknown syscall '{syscall}' captured, stopping emulation");
                        emu.emu_stop().unwrap();
                    }
                },
            )
            .unwrap();

        engine
            .add_code_hook(program.start(), 0, |emu, addr, _len| {
                let has_break = emu.get_data().get_break(addr).is_some();
                if has_break {
                    let is_intr = emu.get_data().get_break(addr).unwrap().intr;
                    if is_intr {
                        emu.emu_stop().unwrap();
                    }
                    let ebreak = emu.get_data_mut().get_break_mut(addr).unwrap();
                    ebreak.intr = !ebreak.intr;
                }
            })
            .unwrap();

        Self { engine }
    }

    pub fn exited(&self) -> bool {
        self.engine.get_data().exited
    }

    pub fn add_break(&mut self, addr: u64) {
        self.engine.get_data_mut().add_break(EngineBreak::new(addr));
    }

    pub fn start(&mut self) {
        self.engine
            .emu_start(self.engine.get_data().start(), 8192, 0, 0)
            .unwrap()
    }

    // TODO: proper function for getting CPU info
    pub fn get_rsi(&mut self) -> u64 {
        self.engine.reg_read(RegisterX86::RSI).unwrap()
    }

    /// Continue run where enigne was stopped
    pub fn cont(&mut self) {
        self.engine
            .emu_start(self.engine.pc_read().unwrap(), 8192, 0, 0)
            .unwrap()
    }
}
