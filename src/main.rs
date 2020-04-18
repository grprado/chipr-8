mod chip8;
mod multimedia;

use chip8::Chip8;
use multimedia::Multimedia;

fn main() {
    let multimedia = Multimedia::new();
    let mut chip8 = Chip8::new(multimedia);


    let args = &std::env::args().collect::<Vec<String>>();
    if args.len() > 1 {
        let rom_file = &args[1];
        chip8.load_rom(rom_file);
    } else {
        chip8.load_rom("roms/Pong (alt).ch8");
    }
    while chip8.is_on(){
        chip8.execute_cycle();
    }

}
