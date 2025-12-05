use crate::program::{PSP, Program};
use std::{collections::HashMap, fmt::Display, rc::Rc};
use unicorn_engine::{Arch, Mode, Prot, RegisterX86, Unicorn};

/// Addresses are 16 bit, but u64 makes it easier to work with unicorn
pub struct Cpu {
    ax: u64,
    bx: u64,
    cx: u64,
    dx: u64,
    si: u64,
    di: u64,
    sp: u64,
    bp: u64,
    ip: u64,
    cs: u64,
    ds: u64,
    es: u64,
    ss: u64,
    fs: u64,
    gs: u64,
}

impl Cpu {
    fn read_engine(engine: &Unicorn<EngineData>) -> Self {
        let ax = engine.reg_read(RegisterX86::AX).unwrap();
        let bx = engine.reg_read(RegisterX86::BX).unwrap();
        let cx = engine.reg_read(RegisterX86::CX).unwrap();
        let dx = engine.reg_read(RegisterX86::DX).unwrap();
        let si = engine.reg_read(RegisterX86::SI).unwrap();
        let di = engine.reg_read(RegisterX86::DI).unwrap();
        let sp = engine.reg_read(RegisterX86::SP).unwrap();
        let bp = engine.reg_read(RegisterX86::BP).unwrap();
        let ip = engine.reg_read(RegisterX86::IP).unwrap();
        let cs = engine.reg_read(RegisterX86::CS).unwrap();
        let ds = engine.reg_read(RegisterX86::DS).unwrap();
        let es = engine.reg_read(RegisterX86::ES).unwrap();
        let ss = engine.reg_read(RegisterX86::SS).unwrap();
        let fs = engine.reg_read(RegisterX86::FS).unwrap();
        let gs = engine.reg_read(RegisterX86::GS).unwrap();

        Self {
            ax,
            bx,
            cx,
            dx,
            si,
            di,
            sp,
            bp,
            ip,
            cs,
            ds,
            es,
            ss,
            fs,
            gs,
        }
    }

    pub fn register(&self, register: &str) -> u64 {
        match register {
            "ax" => self.ax,
            "bx" => self.bx,
            "cx" => self.cx,
            "dx" => self.dx,
            "si" => self.si,
            "di" => self.di,
            "sp" => self.sp,
            "bp" => self.bp,
            "ip" => self.ip,
            "cs" => self.cs,
            "ds" => self.ds,
            "es" => self.es,
            "ss" => self.ss,
            "fs" => self.fs,
            "gs" => self.gs,
            _ => panic!("Unknown cpu register {register}"),
        }
    }
}

impl Display for Cpu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Cpu {{")?;
        writeln!(f, "    ax: {:04x},", self.ax)?;
        writeln!(f, "    bx: {:04x},", self.bx)?;
        writeln!(f, "    cx: {:04x},", self.cx)?;
        writeln!(f, "    dx: {:04x},", self.dx)?;
        writeln!(f, "    si: {:04x},", self.si)?;
        writeln!(f, "    di: {:04x},", self.di)?;
        writeln!(f, "    sp: {:04x},", self.sp)?;
        writeln!(f, "    bp: {:04x},", self.bp)?;
        writeln!(f, "    ip: {:04x},", self.ip)?;
        writeln!(f, "    cs: {:04x},", self.cs)?;
        writeln!(f, "    ds: {:04x},", self.ds)?;
        writeln!(f, "    es: {:04x},", self.es)?;
        writeln!(f, "    ss: {:04x},", self.ss)?;
        writeln!(f, "    fs: {:04x},", self.fs)?;
        writeln!(f, "    gs: {:04x},", self.gs)?;
        write!(f, "}}")?;

        Ok(())
    }
}

pub struct FarPointer {
    cs: u64,
    ip: u64,
}

impl FarPointer {
    pub fn read_engine(engine: &Unicorn<EngineData>) -> Self {
        let cs = engine.reg_read(RegisterX86::CS).unwrap();
        let ip = engine.reg_read(RegisterX86::IP).unwrap();
        Self { cs, ip }
    }

    pub fn from_segment_offset(segment: u64, offset: u64) -> Self {
        Self {
            cs: segment,
            ip: offset,
        }
    }

    pub fn address(&self) -> u64 {
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

pub struct EngineData {
    program: Rc<Program>,
    /// address -> break data
    breaks: HashMap<u64, EngineBreak>,
    /// started -> addr
    while_break: Option<(bool, u64)>,
    exited: bool,
    verbose: bool,
}

impl EngineData {
    fn new(program: Program) -> Self {
        Self {
            program: Rc::new(program),
            breaks: HashMap::new(),
            exited: false,
            verbose: false,
            while_break: None,
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

    pub fn engine(&self) -> &Unicorn<'a, EngineData> {
        &self.engine
    }

    pub fn new(program: Program) -> Self {
        let data = EngineData::new(program);
        let mut engine = Unicorn::new_with_data(Arch::X86, Mode::MODE_16, data).unwrap();
        engine.mem_map(0, 8 * 1024 * 1024, Prot::ALL).unwrap();
        let program = engine.get_data().program.clone();

        // the start is a far pointer segment thingy so we need to multiply it with 16
        let start_segment = program.start() * 16;
        let psp_segment = start_segment - 256;
        engine.mem_write(start_segment, program.data()).unwrap();

        let psp = &PSP::new(0x0, [0x0; 5]);
        let psp_data: &[u8] = psp.into();
        engine.mem_write(psp_segment, psp_data).unwrap();

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
            .reg_write(RegisterX86::DS, program.start() - 256)
            .unwrap();
        engine
            .reg_write(RegisterX86::ES, program.start() - 256)
            .unwrap();

        engine
            .add_code_hook(program.start(), 0, |emu, addr, len| {
                let fp = FarPointer::read_engine(&emu);
                if emu.get_data().verbose {
                    let decoder = yaxpeax_x86::real_mode::InstDecoder::default();
                    let inst = decoder
                        .decode_slice(&emu.mem_read_as_vec(addr, len as usize).unwrap())
                        .unwrap();
                    println!("code exec: [{fp}]: {}", inst.to_string());
                }

                let has_break = emu.get_data().get_break(addr).is_some();
                if has_break {
                    let is_intr = emu.get_data().get_break(addr).unwrap().intr;
                    if !is_intr {
                        println!("breaking at [{fp}]");
                        emu.emu_stop().unwrap();
                        if emu.get_data().while_break.is_some_and(|wb| wb.1 == addr) {
                            emu.get_data_mut().while_break = Some((true, addr));
                        }
                    }
                    let ebreak = emu.get_data_mut().get_break_mut(addr).unwrap();
                    ebreak.intr = !ebreak.intr;
                } else if emu.get_data().while_break.is_some_and(|wb| wb.0) {
                    println!("stopping after while break at [{fp}]");
                    emu.get_data_mut().while_break = None;
                    emu.emu_stop().unwrap();
                }
            })
            .unwrap();

        engine
            .add_intr_hook(|emu, num| {
                let cpu = Cpu::read_engine(&emu);
                if num == 0x21 {
                    let ah = cpu.ax >> 8;
                    if ah == 0x25 {
                        let al = cpu.ax & 0xff;
                        let handler_ptr = (cpu.ds * 16 + cpu.dx) as u32;
                        emu.mem_write(al * 4, &handler_ptr.to_le_bytes()).unwrap();
                    } else if ah == 0x30 {
                        // TXLIST.EXE is checking for DOS version 2 so lets set the dos version to that for now
                        emu.reg_write(RegisterX86::AL, 2).unwrap();
                    } else if ah == 0x35 {
                        let al = cpu.ax & 0xff;
                        emu.reg_write(RegisterX86::BX, al * 4).unwrap();
                        emu.reg_write(RegisterX86::ES, al * 4 + 2).unwrap();
                    } else if ah == 0x40 {
                        let ds = cpu.ds;
                        let dx = cpu.dx;
                        let addr = ds * 16 + dx;
                        let data = emu.mem_read_as_vec(addr, cpu.cx as usize).unwrap();
                        println!(
                            "Write to fd '{}', string: '{}'",
                            cpu.bx,
                            String::from_utf8_lossy(&data)
                        );
                    } else if ah == 0x4a {
                        // Dosbox is doing this so lets do it too for now?
                        if cpu.ax == 0x4a01 || cpu.ax == 0x4a02 {
                            emu.reg_write(RegisterX86::BX, 0).unwrap();
                            emu.reg_write(RegisterX86::ES, 0xffff).unwrap();
                            emu.reg_write(RegisterX86::DI, 0xffff).unwrap();
                        } else {
                            panic!("Only ax 0x4a01 and 0x4a02 are implemented for INT 21,4a");
                        }
                    } else if ah == 0x4c {
                        let al = cpu.ax & 0xff;
                        println!("Program terminating with code '0x{al:x}', exiting...");
                        emu.get_data_mut().exited = true;
                        emu.emu_stop().unwrap();
                    } else {
                        println!("Unimplemented ah for 0x21: 0x{ah:x}, exiting...");
                        emu.get_data_mut().exited = true;
                        emu.emu_stop().unwrap();
                        return;
                    }
                } else {
                    println!("Unimplemented interrupt 0x{num:x}, exiting...");
                    emu.get_data_mut().exited = true;
                    emu.emu_stop().unwrap();
                }
            })
            .unwrap();

        Self { engine }
    }

    pub fn set_verbose(&mut self, verbose: bool) {
        self.engine.get_data_mut().verbose = verbose;
    }

    pub fn exited(&self) -> bool {
        self.engine.get_data().exited
    }

    pub fn add_break(&mut self, addr: u64) {
        self.engine.get_data_mut().add_break(EngineBreak::new(addr));
    }

    pub fn add_while_break(&mut self, addr: u64) {
        self.engine.get_data_mut().add_break(EngineBreak::new(addr));
        self.engine.get_data_mut().while_break = Some((false, addr))
    }

    pub fn start(&mut self) {
        let ip = FarPointer::read_engine(&self.engine);
        self.engine.emu_start(ip.address(), 8192, 0, 0).unwrap()
    }

    pub fn read_cpu(&self) -> Cpu {
        Cpu::read_engine(&self.engine)
    }

    /// Read two bytes from memory
    pub fn read_mem(&self, addr: u64) -> u16 {
        let mut buf: [u8; 2] = [0; 2];
        self.engine.mem_read(addr, &mut buf).unwrap();
        u16::from_le_bytes(buf)
    }

    /// Continue run where enigne was stopped
    pub fn cont(&mut self) {
        self.start();
    }

    pub fn step(&mut self) {
        let ip = FarPointer::read_engine(&self.engine);
        self.engine.emu_start(ip.address(), 8192, 0, 1).unwrap()
    }
}
