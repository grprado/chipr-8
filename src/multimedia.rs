
use crate::chip8::gfx::Gfx;
use crate::multimedia::input::{EventManager, Input};
use crate::multimedia::screen::{Drawable, Screen};
use crate::multimedia::sound::{Beeper, Sound};

pub mod screen;
pub mod sound;
pub mod input;


pub struct Multimedia {
    screen: Screen,
    sound: Sound,
    input: Input
}

impl Drawable for Multimedia {
    fn draw(&mut self, gfx: &mut Gfx) {
        self.screen.draw(gfx);
    }
}
impl Beeper for Multimedia {
    fn start_beep(&mut self) {
        self.sound.start_beep();
    }

    fn stop_beep(&mut self) {
        self.sound.stop_beep();
    }
}

impl EventManager for Multimedia {
    fn check_events(&mut self) {
        self.input.check_events();
    }

    fn is_quiting(&self) -> bool {
        self.input.is_quiting()
    }

    fn is_key_pressed(&self, key: u8) -> bool {
        self.input.is_key_pressed(key)
    }
}
impl Multimedia {
    pub fn new() -> Multimedia {
        let sdl_context = sdl2::init().unwrap();
        let screen = Screen::new(&sdl_context);
        let sound = Sound::new(&sdl_context);
        let input = Input::new(&sdl_context);

        Multimedia {
            screen,
            sound,
            input
        }
    }

}