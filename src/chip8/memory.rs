use std::ops::{Index, IndexMut};

use super::font::FONT_SET;

pub const MEM_SIZE: usize = 4096;

pub struct Memory {
    memory: [u8; MEM_SIZE],
}


impl Index<usize> for Memory {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        self.check_valid_addr(index);
        &self.memory[index]
    }
}

impl IndexMut<usize> for Memory {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.memory[index]
    }
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            memory: [0; MEM_SIZE],
        }
    }

    pub fn load_rom(&mut self, vec: &Vec<u8>) {
        if vec.len() > (0xFFF - 0x200) {
            panic!("Invalid ROM size {}.", vec.len());
        }
        for i in 0..vec.len() {
            self.memory[0x200 + i] = vec[i];
        }
        self.load_font();
    }

    pub fn dump(&self) {
        println!("{}", "******************************* MEMORY *******************************");
        print!("     ");
        for i in 0..16 {
            print!("\t{:02X}", i);
        }
        println!();
        for i in 0..256 {
            print!("{:04X}:", i * 16);
            for j in 0..16 {
                print!("\t{:02X}", self.memory[i * 16 + j]);
            }
            println!();
        }
    }

    pub fn load_font(&mut self) {
        for i in 0..FONT_SET.len() {
            self.memory[i] = FONT_SET[i];
        }
    }

    pub fn read_u16(&self, addr: usize) -> u16 {
        self.check_valid_addr(addr);
        ((self.memory[addr] as u16) << 8) | self.memory[addr + 1] as u16
    }

    fn check_valid_addr(&self, addr: usize) {
        if addr > 0xFFF {
            self.dump();
            panic!("Invalid memory address {}", addr);
        }
    }
}