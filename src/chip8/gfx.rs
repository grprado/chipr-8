use std::ops::{Index, IndexMut};

pub const GFX_COLS: usize = 64;
pub const GFX_ROWS: usize = 32;
pub const GFX_MEM_SIZE: usize = GFX_COLS * GFX_ROWS;

pub struct Gfx {
    gfx: [bool; GFX_MEM_SIZE],
    needs_redraw: bool
}

impl Index<usize> for Gfx {
    type Output = bool;

    fn index(&self, index: usize) -> &Self::Output {
        &self.gfx[index]
    }
}

impl IndexMut<usize> for Gfx {

    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.needs_redraw = true;
        &mut self.gfx[index]
    }
}


impl Gfx {
    pub fn new() -> Gfx {
        Gfx {
            gfx: [false; GFX_MEM_SIZE],
            needs_redraw: false
        }
    }

    pub fn clear(&mut self) {
        self.gfx = [false; GFX_MEM_SIZE];
        self.needs_redraw = true;
    }

    pub fn dump(&self) {
        println!("{}", "****************************************   GFX  *********************************************");
        print!("    ");
        for i in 0..64 {
            print!("{}", i / 10);
        }
        println!();
        print!("    ");
        for i in 0..64 {
            print!("{}", i % 10);
        }
        println!();
        for i in 0..32 {
            print!("{:02}: ", i);
            for j in 0..64 {
                if self.gfx[i * 64 + j] {
                    print!("{}", "█");
                } else {
                    print!("{}", " ");
                }
            }
            println!();
        }
    }

    pub fn needs_redraw(&self) -> bool {
        self.needs_redraw
    }

    pub fn set_needs_redraw(&mut self, b: bool) {
        self.needs_redraw = b;
    }
}