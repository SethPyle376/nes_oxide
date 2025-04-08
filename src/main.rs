mod cpu;
mod ppu;
mod renderer;

use crate::cpu::joypad;
pub use cpu::Bus;
pub use cpu::Cartridge;
pub use cpu::Cpu;
use rand::Rng;
use renderer::Renderer;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::collections::HashMap;
use std::time::{Duration, Instant};

const FRAME_TIME: Duration = Duration::from_nanos(16_666_667);

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut renderer = Renderer::new(&sdl_context);

    let bus = Bus::new(Cartridge::load("./ROMS/dk.nes").unwrap());
    let mut cpu = Cpu::new(bus);

    let mut last_frame = Instant::now();

    let mut key_map = HashMap::new();
    key_map.insert(Keycode::Down, joypad::Buttons::DOWN);
    key_map.insert(Keycode::Up, joypad::Buttons::UP);
    key_map.insert(Keycode::Right, joypad::Buttons::RIGHT);
    key_map.insert(Keycode::Left, joypad::Buttons::LEFT);
    key_map.insert(Keycode::Space, joypad::Buttons::SELECT);
    key_map.insert(Keycode::Return, joypad::Buttons::START);
    key_map.insert(Keycode::A, joypad::Buttons::BUTTON_A);
    key_map.insert(Keycode::S, joypad::Buttons::BUTTON_B);

    let mut inject = move |cpu: &mut Cpu, render: bool| {
        if render {
            for event in event_pump.poll_iter() {
                renderer.handle_event(&event);
                match event {
                    Event::Quit { .. } => {
                        cpu.controller.quit = true;
                    }
                    Event::KeyDown { keycode, .. } => {
                        if let Some(key) = key_map.get(&keycode.unwrap()) {
                            cpu.bus.joypad.buttons.insert(*key);
                        }

                        // let key = keycode.unwrap();
                        //
                        // match key {
                        //     Keycode::Space => cpu.controller.pause = false,
                        //     Keycode::Return => {
                        //         cpu.controller.step_mode = !cpu.controller.step_mode;
                        //         cpu.controller.pause = false;
                        //     }
                        //     _ => {}
                        // }
                    }
                    Event::KeyUp { keycode, .. } => {
                        if let Some(key) = key_map.get(&keycode.unwrap()) {
                            cpu.bus.joypad.buttons.remove(*key);
                        }
                    }
                    _ => {}
                }
            }
            renderer.render(cpu, &event_pump);

            if last_frame.elapsed() < FRAME_TIME {
                std::thread::sleep(FRAME_TIME - last_frame.elapsed());
            }
            last_frame = Instant::now();
        }
    };

    loop {
        if cpu.controller.quit {
            break;
        } else {
            cpu.step(&mut inject);
        }
    }
}
