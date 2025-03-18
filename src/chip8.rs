use std::fs;
use std::time;
use std::time::Duration;

use rand::prelude::*;

use gfx::Gfx;
use memory::Memory;
use register::Registers;

use crate::multimedia::Multimedia;
use crate::multimedia::screen::Drawable;
use crate::multimedia::sound::Beeper;
use crate::multimedia::input::EventManager;
use std::rc::Rc;
use std::cell::RefCell;


mod memory;
pub mod gfx;
mod register;
mod font;

const STACK_SIZE: usize = 16;

const PC_START_ADDR: u16 = 0x200;

const MIN_CYCLE_DURATION_MICROS: u64 = 1666;

pub struct Chip8 {
    opcode: u16,
    v: Registers,
    i: u16,
    pc: u16,
    stack: [u16; STACK_SIZE],
    sp: usize,
    memory: Memory,
    gfx: Gfx,
    delay_timer: u8,
    sound_timer: u8,
    time: time::Instant,
    delta: Duration,
    timer_delta: Duration,
    // This is here just to play a bit with Rc and RefCell
    // Looks ugly as hell.
    drawable: Rc<RefCell<dyn Drawable>>,
    beeper: Rc<RefCell<dyn Beeper>>,
    event_manager: Rc<RefCell<dyn EventManager>>,
    is_on: bool
}

const CLOCK_60_HZ: Duration = Duration::from_micros(16666);
const DEFAULT_SLEEP_DURATION: Duration = Duration::from_micros(10);

impl Chip8 {

    pub fn new(multimedia :Multimedia) -> Chip8 {
        let mm = Rc::new(RefCell::new(multimedia));
        Chip8 {
            opcode: 0,
            v: Registers::new(),
            i: 0,
            pc: PC_START_ADDR,
            sp: 0,
            stack: [0; STACK_SIZE],
            memory: Memory::new(),
            gfx: Gfx::new(),
            delay_timer: 0,
            sound_timer: 0,
            time: time::Instant::now(),
            delta: Duration::from_millis(0),
            timer_delta: Duration::from_millis(0),
            // see comment in struct
            drawable: Rc::clone(&mm) as Rc<RefCell<dyn Drawable>>,
            beeper: Rc::clone(&mm) as Rc<RefCell<dyn Beeper>>,
            event_manager: mm as Rc<RefCell<dyn EventManager>>,
            is_on: true
        }
    }

    pub fn shutdown(&mut self) {
        self.is_on = false;
    }

    pub fn is_on(&self) -> bool {
        self.is_on
    }

    pub fn load_rom(&mut self, file_path: &str) {
        let file = fs::read(file_path).expect("Invalid ROM file");
        self.memory.load_rom(&file);
    }

    pub fn dump_stack(&self) {
        println!("  SP: 0x{:02X}", self.sp);
        for i in 0..(STACK_SIZE / 8) {
            print!("  S{:X}: 0x{:02X}", i * 8 + 0, self.stack[i * 8 + 0]);
            print!("         S{:X}: 0x{:02X}", i * 8 + 1, self.stack[i * 8 + 1]);
            print!("         S{:X}: 0x{:02X}", i * 8 + 2, self.stack[i * 8 + 2]);
            print!("         S{:X}: 0x{:02X}", i * 8 + 3, self.stack[i * 8 + 3]);
            print!("         S{:X}: 0x{:02X}", i * 8 + 4, self.stack[i * 8 + 4]);
            print!("         S{:X}: 0x{:02X}", i * 8 + 5, self.stack[i * 8 + 5]);
            print!("         S{:X}: 0x{:02X}", i * 8 + 6, self.stack[i * 8 + 6]);
            println!("         S{:X}: 0x{:02X}", i * 8 + 7, self.stack[i * 8 + 7]);
        }
    }

    /// Dumps processor and memory to console
    pub fn dump(&self) {
        println!(" OPC: 0x{:04X}", self.opcode);
        println!("  PC: 0x{:04X}", self.pc);
        println!("   I: 0x{:04X}", self.i);
        self.v.dump();
        self.dump_stack();
        println!("  DT: 0x{:1X}", self.delay_timer);
        println!("  ST: 0x{:1X}", self.sound_timer);

        println!();

        self.gfx.dump();
        println!();
        self.memory.dump();
    }

    pub fn execute_cycle(&mut self) {
        self.calculate_delta();
        self.run_multimedia();

        if self.hit_min_delta_duration() {
            self.fetch();
            self.execute();
            self.pc += 2;
        } else {
            std::thread::sleep(DEFAULT_SLEEP_DURATION);
        }

    }

    fn execute(&mut self) {
        let op = self.opcode & 0xF000;

        match op {
            0x0000 => {
                match self.opcode {
                    0x00E0 => self.clear_gfx(),
                    0x00EE => self.unstack(),
                    _ => panic!("code {:X} not implemented", self.opcode)
                }
            }
            0x1000 => self.goto(),
            0x2000 => self.subroutine(),
            0x3000 => self.jmp_eq(),
            0x4000 => self.jmp_neq(),
            0x5000 => self.jmp_vx_eq_vy(),
            0x6000 => self.set_register(),
            0x7000 => self.add_nn_vx(),
            0x8000 => match self.opcode & 0x000F {
                0x0 => self.set_vx_vy(),
                0x1 => self.or_vx_vy(),
                0x2 => self.and_vx_vy(),
                0x3 => self.xor_vx_vy(),
                0x4 => self.add_vx_vy(),
                0x5 => self.sub_vx_vy(),
                0x6 => self.shift_r(),
                0x7 => self.sub_vy_vx(),
                0xE => self.shift_l(),
                _ => panic!("code {:X} - {:X} not implemented", op, self.opcode)
            }
            0x9000 => self.jmp_vx_neq_vy(),
            0xA000 => self.set_i_nnn(),
            0xB000 => self.jmp_nnn(),
            0xC000 => self.rand(),
            0xD000 => self.draw(),
            0xE000 => match self.opcode & 0x00FF {
                0x9E => self.skip_if_pressed(),
                0xA1 => self.skip_not_pressed(),
                _ => panic!("code {:X} - {:X} not implemented", op, self.opcode)
            }
            0xF000 => match self.opcode & 0x00FF {
                0x07 => self.vx_get_delay(),
                0x0A => self.get_key(),
                0x15 => self.set_delay_timer(),
                0x18 => self.set_sound_timer(),
                0x1E => self.add_vx_i(),
                0x29 => self.i_sprite_loc(),
                0x33 => self.bin_dec_vx(),
                0x55 => self.reg_dump(),
                0x65 => self.reg_load(),
                _ => panic!("code {:X} - {:X} not implemented", op, self.opcode)
            }

            _ => {
                self.dump();
                panic!("code {:X} - {:X} not implemented", op, self.opcode)
            }
        }
    }

    fn fetch(&mut self) {
        self.opcode = self.memory.read_u16(self.pc as usize);
    }

    fn run_multimedia(&mut self) {
        if self.timer_delta > CLOCK_60_HZ {
            if self.delay_timer > 0 {
                self.delay_timer -= 1;
            }
            if self.sound_timer > 0 {
                if self.sound_timer > 1 {
                    self.beeper.borrow_mut().start_beep();
                } else {
                    self.beeper.borrow_mut().stop_beep();
                }
                self.sound_timer -= 1;
            }

            self.draw_and_check_events();

            self.timer_delta = Duration::from_micros((self.timer_delta.as_micros() % CLOCK_60_HZ.as_micros()) as u64);
        }
    }

    fn draw_and_check_events(&mut self) {
        self.drawable.borrow_mut().draw(&mut self.gfx);
        self.event_manager.borrow_mut().check_events();
        if self.event_manager.borrow_mut().is_quiting() {
            self.shutdown();
        }
    }

    fn calculate_delta(&mut self) {
        let now = time::Instant::now();
        let delta = now.duration_since(self.time);
        self.time = now;
        self.delta += delta;
        self.timer_delta += delta;
    }

    fn hit_min_delta_duration(&mut self) -> bool {
        if self.delta > Duration::from_micros(MIN_CYCLE_DURATION_MICROS) {
            self.delta = Duration::from_nanos(0);
            return true;
        }
        false
    }

    ///00E0
    ///
    /// Clears the screen.
    ///
    fn clear_gfx(&mut self) {
        self.gfx.clear();
    }

    /// 00EE<br>
    /// return<br>
    ///	Returns from a subroutine.
    fn unstack(&mut self) {
        self.sp -= 1;
        self.pc = self.stack[self.sp];
        self.stack[self.sp] = 0;
    }


    /// 1NNN<br>
    ///	goto NNN<br>
    /// Jumps to address NNN.
    fn goto(&mut self) {
        self.pc = (self.opcode & 0x0FFF) - 2;
    }

    /// 2NNN<br>
    ///	*(0xNNN)()<br>
    ///	Calls subroutine at NNN.
    fn subroutine(&mut self) {
        self.stack[self.sp] = self.pc;
        self.sp += 1;
        self.pc = (self.opcode & 0x0FFF) - 2;
    }

    /// 3XNN
    ///
    /// if(Vx==NN)
    ///
    /// Skips the next instruction if VX equals NN. (Usually the next instruction is a jump to skip a code block)
    fn jmp_eq(&mut self) {
        let nn = (self.opcode & 0x00FF) as u8;
        let r = (self.opcode & 0x0F00) as usize >> 8;
        let vx = self.v[r];
        if vx == nn {
            self.pc += 2;
        }
    }

    /// 4XNN
    ///
    /// if(Vx!=NN)
    ///
    /// Skips the next instruction if VX doesn't equal NN. (Usually the next instruction is a jump to skip a code block)
    fn jmp_neq(&mut self) {
        let nn = (self.opcode & 0x00FF) as u8;
        let r = (self.opcode & 0x0F00) as usize >> 8;
        let vx = self.v[r];
        if vx != nn {
            self.pc += 2;
        }
    }

    /// 5XY0
    ///
    /// if(Vx==Vy)
    ///
    /// Skips the next instruction if VX equals VY. (Usually the next instruction is a jump to skip a code block)
    fn jmp_vx_eq_vy(&mut self) {
        let x = (self.opcode & 0x0F00) as usize >> 8;
        let y = (self.opcode & 0x00F0) as usize >> 4;
        if self.v[x] == self.v[y] {
            self.pc += 2;
        }
    }

    /// 6XNN<br>
    /// Vx = NN<br>
    ///	Sets VX to NN.
    fn set_register(&mut self) {
        let register = ((self.opcode & 0x0F00) >> 8) as usize;
        let val = (self.opcode & 0x00FF) as u8;
        self.v[register] = val;
    }

    /// 7XNN<br>
    ///	Vx += NN<br>
    ///	Adds NN to VX. (Carry flag is not changed)
    fn add_nn_vx(&mut self) {
        let r = (self.opcode & 0x0F00) as usize >> 8;
        let result = self.v[r] as u16 + (self.opcode & 0x00FF);
        self.v[r] = (result & 0xFF) as u8;
    }

    /// 8XY0
    ///
    /// Vx=Vy
    ///
    /// Sets VX to the value of VY.
    fn set_vx_vy(&mut self) {
        let x = (self.opcode & 0x0F00) as usize >> 8;
        let y = (self.opcode & 0x00F0) as usize >> 4;
        self.v[x] = self.v[y];
    }

    /// 8XY1
    ///
    /// Vx=Vx|Vy
    ///
    /// Sets VX to VX or VY. (Bitwise OR operation)
    fn or_vx_vy(&mut self) {
        let x = (self.opcode & 0x0F00) as usize >> 8;
        let y = (self.opcode & 0x00F0) as usize >> 4;
        self.v[x] |= self.v[y];
    }

    /// 8XY2
    ///
    /// Vx=Vx&Vy
    ///
    /// Sets VX to VX and VY. (Bitwise AND operation)
    fn and_vx_vy(&mut self) {
        let x = (self.opcode & 0x0F00) as usize >> 8;
        let y = (self.opcode & 0x00F0) as usize >> 4;
        self.v[x] &= self.v[y];
    }

    /// 8XY3
    ///
    /// Vx=Vx^Vy
    ///
    /// Sets VX to VX xor VY.
    fn xor_vx_vy(&mut self) {
        let x = (self.opcode & 0x0F00) as usize >> 8;
        let y = (self.opcode & 0x00F0) as usize >> 4;
        self.v[x] ^= self.v[y];
    }

    /// 8XY4
    ///
    /// Vx += Vy
    ///
    /// Adds VY to VX. VF is set to 1 when there's a carry, and to 0 when there isn't.
    fn add_vx_vy(&mut self) {
        let x = (self.opcode & 0x0F00) as usize >> 8;
        let y = (self.opcode & 0x00F0) as usize >> 4;
        let result = self.v[x] as u16 + self.v[y] as u16;
        self.v[x] = (result & 0xFF) as u8;
        self.v[0xF] = (result >> 8) as u8;
    }

    /// 8XY5
    ///
    /// Vx -= Vy
    ///
    /// VY is subtracted from VX. VF is set to 0 when there's a borrow, and 1 when there isn't.
    fn sub_vx_vy(&mut self) {
        let x = (self.opcode & 0x0F00) as usize >> 8;
        let y = (self.opcode & 0x00F0) as usize >> 4;

        if self.v[x] >= self.v[y] {
            self.v[x] -= self.v[y];
            self.v[0xF] = 1;
        } else {
            let result = 256u16 + self.v[x] as u16 - self.v[y] as u16;
            self.v[x] = result as u8;
            self.v[0xF] = 0;
        }
    }

    /// 8XY6
    ///
    /// Vx>>=1
    ///
    /// Stores the least significant bit of VX in VF and then shifts VX to the right by 1.
    fn shift_r(&mut self) {
        let register = ((self.opcode & 0x0F00) >> 8) as usize;
        self.v[0xF] = self.v[register] & 0b1;
        self.v[register] = self.v[register] >> 1;
    }

    /// 8XY7
    ///
    /// Vx = Vy - Vx
    ///
    /// Sets VX to VY minus VX. VF is set to 0 when there's an underflow, and 1 when there is not. (i.e. VF set to 1 if VY >= VX).
    fn sub_vy_vx(&mut self) {
        let x = (self.opcode & 0x0F00) as usize >> 8;
        let y = (self.opcode & 0x00F0) as usize >> 4;

        if self.v[x] <= self.v[y] {
            self.v[x] = self.v[y] - self.v[x];
            self.v[0xF] = 1;
        } else {
            let result = 256u16 + self.v[y] as u16 - self.v[x] as u16;
            self.v[x] = result as u8;
            self.v[0xF] = 0;
        }
    }

    /// 8XYE
    ///
    /// Vx<<=1
    ///
    /// Stores the most significant bit of VX in VF and then shifts VX to the left by 1
    fn shift_l(&mut self) {
        let register = ((self.opcode & 0x0F00) >> 8) as usize;
        self.v[0xF] = self.v[register] & 0b1;
        self.v[register] = self.v[register] << 1;
    }

    /// 9XY0
    ///
    /// if(Vx!=Vy)
    ///
    ///	Skips the next instruction if VX doesn't equal VY. (Usually the next instruction is a jump to skip a code block)
    fn jmp_vx_neq_vy(&mut self) {
        let x = (self.opcode & 0x0F00) as usize >> 8;
        let y = (self.opcode & 0x00F0) as usize >> 4;
        if self.v[x] != self.v[y] {
            self.pc += 2;
        }
    }

    /// ANNN<br>
    ///	I = NNN<br>
    ///	Sets I to the address NNN.
    fn set_i_nnn(&mut self) {
        self.i = self.opcode & 0x0FFF;
    }

    fn jmp_nnn(&mut self) {
        self.pc = (self.opcode & 0x0FFF) + self.v[0] as u16;
    }

    /// CXNN
    ///
    /// Vx=rand()&NN
    ///
    /// Sets VX to the result of a bitwise and operation on a random number (Typically: 0 to 255) and NN.
    fn rand(&mut self) {
        let x = (self.opcode & 0x0F00) as usize >> 8;
        let nn = (self.opcode & 0x00FF) as u8;
        let r: u8 = thread_rng().gen();
        self.v[x] = r & nn;
    }

    /// DXYN
    /// Disp
    /// draw(Vx,Vy,N)
    /// Draws a sprite at coordinate (VX, VY) that has a width of 8 pixels and a height of N pixels. Each row of 8 pixels is read as bit-coded starting from memory location I;
    /// I value doesn’t change after the execution of this instruction. As described above,
    /// VF is set to 1 if any screen pixels are flipped from set to unset when the sprite is drawn,
    /// and to 0 if that doesn’t happen
    fn draw(&mut self) {
        let x = self.v[(self.opcode & 0x0F00) as usize >> 8] as usize;
        let y = self.v[(self.opcode & 0x00F0) as usize >> 4] as usize;
        let n = (self.opcode & 0x000F) as usize;

        self.v[0xF] = 0;

        for line in 0..n {
            let pixels = self.memory[self.i as usize + line];
            for col in 0..8 {
                let pixel = (pixels & (0x80 >> col)) > 0;
                if pixel {
                    let index = (x + col as usize + (line + y) * gfx::GFX_COLS) % 2048;
                    if self.v[0xF] == 0 && self.gfx[index] == pixel {
                        self.v[0xF] = 1;
                    }
                    self.gfx[index] ^= pixel;
                }
            }
        }
    }

    /// EX9E
    ///
    /// Skips the next instruction if the key stored in VX is pressed. (Usually the next instruction is a jump to skip a code block)
    fn skip_if_pressed(&mut self) {
        let x = (self.opcode & 0x0F00) as usize >> 8;
        if self.event_manager.borrow().is_key_pressed(self.v[x]) {
            self.pc += 2;
        }
    }

    /// EXA1
    ///
    /// Skips the next instruction if the key stored in VX isn't pressed. (Usually the next instruction is a jump to skip a code block)
    fn skip_not_pressed(&mut self) {
        let x = (self.opcode & 0x0F00) as usize >> 8;
        if !self.event_manager.borrow().is_key_pressed(self.v[x]) {
            self.pc += 2;
        }
    }

    /// FX07
    ///
    /// Vx = get_delay()
    ///
    /// Sets VX to the value of the delay timer.
    fn vx_get_delay(&mut self) {
        let x = (self.opcode & 0x0F00) as usize >> 8;
        self.v[x] = self.delay_timer;
    }

    /// FX0A
    ///
    /// A key press is awaited, and then stored in VX.
    /// (Blocking Operation. All instruction halted until next key event)
    ///
    fn get_key(&mut self) {
        let x = (self.opcode & 0x0F00) as usize >> 8;
        let mut waiting = true;
        while self.is_on && waiting {
            self.draw_and_check_events();
            for i in 0..0xF {
                if self.event_manager.borrow().is_key_pressed(i) {
                    self.v[x] = i;
                    waiting = false;
                    break
                }
            }
            if waiting {
                std::thread::sleep(DEFAULT_SLEEP_DURATION);
            }
        }
        self.v[x] = thread_rng().gen()
    }

    /// FX15
    ///
    /// delay_timer(Vx)
    ///
    /// Sets the delay timer to VX.
    fn set_delay_timer(&mut self) {
        let x = (self.opcode & 0x0F00) as usize >> 8;
        self.delay_timer = self.v[x];
    }

    /// FX18
    ///
    /// Sets the sound timer to VX.
    fn set_sound_timer(&mut self) {
        let x = (self.opcode & 0x0F00) as usize >> 8;
        self.sound_timer = self.v[x];
    }


    /// FX1E
    /// MEM
    /// I +=Vx
    /// Adds VX to I.
    /// VF is set to 1 when there is a range overflow (I+VX>0xFFF), and to 0 when there isn't.
    fn add_vx_i(&mut self) {
        self.i = self.v[(self.opcode & 0x0F00) as usize >> 8] as u16 + self.i;
        if self.i > 0x0FFF {
            self.v[0xF] = 1;
        }
    }

    /// FX29
    ///
    /// Sets I to the location of the sprite for the character in VX.
    ///
    /// Characters 0-F (in hexadecimal) are represented by a 4x5 font.
    fn i_sprite_loc(&mut self) {
        self.i = self.v[(self.opcode & 0x0F00) as usize >> 8] as u16 * 5;
    }

    /// FX33
    ///
    /// Stores the binary-coded decimal representation of VX, with the most significant of
    /// three digits at the address in I, the middle digit at I plus 1, and the least significant
    /// digit at I plus 2.
    ///
    /// (In other words, take the decimal representation of VX, place the hundreds digit in memory
    /// at location in I, the tens digit at location I+1, and the ones digit at location I+2.)
    fn bin_dec_vx(&mut self) {
        let vx = self.v[((self.opcode & 0x0F00) >> 8) as usize];
        let d100 = vx / 100;
        let d10 = vx % 100 / 10;
        let d1 = vx % 10;

        self.memory[self.i as usize] = d100;
        self.memory[self.i as usize + 1] = d10;
        self.memory[self.i as usize + 2] = d1;
    }

    /// FX55
    ///
    /// reg_dump(Vx,&I)
    ///
    /// Stores V0 to VX (including VX) in memory starting at address I. The offset from I is increased by 1 for each value written, but I itself is left unmodified.
    fn reg_dump(&mut self) {
        let reg = ((self.opcode & 0x0F00) >> 8) as usize;
        for i in 0..reg + 1 {
            self.memory[self.i as usize + i] = self.v[i];
        }
    }

    /// FX65
    ///
    /// reg_load(Vx,&I)
    ///
    /// Fills V0 to VX (including VX) with values from memory starting at address I.
    /// The offset from I is increased by 1 for each value written, but I itself is left unmodified.
    fn reg_load(&mut self) {
        let reg = ((self.opcode & 0x0F00) >> 8) as usize;
        for i in 0..reg + 1 {
            self.v[i] = self.memory[self.i as usize + i];
        }
    }
}