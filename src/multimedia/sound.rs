use sdl2::Sdl;
use sdl2::audio::{AudioSpecDesired, AudioDevice, AudioCallback};

pub struct Sound {
    device: AudioDevice<Wave>,
    playing: bool
}

struct Wave {
    phase_inc: f32,
    phase: f32,
    volume: f32
}

impl AudioCallback for Wave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        // Generate a square wave
        for x in out.iter_mut() {
            if self.phase <= 0.5 {
                *x = self.volume;
            } else {
                *x = -self.volume;
            }
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}


pub trait Beeper {
    fn start_beep(&mut self);
    fn stop_beep(&mut self);
}

impl Beeper for Sound {
    fn start_beep(&mut self) {
        if !self.playing {
            self.device.resume();
            self.playing = true;
        }
    }

    fn stop_beep(&mut self) {
        if self.playing {
            self.device.pause();
            self.playing = false;
        }
    }
}

impl Sound {
    pub fn new(sdl_context: &Sdl) -> Sound {
        let audio_subsystem = sdl_context.audio().unwrap();
        let desired_spec = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(1),  // mono
            samples: None       // default sample size
        };

        let device = audio_subsystem.open_playback(None, &desired_spec, |spec| {
            // initialize the audio callback
            Wave {
                phase_inc: 440.0 / spec.freq as f32,
                phase: 0.0,
                volume: 0.25
            }
        }).unwrap();
        Sound {
            device,
            playing: false
        }
    }
}