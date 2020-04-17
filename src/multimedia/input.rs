use sdl2::{EventPump, Sdl};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

pub struct Input {
    event_pump: EventPump,
    keys: [bool; 0xF],
    is_quiting: bool,
}

pub trait EventManager {
    fn check_events(&mut self);
    fn is_quiting(&self) -> bool;
    fn is_key_pressed(&self, key: u8) -> bool;
}

impl EventManager for Input {
    fn check_events(&mut self) {
        self.keys = [false; 0xF];

        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => self.is_quiting = true,
                _ => {}
            }
        }

        let keys: Vec<Keycode> = self.event_pump.keyboard_state().pressed_scancodes().filter_map(Keycode::from_scancode).collect();

        for key in keys {
            let op_i = match key {
                Keycode::Num1 => Some(0x1),
                Keycode::Num2 => Some(0x2),
                Keycode::Num3 => Some(0x3),
                Keycode::Num4 => Some(0xc),
                Keycode::Q => Some(0x4),
                Keycode::W => Some(0x5),
                Keycode::E => Some(0x6),
                Keycode::R => Some(0xd),
                Keycode::A => Some(0x7),
                Keycode::S => Some(0x8),
                Keycode::D => Some(0x9),
                Keycode::F => Some(0xe),
                Keycode::Z => Some(0xa),
                Keycode::X => Some(0x0),
                Keycode::C => Some(0xb),
                Keycode::V => Some(0xf),
                _ => None,
            };

            if let Some(i) = op_i {
                self.keys[i] = true;
            }
        }
    }

    fn is_quiting(&self) -> bool {
        self.is_quiting
    }

    fn is_key_pressed(&self, key: u8) -> bool {
        self.keys[key as usize]
    }
}

impl Input {
    pub fn new(sdl_context: &Sdl) -> Input {
        let event_pump = sdl_context.event_pump().unwrap();
        Input {
            event_pump,
            keys: [false; 0xF],
            is_quiting: false,
        }
    }
}