mod chip8;
mod multimedia;

use chip8::Chip8;
use multimedia::Multimedia;

fn main() {
    let args = &std::env::args().collect::<Vec<String>>();

    let rom_file:Option<&str> =
        if args.len() > 1 {
            let rom_file = &args[1];
            Some(rom_file)
        } else if cfg!(debug_assertions) {
            Some("roms/rom.ch8")
        } else {
            None
        };

    match rom_file {
        Some(file) => run_chip8(file),
        None => {
            eprintln!("Chipr-8 needs a ROM to run, please run: \nchipr-8 rom_file.ch8");
            std::process::exit(1);
        }
    }

}

fn run_chip8(rom_file: &str) {
    let multimedia = Multimedia::new();
    let mut chip8 = Chip8::new(multimedia);
    chip8.load_rom(rom_file);

    while chip8.is_on() {
        chip8.execute_cycle();
    }
}
