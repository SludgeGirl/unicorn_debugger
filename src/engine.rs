use crate::program::Program;
use std::{collections::HashMap, fmt::Display, rc::Rc};
use unicorn_engine::{Arch, Mode, Prot, RegisterX86, Unicorn};

struct FarPointer {
    cs: u64,
    ip: u64,
}

impl FarPointer {
    fn read_engine(engine: &Unicorn<EngineData>) -> Self {
        let cs = engine.reg_read(RegisterX86::CS).unwrap();
        let ip = engine.reg_read(RegisterX86::IP).unwrap();
        Self { cs, ip }
    }

    fn address(&self) -> u64 {
        self.cs * 16 + self.ip
    }
}

impl Display for FarPointer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04x}:{:04x}", self.cs, self.ip)
    }
}

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
        let mut engine = Unicorn::new_with_data(Arch::X86, Mode::MODE_16, data).unwrap();
        engine.mem_map(0, 8 * 1024 * 1024, Prot::ALL).unwrap();
        let program = engine.get_data().program.clone();

        // the start is a far pointer segment thingy so we need to multiply it with 16
        engine
            .mem_write(program.start() * 16, program.data())
            .unwrap();

        engine
            .reg_write(RegisterX86::IP, program.header().initial_ip as u64)
            .unwrap();
        engine
            .reg_write(RegisterX86::SP, program.header().initial_sp as u64)
            .unwrap();
        engine
            .reg_write(
                RegisterX86::CS,
                program.header().initial_cs as u64 + program.start(),
            )
            .unwrap();
        engine
            .reg_write(
                RegisterX86::SS,
                program.header().initial_ss as u64 + program.start(),
            )
            .unwrap();

        engine
            .add_code_hook(program.start(), 0, |emu, addr, len| {
                let decoder = yaxpeax_x86::real_mode::InstDecoder::default();
                let inst = decoder
                    .decode_slice(&emu.mem_read_as_vec(addr, len as usize).unwrap())
                    .unwrap();
                let fp = FarPointer::read_engine(&emu);
                println!("code exec: [{fp}]: {}", inst.to_string());
                // NOTE: remove this to keep running the VM, it might crash though!
                emu.emu_stop().unwrap();
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
        let ip = FarPointer::read_engine(&self.engine);
        self.engine.emu_start(ip.address(), 8192, 0, 0).unwrap()
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
