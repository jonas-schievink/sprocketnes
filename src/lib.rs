//
// Author: Patrick Walton
//

#[macro_use]
extern crate lazy_static;
extern crate libc;
extern crate sdl2;
extern crate time;

// NB: This must be first to pick up the macro definitions. What a botch.
#[macro_use]
pub mod util;

pub mod apu;
pub mod audio;
#[macro_use]
pub mod cpu;
pub mod disasm;
pub mod gfx;
pub mod input;
pub mod mapper;
pub mod mem;
pub mod ppu;
pub mod rom;
pub mod resampler;

use apu::Apu;
use cpu::Cpu;
use gfx::Gfx;
use input::Input;
use mapper::Mapper;
use mem::MemMap;
use ppu::{Oam, Ppu, Vram};
use rom::Rom;
use util::Save;

use sdl2::EventPump;

use std::cell::RefCell;
use std::fs::File;
use std::path::Path;
use std::rc::Rc;

fn record_fps(last_time: &mut f64, frames: &mut usize) {
    if cfg!(debug) {
        let now = time::precise_time_s();
        if now >= *last_time + 1f64 {
            println!("{} FPS", *frames);
            *frames = 0;
            *last_time = now;
        } else {
            *frames += 1;
        }
    }
}

pub struct Emulator {
    cpu: Cpu<MemMap>,
    gfx: Gfx<'static>,
    event_pump: EventPump,
    pub mute: bool,
}

impl Emulator {
    /// Creates a new emulator and window
    pub fn new(rom: Rom, scale: f32) -> Emulator {
        let rom = Box::new(rom);
        println!("Loaded ROM: {}", rom.header);

        let sdl = sdl2::init().unwrap();
        let video = sdl.video().unwrap();
        let audio = sdl.audio().unwrap();
        let event_pump = sdl.event_pump().unwrap();
        let gfx = Gfx::new(&video, scale);
        let audio_buffer = audio::open(&audio);

        let mapper: Box<Mapper+Send> = mapper::create_mapper(rom);
        let mapper = Rc::new(RefCell::new(mapper));
        let ppu = Ppu::new(Vram::new(mapper.clone()), Oam::new());
        let input = Input::new();
        let apu = Apu::new(audio_buffer);
        let memmap = MemMap::new(ppu, input, mapper, apu);
        let mut cpu = Cpu::new(memmap);

        // TODO: Add a flag to not reset for nestest.log
        cpu.reset();

        Emulator {
            cpu: cpu,
            gfx: gfx,
            event_pump: event_pump,
            mute: false,
        }
    }

    /// Starts the emulator main loop. Returns when the user presses escape or the window is
    /// closed.
    pub fn start(&mut self) {
        let mut last_time = time::precise_time_s();
        let mut frames = 0;

        'main: loop {
            self.cpu.step();

            let ppu_result = self.cpu.mem.ppu.step(self.cpu.cy);
            if ppu_result.vblank_nmi {
                self.cpu.nmi();
            } else if ppu_result.scanline_irq {
                self.cpu.irq();
            }

            self.cpu.mem.apu.step(self.cpu.cy);

            if ppu_result.new_frame {
                self.gfx.tick();
                self.gfx.composite(&mut self.cpu.mem.ppu.screen);
                record_fps(&mut last_time, &mut frames);
                self.cpu.mem.apu.play_channels(self.mute);

                for event in self.event_pump.poll_iter() {
                    use sdl2::event::Event;
                    use sdl2::event::WindowEventId;
                    use sdl2::keyboard::Keycode;

                    match event {
                        Event::KeyDown { keycode: Some(Keycode::Escape), .. }
                        | Event::Quit { .. } => break 'main,

                        Event::KeyDown { keycode: Some(Keycode::S), .. } => {
                            self.cpu.save(&mut File::create(&Path::new("state.sav")).unwrap());
                            self.gfx.status_line.set("Saved state".to_owned());
                        }
                        Event::KeyDown { keycode: Some(Keycode::L), .. } => {
                            self.cpu.load(&mut File::open(&Path::new("state.sav")).unwrap());
                            self.gfx.status_line.set("Loaded state".to_owned());
                        }
                        Event::KeyDown { keycode: Some(Keycode::M), .. } => {
                            self.mute = !self.mute;
                        }
                        #[cfg(feature = "cpuspew")]
                        Event::KeyDown { keycode: Some(Keycode::T), .. } => {
                            self.cpu.trace = !self.cpu.trace;
                        }

                        Event::Window {
                            win_event_id: WindowEventId::Resized, data1: w, data2: h, ..
                        } => {
                            self.gfx.on_window_resize(w as u32, h as u32);
                        }

                        _ => {
                            // Let the input module handle it
                            self.cpu.mem.input.handle_event(event);
                        }
                    }
                }
            }
        }

        audio::close();
    }
}
