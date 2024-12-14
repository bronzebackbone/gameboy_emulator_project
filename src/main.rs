pub mod bus;
pub mod cpu;
pub mod cartridge;
pub mod ppu;
pub mod dma;
pub mod timer;
pub mod apu;
use cartridge::Cartridge;
use bus::Bus;
use cpu::Cpu;
use ppu::Ppu;
use dma::Dma;


use sdl2::pixels::{PixelFormatEnum};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::audio::{AudioSpecDesired};

use std::env;
use std::fs;
use std::time::{Duration,Instant};
use std::thread;
fn main() {
    let args: Vec<String> = env::args().collect();
    let boot_rom = fs::read("dmg_boot.bin").unwrap();
    let cart_rom = fs::read(&args[1]).unwrap();
    let debugmode = if args.len() < 3 {
        false
    }else {
        args[2].parse::<bool>().unwrap_or(false)
    };
    let cart = Cartridge::new(cart_rom, boot_rom);
    let mut bus = Bus::new(cart);
    let mut cpu = Cpu::new();
    let mut ppu = Ppu::new();
    let mut dma = Dma::new();

    if debugmode {
        bus.after_bootup();
        cpu.after_bootup();
    }

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem.window("quarrygb",4*160, 4*144)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    let creator = canvas.texture_creator();
    let mut texture = creator.create_texture_target(PixelFormatEnum::RGB24, 160, 144).unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();
    let specs = AudioSpecDesired{
        freq: Some(22_050),
        channels: Some(2),
        samples: Some(4096),
    };
    let queue = audio_subsystem.open_queue::<f32, _>(None, &specs).unwrap();
    queue.resume();

    let mut t = Instant::now();
    loop {
        for event in event_pump.poll_iter() {
            handle_event(&mut bus, event);
        }


        let tstates = cpu.clock(&mut bus) * 4;

        for tstate in 0..tstates {

            ppu.tick(&mut bus);

            dma.tick(&mut bus, tstate);

            bus.timer.tick(&mut bus.iff);

            bus.apu.tick(bus.timer.read_div());
            
            if ppu.entered_vblank {
                let audio_buffer = std::mem::take(&mut bus.apu.buffer);
                queue.queue_audio(&audio_buffer).unwrap();
                ppu.entered_vblank = false;
                texture.update(None, &ppu.framebuffer, 3*160).unwrap();
                canvas.copy(&texture, None, None).unwrap();
                canvas.present();
                let elapsed = t.elapsed().as_micros() as u64;
                thread::sleep(Duration::from_micros(16750 - (elapsed).clamp(0, 16749)));
                t = Instant::now();
                // let elapsed = now.elapsed();
                // let sleeptime = Duration::from_millis(59700) - elapsed;
                // thread::sleep(sleeptime);
                // let now = Instant::now();
            }
        }
    }
}

pub fn handle_event(bus: &mut Bus, event: Event) {
    match event {
        Event::Quit{..}
        | Event::KeyDown {
            keycode: Some(Keycode::Escape),
            ..
        } => std::process::exit(0),
        
        Event::KeyDown {
            keycode: Some(key),
            ..
        } => match key {
            Keycode::Down => bus.jpad_down = true,
            Keycode::Up => bus.jpad_up = true,
            Keycode::Left => bus.jpad_left = true,
            Keycode::Right => bus.jpad_right = true,
            Keycode::Return => bus.jpad_start = true,
            Keycode::Backspace => bus.jpad_select = true,
            Keycode::X => bus.jpad_b = true,
            Keycode::Z => bus.jpad_a = true,

            Keycode::U => bus.apu.dbgch1 ^= true,
            Keycode::I => bus.apu.dbgch2 ^= true,
            Keycode::O => bus.apu.dbgch3 ^= true,
            Keycode::P => bus.apu.dbgch4 ^= true,

            Keycode::Q => bus.debug_inst ^= true,
            _ => (),
        },


        Event::KeyUp {
            keycode: Some(key),
            ..
        } => match key {
            Keycode::Down => bus.jpad_down = false,
            Keycode::Up => bus.jpad_up = false,
            Keycode::Left => bus.jpad_left = false,
            Keycode::Right => bus.jpad_right = false,
            Keycode::Return => bus.jpad_start = false,
            Keycode::Backspace => bus.jpad_select = false,
            Keycode::X => bus.jpad_b = false,
            Keycode::Z => bus.jpad_a = false,
            _ => (),
        },
        _ => (),
    }
}
/* 

7A      :LD A, D
B3      :OR A, E
20 F8 C9:JR NZ $F8 
00      :nop
00      :nop
00      :nop
1B      :DEC DE

FE 90
20 FA
F0 44

22      : LD [HLI], A
1D      : DEC E
20 FC 15: JR NZ -4
15      : DEC D
20 F9   : JR NZ -7
C9      : RET

let sign = if x == 0 {""} else if x < 0 {"-"} else {"+"}
println!("{sign}{:#04X}", x.abs())

1:
call - store [pc+1] in emu mem, inc cd
ret  - check that popped val is stored val, if so, dec cd
*/
