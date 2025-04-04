mod cpu;
mod ppu;
mod renderer;

use std::time::Instant;
pub use cpu::Bus;
pub use cpu::Cartridge;
pub use cpu::Cpu;
use rand::Rng;
use renderer::Renderer;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut renderer = Renderer::new(&sdl_context);

    let bus = Bus::new(Cartridge::load("./ROMS/pacman.nes").unwrap());
    let mut cpu = Cpu::new(bus);

    let mut last_frame = Instant::now();

    let mut inject = move |cpu: &mut Cpu, render: bool| {
        if render {
                for event in event_pump.poll_iter() {
                    renderer.handle_event(&event);
                    match event {
                        Event::Quit { .. } => {
                            cpu.controller.quit = true;
                        }
                        Event::KeyDown { keycode, .. } => {
                            let key = keycode.unwrap();

                            match key {
                                Keycode::Space => cpu.controller.pause = false,
                                Keycode::Return => {
                                    cpu.controller.step_mode = !cpu.controller.step_mode;
                                    cpu.controller.pause = false;
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
            }

            if last_frame.elapsed().as_secs_f32() > 1.0 / 60.0 {
                renderer.render(cpu, &event_pump);
                last_frame = Instant::now();
            }
        }
    };

    let start = Instant::now();
    loop {
        if cpu.controller.quit {
            break;
        } else {
            cpu.step(&mut inject);
        }
    }
}
