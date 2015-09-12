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

/// Starts the emulator main loop with a ROM and window scaling. Returns when the user presses ESC.
pub fn start_emulator(rom: Rom, scale: f32) {
    let rom = Box::new(rom);
    println!("Loaded ROM: {}", rom.header);

    let sdl = sdl2::init().unwrap();
    let video = sdl.video().unwrap();
    let audio = sdl.audio().unwrap();
    let mut event_pump = sdl.event_pump().unwrap();
    let mut gfx = Gfx::new(&video, scale);
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

    let mut last_time = time::precise_time_s();
    let mut frames = 0;

    'main: loop {
        cpu.step();

        let ppu_result = cpu.mem.ppu.step(cpu.cy);
        if ppu_result.vblank_nmi {
            cpu.nmi();
        } else if ppu_result.scanline_irq {
            cpu.irq();
        }

        cpu.mem.apu.step(cpu.cy);

        if ppu_result.new_frame {
            gfx.tick();
            gfx.composite(&mut *cpu.mem.ppu.screen);
            record_fps(&mut last_time, &mut frames);
            cpu.mem.apu.play_channels();

            for event in event_pump.poll_iter() {
                use sdl2::event::Event;
                use sdl2::event::WindowEventId;
                use sdl2::keyboard::Keycode;

                match event {
                    Event::KeyDown { keycode: Some(Keycode::Escape), .. }
                    | Event::Quit { .. } => break 'main,

                    Event::KeyDown { keycode: Some(Keycode::S), .. } => {
                        cpu.save(&mut File::create(&Path::new("state.sav")).unwrap());
                        gfx.status_line.set("Saved state".to_string());
                    }
                    Event::KeyDown { keycode: Some(Keycode::L), .. } => {
                        cpu.load(&mut File::open(&Path::new("state.sav")).unwrap());
                        gfx.status_line.set("Loaded state".to_string());
                    }

                    Event::Window {
                        win_event_id: WindowEventId::Resized, data1: w, data2: h, ..
                    } => {
                        gfx.on_window_resize(w as u32, h as u32);
                    }

                    _ => {
                        // Let the input module handle it
                        cpu.mem.input.handle_event(event);
                    }
                }
            }
        }
    }

    audio::close();
}
