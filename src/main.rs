use elf::{ElfBytes, endian::LittleEndian};
use unicorn_engine::{Mode, Prot, RegisterX86, Unicorn};

const ASM_FILE: &'static [u8] = include_bytes!("../asm/hello.out");

fn main() {
    let start = 0x0000000000401000;

    let mut engine = Unicorn::new(unicorn_engine::Arch::X86, Mode::MODE_64).unwrap();
    engine.mem_map(0, 8 * 1024 * 1024, Prot::ALL).unwrap();
    let file = ElfBytes::<LittleEndian>::minimal_parse(ASM_FILE).unwrap();

    for section in file.section_headers().unwrap() {
        if section.sh_addr > 0 {
            let offset = section.sh_offset as usize;
            engine
                .mem_write(
                    section.sh_addr,
                    &ASM_FILE[offset..offset + section.sh_size as usize],
                )
                .unwrap();
        }
    }

    engine
        .add_insn_sys_hook(unicorn_engine::X86Insn::SYSCALL, start, 0, |emu| {
            let syscall = emu.reg_read(RegisterX86::RAX).unwrap();
            if syscall == 60 {
                emu.emu_stop().unwrap();
                println!("exit captured, stopping emulation");
            }
        })
        .unwrap();

    engine
        .emu_start(start, ASM_FILE.len() as u64, 0, 0)
        .unwrap();
}
