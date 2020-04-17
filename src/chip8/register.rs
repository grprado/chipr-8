const REGISTERS_SIZE: usize = 16;

use std::ops::{Index, IndexMut};

pub struct Registers {
    v: [u8; REGISTERS_SIZE],
}

impl Index<usize> for Registers {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self.v[index]
    }
}

impl IndexMut<usize> for Registers {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.v[index]
    }
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            v: [0; REGISTERS_SIZE]
        }
    }

    pub fn dump(&self) {
        for i in 0..2 {
            print!("  V{:X}: 0x{:02X}", i * 8 + 0, self.v[i * 8 + 0]);
            print!("         V{:X}: 0x{:02X}", i * 8 + 1, self.v[i * 8 + 1]);
            print!("         V{:X}: 0x{:02X}", i * 8 + 2, self.v[i * 8 + 2]);
            print!("         V{:X}: 0x{:02X}", i * 8 + 3, self.v[i * 8 + 3]);
            print!("         V{:X}: 0x{:02X}", i * 8 + 4, self.v[i * 8 + 4]);
            print!("         V{:X}: 0x{:02X}", i * 8 + 5, self.v[i * 8 + 5]);
            print!("         V{:X}: 0x{:02X}", i * 8 + 6, self.v[i * 8 + 6]);
            println!("         V{:X}: 0x{:02X}", i * 8 + 7, self.v[i * 8 + 7]);
        }
    }

}