mod cpu;
mod renderer;

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

    let bus = Bus::new(Cartridge::load("./ROMS/nestest.nes").unwrap());
    let mut cpu = Cpu::new(bus);

    let mut rng = rand::thread_rng();

    let mut inject = move |cpu: &mut Cpu| {
        let width = rng.gen_range(0..256);
        let height = rng.gen_range(0..240);
        let byte = rng.gen_range(0..3);

        renderer.frame.data[(height * 256 + width) * 3 + byte] = rng.gen_range(0..255);

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

        renderer.render(cpu, &event_pump);
    };

    loop {
        if cpu.controller.quit || cpu.cycle > 15000 {
            break;
        } else {
            cpu.step(&mut inject);
        }
    }
}
