use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::chip8::gfx::{Gfx, GFX_COLS, GFX_MEM_SIZE, GFX_ROWS};
use sdl2::Sdl;

const SCALE: u32 = 12;

pub struct Screen {
    canvas: Canvas<Window>,
}

pub trait Drawable {
    fn draw(&mut self, gfx: &mut Gfx);
}

impl Drawable for Screen {
    fn draw(&mut self, gfx: &mut Gfx) {
        if gfx.needs_redraw() {
            let canvas = &mut self.canvas;
            canvas.set_draw_color(Color::RGB(0, 0, 0));
            canvas.clear();
            canvas.set_draw_color(Color::RGB(65, 255, 0));

            let mut rect = Rect::new(0, 0, SCALE as u32, SCALE as u32);
            for i in 0..GFX_MEM_SIZE {
                if gfx[i] {
                    let x = (i % GFX_COLS) as i32 * SCALE as i32;
                    let y = (i / GFX_COLS) as i32 * SCALE as i32;
                    rect.set_x(x);
                    rect.set_y(y);
                    canvas.fill_rect(rect).expect("Could not draw rect");
                }
            }
            canvas.present();
            gfx.set_needs_redraw(false);
        }
    }
}

impl Screen {
    pub fn new(sdl_context: &Sdl) -> Screen {
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem.window("Chipr-8 - CHIP-8 Emulator", GFX_COLS as u32 * SCALE, GFX_ROWS as u32 * SCALE)
            .position_centered()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().present_vsync().build().unwrap();
        canvas.clear();
        canvas.present();

        Screen {
            canvas,
        }
    }
}